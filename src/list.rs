use crate::config::Config;
use crate::sync::{collect_target_files, fragment_files, fragment_name, referenced_fragment_names};
use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Print every fragment in `fragments_dir` and how many pages reference it.
pub fn list_fragments(root: &Path, config: &Config) -> Result<()> {
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

    let max_name_len = frag_names.iter().map(|n| n.len()).max().unwrap_or(0);
    println!(
        "fragments in {}/ ({} total):",
        config.fragments_dir,
        frag_names.len()
    );
    for name in &frag_names {
        let count = counts.get(name).copied().unwrap_or(0);
        let suffix = if count == 0 { " (unreferenced)" } else { "" };
        println!(
            "  {name:<width$}  {count} page(s){suffix}",
            width = max_name_len
        );
    }
    println!();
    println!("scanned {} page(s)", files.len());

    Ok(())
}
