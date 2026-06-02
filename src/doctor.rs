use crate::config::Config;
use crate::sync::{
    check_all, collect_target_files, fragment_files, fragment_name, referenced_fragment_names,
    CheckIssue,
};
use anyhow::{bail, Result};
use std::collections::HashMap;
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

    // Fragment name -> source file name (e.g. "nav" -> "nav.html"), so orphan
    // messages can show the real file regardless of its extension.
    let frag_files: HashMap<String, String> = fragment_files(&fragments_dir)?
        .iter()
        .filter_map(|p| {
            let name = fragment_name(p)?;
            let file = p.file_name()?.to_str()?.to_string();
            Some((name, file))
        })
        .collect();

    let scan_root = root.join(&config.target_dir);
    let files = collect_target_files(&scan_root, &fragments_dir, config);

    // Map fragment-name -> pages that reference it
    let mut references: HashMap<String, Vec<String>> = HashMap::new();
    for path in &files {
        let Some(syntax) = config.syntax_for(path) else {
            continue;
        };
        let content = fs::read_to_string(path)?;
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string();
        for name in referenced_fragment_names(&content, &syntax, &config.marker_prefix) {
            references.entry(name).or_default().push(rel.clone());
        }
    }

    let mut issues = 0;

    // 1. Orphan fragments
    let mut orphan_frags: Vec<(&String, &String)> = frag_files
        .iter()
        .filter(|(name, _)| !references.contains_key(*name))
        .collect();
    orphan_frags.sort();
    for (_, file) in orphan_frags {
        println!(
            "orphan fragment: {}/{} — no page references it",
            config.fragments_dir, file
        );
        issues += 1;
    }

    // 2. Orphan markers
    for (name, pages) in &references {
        if !frag_files.contains_key(name) {
            let pages_str = pages.join(", ");
            println!(
                "orphan marker: '{}' referenced by {} but no fragment named '{}' exists in {}/",
                name, pages_str, name, config.fragments_dir
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
