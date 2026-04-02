mod sync;
mod watch;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "html-sync", about = "Global variables for HTML")]
struct Cli {
    /// Project root (contains inject/ and *.html files)
    #[arg(default_value = ".")]
    root: PathBuf,

    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Sync all HTML files with current inject/ content (default)
    Sync,
    /// Watch inject/ for changes, update HTML files on save
    Watch,
    /// Dry-run: exit 1 if any HTML file has stale markers
    Check,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = std::fs::canonicalize(&cli.root)
        .with_context(|| format!("cannot resolve root: {}", cli.root.display()))?;

    match cli.cmd.unwrap_or(Cmd::Sync) {
        Cmd::Sync => {
            let n = sync::sync_all(&root)?;
            println!("html-sync: updated {n} file(s)");
        }
        Cmd::Watch => {
            let n = sync::sync_all(&root)?;
            println!("html-sync: synced {n} file(s), watching inject/ …");
            watch::run(&root)?;
        }
        Cmd::Check => {
            let stale = sync::check_all(&root)?;
            if stale.is_empty() {
                println!("html-sync: all files up to date");
            } else {
                for p in &stale {
                    eprintln!("stale: {}", p.display());
                }
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
