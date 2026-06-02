use crate::config::Config;
use crate::sync::{collect_target_files, fragment_files, fragment_name, referenced_fragment_names};
use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// One fragment and how many target pages reference it.
#[derive(serde::Serialize)]
pub struct FragmentRef {
    pub name: String,
    pub pages: usize,
}

/// Structured result of `list`: every fragment with its reference count,
/// plus totals. Serializes directly to the `list --json` payload.
#[derive(serde::Serialize)]
pub struct ListReport {
    pub fragments_dir: String,
    pub total: usize,
    pub scanned_pages: usize,
    pub fragments: Vec<FragmentRef>,
}

/// Compute the fragment-to-page reference map without printing. The text
/// renderer [`list_fragments`] and the `list --json` path share this.
pub fn collect(root: &Path, config: &Config) -> Result<ListReport> {
    let fragments_dir = root.join(&config.fragments_dir);
    if !fragments_dir.is_dir() {
        bail!(
            "no {}/ directory in {}",
            config.fragments_dir,
            root.display()
        );
    }

    let mut frag_names: Vec<String> = fragment_files(&fragments_dir)
        .with_context(|| format!("reading {}", fragments_dir.display()))?
        .iter()
        .filter_map(|p| fragment_name(p))
        .collect();
    frag_names.sort();

    let scan_root = root.join(&config.target_dir);
    let files = collect_target_files(&scan_root, &fragments_dir, config);

    let mut counts: HashMap<String, usize> = HashMap::new();
    for path in &files {
        let Some(syntax) = config.syntax_for(path) else {
            continue;
        };
        let content = fs::read_to_string(path)?;
        for name in referenced_fragment_names(&content, &syntax, &config.marker_prefix) {
            *counts.entry(name).or_insert(0) += 1;
        }
    }

    let fragments = frag_names
        .iter()
        .map(|name| FragmentRef {
            pages: counts.get(name).copied().unwrap_or(0),
            name: name.clone(),
        })
        .collect();

    Ok(ListReport {
        fragments_dir: config.fragments_dir.clone(),
        total: frag_names.len(),
        scanned_pages: files.len(),
        fragments,
    })
}

/// Print every fragment in `fragments_dir` and how many pages reference it.
pub fn list_fragments(root: &Path, config: &Config) -> Result<()> {
    let report = collect(root, config)?;

    let max_name_len = report
        .fragments
        .iter()
        .map(|f| f.name.len())
        .max()
        .unwrap_or(0);
    println!(
        "fragments in {}/ ({} total):",
        report.fragments_dir, report.total
    );
    for frag in &report.fragments {
        let suffix = if frag.pages == 0 {
            " (unreferenced)"
        } else {
            ""
        };
        println!(
            "  {name:<width$}  {count} page(s){suffix}",
            name = frag.name,
            width = max_name_len,
            count = frag.pages,
        );
    }
    println!();
    println!("scanned {} page(s)", report.scanned_pages);

    Ok(())
}
