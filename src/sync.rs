use crate::config::Config;
use crate::syntax::CommentSyntax;
use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct Fragments {
    prefix: String,
    entries: Vec<(String, String)>, // (marker name, file content)
}

impl Fragments {
    pub fn load(fragments_dir: &Path, prefix: &str) -> Result<Self> {
        let mut entries: Vec<(String, String)> = Vec::new();
        let mut seen: std::collections::HashMap<String, PathBuf> = std::collections::HashMap::new();

        for path in fragment_files(fragments_dir)? {
            let Some(name) = fragment_name(&path) else {
                continue;
            };
            if let Some(prev) = seen.get(&name) {
                bail!(
                    "duplicate fragment name '{}': both {} and {} resolve to it. \
                     Fragment names are file stems and must be unique.",
                    name,
                    prev.display(),
                    path.display()
                );
            }
            let content = fs::read_to_string(&path)
                .with_context(|| format!("reading fragment {}", path.display()))?;
            seen.insert(name.clone(), path);
            entries.push((name, content));
        }

        Ok(Self {
            prefix: prefix.to_string(),
            entries,
        })
    }
}

/// All fragment source files in `dir`: regular, non-hidden files, sorted by
/// name. Format-agnostic — any extension is a candidate; the fragment name is
/// the file stem. Hidden files (dotfiles) and subdirectories are skipped.
pub(crate) fn fragment_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .with_context(|| format!("cannot read {}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name()
                    .and_then(|f| f.to_str())
                    .map(|f| !f.starts_with('.'))
                    .unwrap_or(false)
        })
        .collect();
    files.sort();
    Ok(files)
}

