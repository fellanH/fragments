use crate::config::Config;
use crate::sync::{self, SyncHook};
use anyhow::{Context, Result};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

/// Watch `fragments_dir` and resync on change. Equivalent to [`run_with`] with no hooks.
pub fn run(root: &Path, config: &Config) -> Result<()> {
    run_with(root, config, &[])
}

/// Watch `fragments_dir` and resync on change, applying `hooks` on every resync.
///
/// Consumers that pass hooks here MUST pass the same hooks to the initial
/// `sync_all_with` and any `check_all_with` calls per the consistency contract
/// in [`crate::sync::SyncHook`] — otherwise reactive resyncs will produce
/// different output than the initial sync.
pub fn run_with(root: &Path, config: &Config, hooks: &[Box<dyn SyncHook>]) -> Result<()> {
    let fragments_dir = root.join(&config.fragments_dir);

    let (tx, rx) = mpsc::channel();

    let mut debouncer =
        new_debouncer(Duration::from_millis(80), tx).context("failed to create file watcher")?;

    debouncer
        .watcher()
        .watch(&fragments_dir, RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(Ok(_events)) => {
                println!("{}/ changed → syncing", config.fragments_dir);
                match sync::sync_all_with(root, config, hooks) {
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
