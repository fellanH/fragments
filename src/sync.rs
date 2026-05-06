use crate::config::Config;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn open_tag(prefix: &str, name: &str) -> String {
    format!("<!-- {prefix}:{name} -->")
}

fn close_tag(prefix: &str, name: &str) -> String {
    format!("<!-- /{prefix}:{name} -->")
}

pub struct Fragments {
    prefix: String,
    entries: Vec<(String, String)>, // (marker name, file content)
}

impl Fragments {
    pub fn load(fragments_dir: &Path, prefix: &str) -> Result<Self> {
        let mut entries = Vec::new();

        let mut files: Vec<_> = fs::read_dir(fragments_dir)
            .with_context(|| format!("cannot read {}", fragments_dir.display()))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "html")
                    .unwrap_or(false)
            })
            .collect();

        files.sort_by_key(|e| e.file_name());

        for entry in files {
            let path = entry.path();
            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            let content =
                fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
            entries.push((name, content));
        }

        Ok(Self {
            prefix: prefix.to_string(),
            entries,
        })
    }
}

fn replace_marker_region(
    html: &str,
    prefix: &str,
    name: &str,
    new_content: &str,
) -> Option<String> {
    let open = open_tag(prefix, name);
    let close = close_tag(prefix, name);

    let open_start = html.find(&open)?;
    let content_start = open_start + open.len();
    let close_start = html[content_start..].find(&close)? + content_start;

    let mut result = String::with_capacity(html.len());
    result.push_str(&html[..content_start]);
    result.push('\n');
    result.push_str(new_content.trim_end());
    result.push('\n');
    result.push_str(&html[close_start..]);
    Some(result)
}

/// A transform applied to fragment content per target file before
/// the content is inserted into the marker region.
///
/// Hooks chain sequentially: each hook receives the output of the
/// previous hook (the first hook receives the canonical fragment
/// content from disk).
///
/// The fragment file on disk is never modified — transforms apply only
/// to the copy that lands inside the target's marker region. This lets
/// consumers do per-target adaptations (path rewriting, variant
/// selection, format-specific escaping) without splitting the canonical
/// source into per-target derived files.
///
/// Reference consumer: `pagekit`. Sprint 4 D2 uses a hook to rewrite
/// relative hrefs based on the target page's directory depth.
pub trait SyncHook: Send + Sync {
    /// Transform `content` for a specific `target` file.
    ///
    /// - `name`: the fragment name (allows hooks to scope by name)
    /// - `content`: the fragment content as it stands after prior hooks
    /// - `target`: the file path the content is being inserted into
    /// - `root`: the project root (useful for computing relative paths)
    ///
    /// Return the transformed content. For identity (no change), return
    /// `content.to_string()`. To halt sync, return `Err(...)`.
    fn transform(&self, name: &str, content: &str, target: &Path, root: &Path) -> Result<String>;
}

fn apply_fragments(
    html: &str,
    frags: &Fragments,
    target: &Path,
    root: &Path,
    hooks: &[Box<dyn SyncHook>],
) -> Result<String> {
    let mut result = html.to_string();
    for (name, content) in &frags.entries {
        let mut transformed = content.clone();
        for hook in hooks {
            transformed = hook
                .transform(name, &transformed, target, root)
                .with_context(|| {
                    format!(
                        "sync hook failed on fragment '{}' for target {}",
                        name,
                        target.display()
                    )
                })?;
        }
        if let Some(updated) = replace_marker_region(&result, &frags.prefix, name, &transformed) {
            result = updated;
        }
    }
    Ok(result)
}

pub(crate) fn collect_html_files(
    scan_root: &Path,
    fragments_dir: &Path,
    exclude_dirs: &[String],
    max_depth: usize,
) -> Vec<PathBuf> {
    let excluded: Vec<PathBuf> = exclude_dirs.iter().map(|d| scan_root.join(d)).collect();

    WalkDir::new(scan_root)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| {
            let p = e.path();
            !p.starts_with(fragments_dir) && !excluded.iter().any(|ex| p.starts_with(ex))
        })
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file() && e.path().extension().map(|x| x == "html").unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect()
}

