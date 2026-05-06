mod config;
mod doctor;
mod extract;
mod init;
mod list;
mod sync;
mod watch;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use config::Config;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "fragments",
    version,
    about = "Sync shared fragments across files",
    long_about = "fragments keeps marked regions in target files identical to source \
files in `fragments/`. Every file is valid, self-contained content at all times. \
\n\nMarkers are HTML comments: `<!-- fragment:NAME -->...<!-- /fragment:NAME -->`. \
Edit `fragments/NAME.html`, run `fragments sync`, every page with the marker pair updates. \
\n\nConfig lives in `fragments.toml` (optional). See specs/fragments.md for full schema."
)]
struct Cli {
    /// Project root (contains fragments/ and target files)
    #[arg(default_value = ".")]
    root: PathBuf,

    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Sync all files with current fragment content (default)
    Sync,
    /// Watch fragments/ for changes, sync on save
    Watch,
    /// Dry-run: exit 1 if any file is stale or has unpaired markers
    Check,
    /// Create a new HTML page with marker pairs for all fragments
    Init {
        /// Filename to create (e.g. about.html)
        file: String,
    },
    /// Scan pages, detect shared blocks, extract to fragments/ and insert markers
    Extract,
    /// List every fragment and how many pages reference it
    List,
    /// Print the effective config (defaults merged with fragments.toml)
    Config,
    /// Health check: report orphan fragments, orphan markers, unpaired markers
    Doctor,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = std::fs::canonicalize(&cli.root)
        .with_context(|| format!("cannot resolve root: {}", cli.root.display()))?;

    let config = Config::load(&root)?;

    match cli.cmd.unwrap_or(Cmd::Sync) {
        Cmd::Sync => {
            let n = sync::sync_all(&root, &config)?;
            println!("fragments: updated {n} file(s)");
        }
        Cmd::Watch => {
            let n = sync::sync_all(&root, &config)?;
            println!(
                "fragments: synced {n} file(s), watching {}/ …",
                config.fragments_dir
            );
            watch::run(&root, &config)?;
        }
        Cmd::Check => {
            let issues = sync::check_all(&root, &config)?;
            if issues.is_empty() {
                println!("fragments: all files up to date");
            } else {
                for issue in &issues {
                    match issue {
                        sync::CheckIssue::Stale(p) => eprintln!("stale: {}", p.display()),
                        sync::CheckIssue::UnpairedOpen { path, name } => {
                            eprintln!("unpaired open marker '{}' in {}", name, path.display())
                        }
                        sync::CheckIssue::UnpairedClose { path, name } => {
                            eprintln!("unpaired close marker '{}' in {}", name, path.display())
                        }
                        sync::CheckIssue::DuplicatePair { path, name } => eprintln!(
                            "duplicate marker pair '{}' in {} (only first pair gets synced)",
                            name,
                            path.display()
                        ),
                    }
                }
                std::process::exit(1);
            }
        }
        Cmd::Init { file } => {
            init::init_page(&root, &file, &config)?;
        }
        Cmd::Extract => {
            let n = extract::extract_fragments(&root, &config)?;
            if n > 0 {
                println!("fragments: extraction complete, {n} page(s) updated");
            }
        }
        Cmd::List => {
            list::list_fragments(&root, &config)?;
        }
        Cmd::Config => {
            let toml = toml::to_string_pretty(&config).context("serializing config")?;
            print!("{toml}");
        }
        Cmd::Doctor => {
            let issues = doctor::run_doctor(&root, &config)?;
            if issues > 0 {
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
