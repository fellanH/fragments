use crate::config::Config;
use crate::sync;
use anyhow::{Context, Result};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

pub fn run(root: &Path, config: &Config) -> Result<()> {
    let fragments_dir = root.join(&config.fragments_dir);

    let (tx, rx) = mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_millis(80), tx)
        .context("failed to create file watcher")?;

    debouncer
        .watcher()
        .watch(&fragments_dir, RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(Ok(_events)) => {
                println!("{}/ changed → syncing", config.fragments_dir);
                match sync::sync_all(root, config) {
                    Ok(n) => {
                        if n > 0 {
                            println!("  updated {n} file(s)");
                        } else {
                            println!("  already up to date");
                        }
                    }
                    Err(e) => eprintln!("  error: {e:#}"),
                }
            }
            Ok(Err(errs)) => {
                eprintln!("watch error: {errs}");
            }
            Err(_) => break,
        }
    }

    Ok(())
}