/// Sync all target files. Equivalent to [`sync_all_with`] with no hooks.
pub fn sync_all(root: &Path, config: &Config) -> Result<usize> {
    sync_all_with(root, config, &[])
}

/// Sync all target files, applying `hooks` to each fragment's content
/// before it's written into a marker region. Hooks chain sequentially.
///
/// Use this entry point when content needs to be transformed per-target
/// (e.g., path rewriting based on target depth, variant selection by
/// page identity). Hooks see fragment content; the fragment files on
/// disk are unchanged.
pub fn sync_all_with(root: &Path, config: &Config, hooks: &[Box<dyn SyncHook>]) -> Result<usize> {
    let fragments_dir = root.join(&config.fragments_dir);
    if !fragments_dir.is_dir() {
        bail!(
            "no {}/ directory in {}",
            config.fragments_dir,
            root.display()
        );
    }

    let scan_root = root.join(&config.target_dir);
    if !scan_root.is_dir() {
        bail!(
            "target_dir {}/ not found in {}",
            config.target_dir,
            root.display()
        );
    }

    let frags = Fragments::load(&fragments_dir, &config.marker_prefix)?;
    let files = collect_html_files(
        &scan_root,
        &fragments_dir,
        &config.exclude_dirs,
        config.max_depth,
    );
    let mut updated = 0;

    for path in &files {
        if sync_one(path, root, &frags, hooks)? {
            updated += 1;
            println!("  {}", path.strip_prefix(root).unwrap_or(path).display());
        }
    }

    Ok(updated)
}

fn sync_one(
    path: &Path,
    root: &Path,
    frags: &Fragments,
    hooks: &[Box<dyn SyncHook>],
) -> Result<bool> {
    let current =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let updated = apply_fragments(&current, frags, path, root, hooks)?;

    if updated == current {
        return Ok(false);
    }

    // Direct truncate-and-write. A SIGKILL or power loss between the
    // truncate and the last byte hitting disk leaves a partial file;
    // recovery is `fragments sync` again (idempotent). Trade chosen
    // deliberately over tempfile+rename to keep inode/perms/xattrs intact.
    fs::write(path, &updated).with_context(|| format!("writing {}", path.display()))?;
    Ok(true)
}

pub enum CheckIssue {
    Stale(PathBuf),
    UnpairedOpen {
        path: PathBuf,
        name: String,
    },
    UnpairedClose {
        path: PathBuf,
        name: String,
    },
    /// Same fragment name has more than one open+close pair in a single
    /// page. Only the first pair gets synced (`replace_marker_region` uses
    /// `find` which returns the first match), so subsequent pairs silently
    /// drift stale relative to the first.
    DuplicatePair {
        path: PathBuf,
        name: String,
    },
}

/// Check all target files for stale, malformed, or duplicate markers.
/// Equivalent to [`check_all_with`] with no hooks.
pub fn check_all(root: &Path, config: &Config) -> Result<Vec<CheckIssue>> {
    check_all_with(root, config, &[])
}

/// Check all target files, applying `hooks` when computing the expected
/// content for each target. A target is reported as `Stale` only if its
/// current content differs from what `sync_all_with(hooks)` would write.
///
/// Consumers that pass hooks to `sync_all_with` MUST pass the same hooks
/// to `check_all_with` for staleness reports to be consistent. CI gates
/// that don't match the sync configuration will produce false positives.
pub fn check_all_with(
    root: &Path,
    config: &Config,
    hooks: &[Box<dyn SyncHook>],
) -> Result<Vec<CheckIssue>> {
    let fragments_dir = root.join(&config.fragments_dir);
    let scan_root = root.join(&config.target_dir);
    let frags = Fragments::load(&fragments_dir, &config.marker_prefix)?;
    let files = collect_html_files(
        &scan_root,
        &fragments_dir,
        &config.exclude_dirs,
        config.max_depth,
    );
    let mut issues = Vec::new();

    for path in &files {
        let current = fs::read_to_string(path)?;
        let rel = path.strip_prefix(root).unwrap_or(path).to_path_buf();

        for issue in validate_markers(&current, &config.marker_prefix) {
            match issue {
                MarkerIssue::UnpairedOpen(name) => issues.push(CheckIssue::UnpairedOpen {
                    path: rel.clone(),
                    name,
                }),
                MarkerIssue::UnpairedClose(name) => issues.push(CheckIssue::UnpairedClose {
                    path: rel.clone(),
                    name,
                }),
                MarkerIssue::DuplicatePair(name) => issues.push(CheckIssue::DuplicatePair {
                    path: rel.clone(),
                    name,
                }),
            }
        }

        let expected = apply_fragments(&current, &frags, path, root, hooks)?;
        if current != expected {
            issues.push(CheckIssue::Stale(rel));
        }
    }

    Ok(issues)
}

