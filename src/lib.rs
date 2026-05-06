//! fragments — marker-region sync for any text format.
//!
//! Library surface for downstream consumers (notably `pagekit`, the
//! HTML-specific site management tool that composes this primitive).
//!
//! # Quick reference
//!
//! - [`Config`] — load and represent project config from `fragments.toml`
//! - [`sync_all`] — replace marker-region content in target files
//! - [`check_all`] — report stale, malformed, and duplicate markers
//! - [`Fragments`] — load fragment source files for direct manipulation
//! - [`referenced_fragment_names`] — extract fragment names referenced by a page
//! - [`watch::run`] — long-running watch loop
//! - [`list::list_fragments`] — print fragment-to-page reference map
//! - [`doctor::run_doctor`] — health checks (orphans, unpaired, duplicates)

pub mod config;
pub mod doctor;
pub mod list;
pub mod sync;
pub mod watch;

pub use config::Config;
pub use sync::{
    check_all, check_all_with, referenced_fragment_names, sync_all, sync_all_with, CheckIssue,
    Fragments, SyncHook,
};
