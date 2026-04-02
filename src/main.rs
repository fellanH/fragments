mod config;
mod init;
mod sync;
mod watch;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use config::Config;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "fragments", about = "Sync shared fragments across files")]
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
    /// Dry-run: exit 1 if any file has stale markers
    Check,
    /// Create a new HTML page with marker pairs for all fragments
    Init {
        /// Filename to create (e.g. about.html)
        file: String,
    },
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
            let stale = sync::check_all(&root, &config)?;
            if stale.is_empty() {
                println!("fragments: all files up to date");
            } else {
                for p in &stale {
                    eprintln!("stale: {}", p.display());
                }
                std::process::exit(1);
            }
        }
        Cmd::Init { file } => {
            init::init_page(&root, &file, &config)?;
        }
    }
    Ok(())
}