/// The fragment name for a source file: its file stem. `None` if the stem
/// can't be derived (e.g. a non-UTF-8 name), in which case the file is skipped
/// rather than panicking.
pub(crate) fn fragment_name(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

/// Find a marker tag in `hay`. For block comments a plain substring match is
/// exact (the closing delimiter delimits the name). For line comments the
/// match must be followed by end-of-line or end-of-string, so a marker for
/// `nav` does not spuriously match inside `navbar`.
fn find_marker(hay: &str, tag: &str, is_line: bool) -> Option<usize> {
    if !is_line {
        return hay.find(tag);
    }
    let mut from = 0;
    while let Some(rel) = hay[from..].find(tag) {
        let pos = from + rel;
        let after = pos + tag.len();
        let boundary = match hay[after..].chars().next() {
            Some(c) => c == '\n' || c == '\r',
            None => true,
        };
        if boundary {
            return Some(pos);
        }
        from = after;
    }
    None
}

fn replace_marker_region(
    content: &str,
    syntax: &CommentSyntax,
    prefix: &str,
    name: &str,
    new_content: &str,
) -> Option<String> {
    let open_tag = syntax.open_marker(prefix, name);
    let close_tag = syntax.close_marker(prefix, name);

    let open_pos = find_marker(content, &open_tag, syntax.is_line())?;
    let content_start = open_pos + open_tag.len();
    let close_rel = find_marker(&content[content_start..], &close_tag, syntax.is_line())?;
    let close_start = content_start + close_rel;

    let mut result = String::with_capacity(content.len());
    result.push_str(&content[..content_start]);
    result.push('\n');
    result.push_str(new_content.trim_end());
    result.push('\n');
    result.push_str(&content[close_start..]);
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
    content: &str,
    frags: &Fragments,
    syntax: &CommentSyntax,
    target: &Path,
    root: &Path,
    hooks: &[Box<dyn SyncHook>],
) -> Result<String> {
    let mut result = content.to_string();
    for (name, frag_content) in &frags.entries {
        let mut transformed = frag_content.clone();
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
        if let Some(updated) =
            replace_marker_region(&result, syntax, &frags.prefix, name, &transformed)
        {
            result = updated;
        }
    }
    Ok(result)
}

/// Collect target files under `scan_root`: regular files whose format has a
/// known comment syntax (built-in table or `[syntax]` config). Files in
/// `fragments_dir` and `config.exclude_dirs` are skipped, and the walk is
/// bounded by `config.max_depth`.
pub(crate) fn collect_target_files(
    scan_root: &Path,
    fragments_dir: &Path,
    config: &Config,
) -> Vec<PathBuf> {
    let excluded: Vec<PathBuf> = config
        .exclude_dirs
        .iter()
        .map(|d| scan_root.join(d))
        .collect();

    WalkDir::new(scan_root)
        .max_depth(config.max_depth)
        .into_iter()
        .filter_entry(|e| {
            let p = e.path();
            !p.starts_with(fragments_dir) && !excluded.iter().any(|ex| p.starts_with(ex))
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file() && config.syntax_for(e.path()).is_some())
        .map(|e| e.into_path())
        .collect()
}

/// Sync all target files, returning the count of files updated. Equivalent
/// to [`sync_all_with`] with no hooks.
pub fn sync_all(root: &Path, config: &Config) -> Result<usize> {
    Ok(sync_all_paths(root, config)?.len())
}

/// Sync all target files, applying `hooks` to each fragment's content
/// before it's written into a marker region; returns the count of files
/// updated. Hooks chain sequentially.
///
/// Use this entry point when content needs to be transformed per-target
/// (e.g., path rewriting based on target depth, variant selection by
/// page identity). Hooks see fragment content; the fragment files on
/// disk are unchanged.
///
/// For the list of paths that changed (e.g. to print a per-file report),
/// use [`sync_all_paths_with`]; this function is a thin `.len()` wrapper
/// over it. The library never writes to stdout — formatting is the
/// caller's job, so consumers like `pagekit` get clean output.
pub fn sync_all_with(root: &Path, config: &Config, hooks: &[Box<dyn SyncHook>]) -> Result<usize> {
    Ok(sync_all_paths_with(root, config, hooks)?.len())
}

/// Sync all target files, returning the paths that were updated (in scan
/// order). Equivalent to [`sync_all_paths_with`] with no hooks.
pub fn sync_all_paths(root: &Path, config: &Config) -> Result<Vec<PathBuf>> {
    sync_all_paths_with(root, config, &[])
}

/// Sync all target files, applying `hooks`, and return the absolute paths
/// of every file whose content changed (in scan order). This is the
/// data-returning core; [`sync_all_with`] wraps it for callers that only
/// need a count. Does not print — the caller decides how to report.
pub fn sync_all_paths_with(
    root: &Path,
    config: &Config,
    hooks: &[Box<dyn SyncHook>],
) -> Result<Vec<PathBuf>> {
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
    let files = collect_target_files(&scan_root, &fragments_dir, config);
    let mut updated = Vec::new();

    for path in files {
        if sync_one(&path, root, &frags, config, hooks)? {
            updated.push(path);
        }
    }

    Ok(updated)
}

fn sync_one(
    path: &Path,
    root: &Path,
    frags: &Fragments,
    config: &Config,
    hooks: &[Box<dyn SyncHook>],
) -> Result<bool> {
    // Determined-present: collect_target_files only yields files with a
    // resolvable syntax. Guard defensively anyway.
    let Some(syntax) = config.syntax_for(path) else {
        return Ok(false);
    };
    let current =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let updated = apply_fragments(&current, frags, &syntax, path, root, hooks)?;

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

/// JSON-serializable view of a [`CheckIssue`]. The `kind` tag names the
/// problem; `name` is omitted for `stale` (which has no fragment name).
#[derive(serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CheckIssueJson {
    Stale { path: String },
    UnpairedOpen { path: String, name: String },
    UnpairedClose { path: String, name: String },
    DuplicatePair { path: String, name: String },
}

impl From<&CheckIssue> for CheckIssueJson {
    fn from(issue: &CheckIssue) -> Self {
        let p = |path: &Path| path.display().to_string();
        match issue {
            CheckIssue::Stale(path) => CheckIssueJson::Stale { path: p(path) },
            CheckIssue::UnpairedOpen { path, name } => CheckIssueJson::UnpairedOpen {
                path: p(path),
                name: name.clone(),
            },
            CheckIssue::UnpairedClose { path, name } => CheckIssueJson::UnpairedClose {
                path: p(path),
                name: name.clone(),
            },
            CheckIssue::DuplicatePair { path, name } => CheckIssueJson::DuplicatePair {
                path: p(path),
                name: name.clone(),
            },
        }
    }
}

/// JSON report for `check`: `ok` is true when no issues were found.
#[derive(serde::Serialize)]
pub struct CheckReport {
    pub ok: bool,
    pub issues: Vec<CheckIssueJson>,
}

impl CheckReport {
    pub fn from_issues(issues: &[CheckIssue]) -> Self {
        CheckReport {
            ok: issues.is_empty(),
            issues: issues.iter().map(CheckIssueJson::from).collect(),
        }
    }
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
    let files = collect_target_files(&scan_root, &fragments_dir, config);
    let mut issues = Vec::new();

    for path in &files {
        let Some(syntax) = config.syntax_for(path) else {
            continue;
        };
        let current = fs::read_to_string(path)?;
        let rel = path.strip_prefix(root).unwrap_or(path).to_path_buf();

        for issue in validate_markers(&current, &syntax, &config.marker_prefix) {
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

        let expected = apply_fragments(&current, &frags, &syntax, path, root, hooks)?;
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

/// Scan `content` for marker tags in document order, returning
/// `(is_open, name)` for each well-formed marker. Format-aware via `syntax`.
fn scan_markers(content: &str, syntax: &CommentSyntax, prefix: &str) -> Vec<(bool, String)> {
    let open_pat = syntax.open_search(prefix);
    let close_pat = syntax.close_search(prefix);
    let block_term = if syntax.is_line() {
        String::new()
    } else {
        format!(" {}", syntax.close)
    };

    let mut markers = Vec::new();
    let mut idx = 0;
    while idx < content.len() {
        let next_open = content[idx..].find(&open_pat).map(|o| idx + o);
        let next_close = content[idx..].find(&close_pat).map(|o| idx + o);

        let (start, is_open, pat_len) = match (next_open, next_close) {
            (None, None) => break,
            (Some(o), None) => (o, true, open_pat.len()),
            (None, Some(c)) => (c, false, close_pat.len()),
            (Some(o), Some(c)) => {
                if o < c {
                    (o, true, open_pat.len())
                } else {
                    (c, false, close_pat.len())
                }
            }
        };

        let name_start = start + pat_len;
        let (name_end, advance) = if syntax.is_line() {
            match content[name_start..].find('\n') {
                Some(o) => (name_start + o, name_start + o + 1),
                None => (content.len(), content.len()),
            }
        } else {
            match content[name_start..].find(&block_term) {
                Some(o) => (name_start + o, name_start + o + block_term.len()),
                None => break,
            }
        };

        let raw_name = content[name_start..name_end].trim();
        // Skip if the "name" contains spaces or other suspicious chars — it's
        // not a real marker (e.g. an ordinary comment that happens to share the
        // opening delimiter).
        if !raw_name.is_empty()
            && raw_name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            markers.push((is_open, raw_name.to_string()));
        }
        idx = advance;
    }
    markers
}

fn validate_markers(content: &str, syntax: &CommentSyntax, prefix: &str) -> Vec<MarkerIssue> {
    let markers = scan_markers(content, syntax, prefix);

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

/// Return the set of fragment names referenced in `content` via opening
/// markers. Used by `list` and `doctor` to map fragment-to-page references
/// and detect orphans. Format-aware via `syntax`.
pub fn referenced_fragment_names(
    content: &str,
    syntax: &CommentSyntax,
    prefix: &str,
) -> HashSet<String> {
    scan_markers(content, syntax, prefix)
        .into_iter()
        .filter_map(|(is_open, name)| if is_open { Some(name) } else { None })
        .collect()
}
