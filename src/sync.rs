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
            let name = path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let content = fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            entries.push((name, content));
        }

        Ok(Self {
            prefix: prefix.to_string(),
            entries,
        })
    }
}

fn replace_marker_region(html: &str, prefix: &str, name: &str, new_content: &str) -> Option<String> {
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

fn apply_fragments(html: &str, frags: &Fragments) -> Result<String> {
    let mut result = html.to_string();
    for (name, content) in &frags.entries {
        if let Some(updated) = replace_marker_region(&result, &frags.prefix, name, content) {
            result = updated;
        }
    }
    Ok(result)
}

fn collect_html_files(root: &Path, fragments_dir: &Path) -> Vec<PathBuf> {
    let tools_dir = root.join("tools");

    WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_entry(|e| {
            let p = e.path();
            !p.starts_with(fragments_dir)
                && !p.starts_with(&tools_dir)
                && !p.starts_with(&root.join("node_modules"))
                && !p.starts_with(&root.join("css"))
                && !p.starts_with(&root.join("fonts"))
        })
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map(|x| x == "html").unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect()
}

pub fn sync_all(root: &Path, config: &Config) -> Result<usize> {
    let fragments_dir = root.join(&config.fragments_dir);
    if !fragments_dir.is_dir() {
        bail!(
            "no {}/ directory in {}",
            config.fragments_dir,
            root.display()
        );
    }

    let frags = Fragments::load(&fragments_dir, &config.marker_prefix)?;
    let files = collect_html_files(root, &fragments_dir);
    let mut updated = 0;

    for path in &files {
        if sync_one(path, &frags)? {
            updated += 1;
            println!("  {}", path.strip_prefix(root).unwrap_or(path).display());
        }
    }

    Ok(updated)
}

fn sync_one(path: &Path, frags: &Fragments) -> Result<bool> {
    let current = fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    let updated = apply_fragments(&current, frags)?;

    if updated == current {
        return Ok(false);
    }

    fs::write(path, &updated)?;
    Ok(true)
}

pub fn check_all(root: &Path, config: &Config) -> Result<Vec<PathBuf>> {
    let fragments_dir = root.join(&config.fragments_dir);
    let frags = Fragments::load(&fragments_dir, &config.marker_prefix)?;
    let files = collect_html_files(root, &fragments_dir);
    let mut stale = Vec::new();

    for path in &files {
        let current = fs::read_to_string(path)?;
        let expected = apply_fragments(&current, &frags)?;
        if current != expected {
            stale.push(path.strip_prefix(root).unwrap_or(path).to_path_buf());
        }
    }

    Ok(stale)
}
