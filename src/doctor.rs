use crate::config::Config;
use crate::sync::{check_all, collect_html_files, referenced_fragment_names, CheckIssue};
use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Run a battery of health checks against the project. Returns the number
/// of issues found so the CLI can surface a non-zero exit code.
///
/// Checks:
/// 1. Orphan fragments — fragment files in `fragments_dir` that no page references
/// 2. Orphan markers — pages reference a fragment that doesn't exist on disk
/// 3. Unpaired markers — open without close (or vice versa); reused from `check`
pub fn run_doctor(root: &Path, config: &Config) -> Result<usize> {
    let fragments_dir = root.join(&config.fragments_dir);
    if !fragments_dir.is_dir() {
        bail!(
            "no {}/ directory in {}",
            config.fragments_dir,
            root.display()
        );
    }

    let frag_names: HashSet<String> = fs::read_dir(&fragments_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "html").unwrap_or(false))
        .map(|e| e.path().file_stem().unwrap().to_string_lossy().to_string())
        .collect();

    let scan_root = root.join(&config.target_dir);
    let files = collect_html_files(
        &scan_root,
        &fragments_dir,
        &config.exclude_dirs,
        config.max_depth,
    );

    // Map fragment-name -> pages that reference it
    let mut references: HashMap<String, Vec<String>> = HashMap::new();
    for path in &files {
        let content = fs::read_to_string(path)?;
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string();
        for name in referenced_fragment_names(&content, &config.marker_prefix) {
            references.entry(name).or_default().push(rel.clone());
        }
    }

    let mut issues = 0;

    // 1. Orphan fragments
    for frag in &frag_names {
        if !references.contains_key(frag) {
            println!(
                "orphan fragment: {}/{}.html — no page references it",
                config.fragments_dir, frag
            );
            issues += 1;
        }
    }

    // 2. Orphan markers
    for (name, pages) in &references {
        if !frag_names.contains(name) {
            let pages_str = pages.join(", ");
            println!(
                "orphan marker: '{}' referenced by {} but no {}/{}.html exists",
                name, pages_str, config.fragments_dir, name
            );
            issues += 1;
        }
    }

    // 3. Unpaired markers + duplicate pairs (reuse check_all)
    for issue in check_all(root, config)? {
        match issue {
            CheckIssue::UnpairedOpen { path, name } => {
                println!("unpaired open marker '{}' in {}", name, path.display());
                issues += 1;
            }
            CheckIssue::UnpairedClose { path, name } => {
                println!("unpaired close marker '{}' in {}", name, path.display());
                issues += 1;
            }
            CheckIssue::DuplicatePair { path, name } => {
                println!(
                    "duplicate marker pair '{}' in {} (only first pair gets synced)",
                    name,
                    path.display()
                );
                issues += 1;
            }
            // Stale files aren't an error here — `check` covers that. Doctor
            // focuses on structural problems that sync alone won't fix.
            CheckIssue::Stale(_) => {}
        }
    }

    if issues == 0 {
        println!("fragments doctor: no issues found");
    } else {
        println!();
        println!("{issues} issue(s) found");
    }

    Ok(issues)
}
