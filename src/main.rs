use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use fragments::Config;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "fragments",
    version,
    about = "Marker-region sync for any text format",
    long_about = "fragments keeps marked regions in target files identical to source \
files in `fragments/`. Format-agnostic — works on any text file with comment-pair syntax. \
Every file is valid in its native format at all times. \
\n\nFor HTML-specific helpers (page scaffolding, DOM-aware extraction), see `pagekit`, \
which composes this primitive. \
\n\nConfig lives in `fragments.toml` (optional). See specs/fragments.md for the schema."
)]
struct Cli {
    /// Project root (contains fragments/ and target files)
    #[arg(default_value = ".")]
    root: PathBuf,

    #[command(subcommand)]
    cmd: Option<Cmd>,

    /// Emit machine-readable JSON instead of human text (check, list, doctor)
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Cmd {
    /// Sync all files with current fragment content (default)
    Sync,
    /// Watch fragments/ for changes, sync on save
    Watch,
    /// Dry-run: exit 1 if any file is stale or has malformed markers
    Check,
    /// List every fragment and how many target files reference it
    List,
    /// Print the effective config (defaults merged with fragments.toml)
    Config,
    /// Health check: report orphan fragments, orphan markers, malformed markers
    Doctor,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = std::fs::canonicalize(&cli.root)
        .with_context(|| format!("cannot resolve root: {}", cli.root.display()))?;

    let config = Config::load(&root)?;

    match cli.cmd.unwrap_or(Cmd::Sync) {
        Cmd::Sync => {
            let updated = fragments::sync_all_paths(&root, &config)?;
            for path in &updated {
                println!("  {}", path.strip_prefix(&root).unwrap_or(path).display());
            }
            println!("fragments: updated {} file(s)", updated.len());
        }
        Cmd::Watch => {
            let n = fragments::sync_all(&root, &config)?;
            println!(
                "fragments: synced {n} file(s), watching {}/ …",
                config.fragments_dir
            );
            fragments::watch::run(&root, &config)?;
        }
        Cmd::Check => {
            let issues = fragments::check_all(&root, &config)?;
            if cli.json {
                let report = fragments::sync::CheckReport::from_issues(&issues);
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else if issues.is_empty() {
                println!("fragments: all files up to date");
            } else {
                for issue in &issues {
                    match issue {
                        fragments::CheckIssue::Stale(p) => eprintln!("stale: {}", p.display()),
                        fragments::CheckIssue::UnpairedOpen { path, name } => {
                            eprintln!("unpaired open marker '{}' in {}", name, path.display())
                        }
                        fragments::CheckIssue::UnpairedClose { path, name } => {
                            eprintln!("unpaired close marker '{}' in {}", name, path.display())
                        }
                        fragments::CheckIssue::DuplicatePair { path, name } => eprintln!(
                            "duplicate marker pair '{}' in {} (only first pair gets synced)",
                            name,
                            path.display()
                        ),
                    }
                }
            }
            if !issues.is_empty() {
                std::process::exit(1);
            }
        }
        Cmd::List => {
            if cli.json {
                let report = fragments::list::collect(&root, &config)?;
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                fragments::list::list_fragments(&root, &config)?;
            }
        }
        Cmd::Config => {
            let toml = toml::to_string_pretty(&config).context("serializing config")?;
            print!("{toml}");
        }
        Cmd::Doctor => {
            let issues = if cli.json {
                let report = fragments::doctor::collect(&root, &config)?;
                println!("{}", serde_json::to_string_pretty(&report)?);
                report.issues.len()
            } else {
                fragments::doctor::run_doctor(&root, &config)?
            };
            if issues > 0 {
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