enum MarkerIssue {
    UnpairedOpen(String),
    UnpairedClose(String),
    DuplicatePair(String),
}

fn validate_markers(html: &str, prefix: &str) -> Vec<MarkerIssue> {
    let open_prefix = format!("<!-- {prefix}:");
    let close_prefix = format!("<!-- /{prefix}:");
    let suffix = " -->";

    let mut markers: Vec<(bool, String)> = Vec::new();
    let mut idx = 0;
    while idx < html.len() {
        let next_open = html[idx..].find(&open_prefix);
        let next_close = html[idx..].find(&close_prefix);

        let (start, is_open, prefix_len) = match (next_open, next_close) {
            (None, None) => break,
            (Some(o), None) => (idx + o, true, open_prefix.len()),
            (None, Some(c)) => (idx + c, false, close_prefix.len()),
            (Some(o), Some(c)) => {
                if idx + o < idx + c {
                    (idx + o, true, open_prefix.len())
                } else {
                    (idx + c, false, close_prefix.len())
                }
            }
        };

        let name_start = start + prefix_len;
        let Some(suffix_offset) = html[name_start..].find(suffix) else {
            break;
        };
        let raw_name = html[name_start..name_start + suffix_offset].trim();
        // Skip if the "name" contains spaces or other suspicious chars — it's not a real marker
        if !raw_name.is_empty()
            && raw_name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            markers.push((is_open, raw_name.to_string()));
        }
        idx = name_start + suffix_offset + suffix.len();
    }

    let mut stack: Vec<String> = Vec::new();
    let mut completed: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut issues = Vec::new();
    for (is_open, name) in markers {
        if is_open {
            stack.push(name);
        } else if stack.last() == Some(&name) {
            stack.pop();
            *completed.entry(name).or_insert(0) += 1;
        } else {
            issues.push(MarkerIssue::UnpairedClose(name));
        }
    }
    for name in stack {
        issues.push(MarkerIssue::UnpairedOpen(name));
    }
    // Each name should have at most one completed pair per file. More
    // than one means only the first pair gets synced (silent drift).
    let mut dup_names: Vec<String> = completed
        .into_iter()
        .filter_map(|(name, count)| if count > 1 { Some(name) } else { None })
        .collect();
    dup_names.sort();
    for name in dup_names {
        issues.push(MarkerIssue::DuplicatePair(name));
    }
    issues
}

/// Return the set of fragment names referenced in `html` via opening
/// markers (`<!-- prefix:NAME -->`). Used by `list` and `doctor` to map
/// fragment-to-page references and detect orphans.
pub fn referenced_fragment_names(html: &str, prefix: &str) -> std::collections::HashSet<String> {
    let open_prefix = format!("<!-- {prefix}:");
    let suffix = " -->";
    let mut names = std::collections::HashSet::new();
    let mut idx = 0;
    while idx < html.len() {
        let Some(rel) = html[idx..].find(&open_prefix) else {
            break;
        };
        let name_start = idx + rel + open_prefix.len();
        let Some(suffix_off) = html[name_start..].find(suffix) else {
            break;
        };
        let raw_name = html[name_start..name_start + suffix_off].trim();
        if !raw_name.is_empty()
            && raw_name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            names.insert(raw_name.to_string());
        }
        idx = name_start + suffix_off + suffix.len();
    }
    names
}
