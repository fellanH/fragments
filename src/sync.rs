use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Marker format: <!-- html-sync:NAME --> ... <!-- /html-sync:NAME -->
const MARKERS: &[(&str, &str)] = &[
    ("head", "head.html"),
    ("body-open", "body-open.html"),
    ("body-close", "body-close.html"),
];

fn open_tag(name: &str) -> String {
    format!("<!-- html-sync:{name} -->")
}

fn close_tag(name: &str) -> String {
    format!("<!-- /html-sync:{name} -->")
}

pub struct Fragments {
    entries: Vec<(String, String)>, // (marker name, file content)
}

impl Fragments {
    pub fn load(inject_dir: &Path) -> Result<Self> {
        let mut entries = Vec::new();
        for &(name, filename) in MARKERS {
            let p = inject_dir.join(filename);
            let content = fs::read_to_string(&p)
                .with_context(|| format!("missing inject/{filename}"))?;
            entries.push((name.to_string(), content));
        }
        Ok(Self { entries })
    }
}

fn replace_marker_region(html: &str, name: &str, new_content: &str) -> Option<String> {
    let open = open_tag(name);
    let close = close_tag(name);

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
        match replace_marker_region(&result, name, content) {
            Some(updated) => result = updated,
            None => {} // marker not present in this file — skip silently
        }
    }
    Ok(result)
}

fn collect_html_files(root: &Path) -> Vec<PathBuf> {
    let inject_dir = root.join("inject");
    let tools_dir = root.join("tools");

    WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_entry(|e| {
            let p = e.path();
            !p.starts_with(&inject_dir)
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

pub fn sync_all(root: &Path) -> Result<usize> {
    let inject_dir = root.join("inject");
    if !inject_dir.is_dir() {
        bail!("no inject/ directory in {}", root.display());
    }

    let frags = Fragments::load(&inject_dir)?;
    let files = collect_html_files(root);
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

pub fn check_all(root: &Path) -> Result<Vec<PathBuf>> {
    let inject_dir = root.join("inject");
    let frags = Fragments::load(&inject_dir)?;
    let files = collect_html_files(root);
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
