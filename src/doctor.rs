use crate::config::Config;
use crate::sync::{
    check_all, collect_target_files, fragment_files, fragment_name, referenced_fragment_names,
    CheckIssue,
};
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A structural problem found by `doctor`. `kind` tags the category in JSON.
#[derive(serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DoctorIssue {
    /// A fragment file that no page references.
    OrphanFragment { file: String },
    /// A marker naming a fragment that doesn't exist on disk.
    OrphanMarker { name: String, pages: Vec<String> },
    /// An open marker with no matching close.
    UnpairedOpen { path: String, name: String },
    /// A close marker with no matching open.
    UnpairedClose { path: String, name: String },
    /// More than one pair of the same name in one file (only the first syncs).
    DuplicatePair { path: String, name: String },
}

/// Structured `doctor` result. `ok` is true when `issues` is empty.
#[derive(serde::Serialize)]
pub struct DoctorReport {
    pub ok: bool,
    pub issues: Vec<DoctorIssue>,
}

/// Run all health checks and collect the issues without printing.
/// The text renderer [`run_doctor`] and `doctor --json` share this.
///
/// Checks:
/// 1. Orphan fragments — fragment files in `fragments_dir` that no page references
/// 2. Orphan markers — pages reference a fragment that doesn't exist on disk
/// 3. Unpaired/duplicate markers — reused from `check`
pub fn collect(root: &Path, config: &Config) -> Result<DoctorReport> {
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

    let mut issues = Vec::new();

    // 1. Orphan fragments
    let mut orphan_frags: Vec<(&String, &String)> = frag_files
        .iter()
        .filter(|(name, _)| !references.contains_key(*name))
        .collect();
    orphan_frags.sort();
    for (_, file) in orphan_frags {
        issues.push(DoctorIssue::OrphanFragment {
            file: format!("{}/{}", config.fragments_dir, file),
        });
    }

    // 2. Orphan markers
    let mut orphan_markers: Vec<(&String, &Vec<String>)> = references
        .iter()
        .filter(|(name, _)| !frag_files.contains_key(*name))
        .collect();
    orphan_markers.sort_by(|a, b| a.0.cmp(b.0));
    for (name, pages) in orphan_markers {
        issues.push(DoctorIssue::OrphanMarker {
            name: name.clone(),
            pages: pages.clone(),
        });
    }

    // 3. Unpaired markers + duplicate pairs (reuse check_all). Stale files
    // aren't an error here — `check` covers that. Doctor focuses on
    // structural problems that sync alone won't fix.
    for issue in check_all(root, config)? {
        match issue {
            CheckIssue::UnpairedOpen { path, name } => issues.push(DoctorIssue::UnpairedOpen {
                path: path.display().to_string(),
                name,
            }),
            CheckIssue::UnpairedClose { path, name } => issues.push(DoctorIssue::UnpairedClose {
                path: path.display().to_string(),
                name,
            }),
            CheckIssue::DuplicatePair { path, name } => issues.push(DoctorIssue::DuplicatePair {
                path: path.display().to_string(),
                name,
            }),
            CheckIssue::Stale(_) => {}
        }
    }

    Ok(DoctorReport {
        ok: issues.is_empty(),
        issues,
    })
}

/// Run health checks and print a human-readable report. Returns the number
/// of issues found so the CLI can surface a non-zero exit code.
pub fn run_doctor(root: &Path, config: &Config) -> Result<usize> {
    let report = collect(root, config)?;

    for issue in &report.issues {
        match issue {
            DoctorIssue::OrphanFragment { file } => {
                println!("orphan fragment: {file} — no page references it");
            }
            DoctorIssue::OrphanMarker { name, pages } => {
                println!(
                    "orphan marker: '{}' referenced by {} but no fragment named '{}' exists in {}/",
                    name,
                    pages.join(", "),
                    name,
                    config.fragments_dir
                );
            }
            DoctorIssue::UnpairedOpen { path, name } => {
                println!("unpaired open marker '{name}' in {path}");
            }
            DoctorIssue::UnpairedClose { path, name } => {
                println!("unpaired close marker '{name}' in {path}");
            }
            DoctorIssue::DuplicatePair { path, name } => {
                println!("duplicate marker pair '{name}' in {path} (only first pair gets synced)");
            }
        }
    }

    let issues = report.issues.len();
    if issues == 0 {
        println!("fragments doctor: no issues found");
    } else {
        println!();
        println!("{issues} issue(s) found");
    }

    Ok(issues)
}
