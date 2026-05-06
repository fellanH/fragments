//! Integration tests for the SyncHook API. Use the library directly
//! rather than spawning the binary, since hooks are a Rust-API concern.

use anyhow::Result;
use fragments::{check_all_with, sync_all_with, CheckIssue, Config, SyncHook};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn setup_site(dir: &Path, fragments: &[(&str, &str)], pages: &[(&str, &str)]) {
    let frag_dir = dir.join("_fragments");
    fs::create_dir_all(&frag_dir).unwrap();
    for (name, content) in fragments {
        fs::write(frag_dir.join(name), content).unwrap();
    }
    for (name, content) in pages {
        fs::write(dir.join(name), content).unwrap();
    }
}

// --- Identity hook: returns content unchanged ---

struct IdentityHook;
impl SyncHook for IdentityHook {
    fn transform(&self, _name: &str, content: &str, _t: &Path, _r: &Path) -> Result<String> {
        Ok(content.to_string())
    }
}

#[test]
fn identity_hook_produces_same_output_as_no_hooks() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_site(
        root,
        &[("head.html", "<meta charset=\"utf-8\">")],
        &[(
            "index.html",
            "<!-- fragment:head -->\nold\n<!-- /fragment:head -->",
        )],
    );
    let config = Config {
        fragments_dir: "_fragments".to_string(),
        ..Config::default()
    };

    let hooks: Vec<Box<dyn SyncHook>> = vec![Box::new(IdentityHook)];
    let n = sync_all_with(root, &config, &hooks).unwrap();
    assert_eq!(n, 1);

    let result = fs::read_to_string(root.join("index.html")).unwrap();
    assert!(result.contains("<meta charset=\"utf-8\">"));
}

// --- Uppercase hook: mutates content ---

struct UppercaseHook;
impl SyncHook for UppercaseHook {
    fn transform(&self, _name: &str, content: &str, _t: &Path, _r: &Path) -> Result<String> {
        Ok(content.to_uppercase())
    }
}

#[test]
fn mutating_hook_transforms_inserted_content() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_site(
        root,
        &[("head.html", "<meta charset=\"utf-8\">")],
        &[(
            "index.html",
            "<!-- fragment:head -->\nold\n<!-- /fragment:head -->",
        )],
    );
    let config = Config {
        fragments_dir: "_fragments".to_string(),
        ..Config::default()
    };

    let hooks: Vec<Box<dyn SyncHook>> = vec![Box::new(UppercaseHook)];
    sync_all_with(root, &config, &hooks).unwrap();

    let result = fs::read_to_string(root.join("index.html")).unwrap();
    assert!(
        result.contains("<META CHARSET=\"UTF-8\">"),
        "expected uppercased content, got:\n{result}"
    );
}

#[test]
fn fragment_file_on_disk_unchanged_by_hook() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_site(
        root,
        &[("head.html", "<meta charset=\"utf-8\">")],
        &[(
            "index.html",
            "<!-- fragment:head -->\nold\n<!-- /fragment:head -->",
        )],
    );
    let config = Config {
        fragments_dir: "_fragments".to_string(),
        ..Config::default()
    };

    let hooks: Vec<Box<dyn SyncHook>> = vec![Box::new(UppercaseHook)];
    sync_all_with(root, &config, &hooks).unwrap();

    // Fragment source on disk MUST be unchanged — hooks transform the
    // copy that lands in target files, not the canonical source.
    let frag = fs::read_to_string(root.join("_fragments/head.html")).unwrap();
    assert_eq!(frag, "<meta charset=\"utf-8\">");
}

// --- Target-aware hook: varies content per page ---

struct TargetNameAppender;
impl SyncHook for TargetNameAppender {
    fn transform(&self, _name: &str, content: &str, target: &Path, _r: &Path) -> Result<String> {
        let stem = target.file_stem().unwrap().to_string_lossy();
        Ok(format!("{content}<!-- on:{stem} -->"))
    }
}

#[test]
fn target_aware_hook_produces_per_page_variation() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_site(
        root,
        &[("head.html", "<meta>")],
        &[
            (
                "a.html",
                "<!-- fragment:head -->\nold\n<!-- /fragment:head -->",
            ),
            (
                "b.html",
                "<!-- fragment:head -->\nold\n<!-- /fragment:head -->",
            ),
        ],
    );
    let config = Config {
        fragments_dir: "_fragments".to_string(),
        ..Config::default()
    };

    let hooks: Vec<Box<dyn SyncHook>> = vec![Box::new(TargetNameAppender)];
    sync_all_with(root, &config, &hooks).unwrap();

    let a = fs::read_to_string(root.join("a.html")).unwrap();
    let b = fs::read_to_string(root.join("b.html")).unwrap();
    assert!(a.contains("<!-- on:a -->"), "a.html missing per-target tag");
    assert!(b.contains("<!-- on:b -->"), "b.html missing per-target tag");
    assert!(!a.contains("<!-- on:b -->"));
    assert!(!b.contains("<!-- on:a -->"));
}

// --- Multiple hooks compose in order ---

struct PrefixHook;
impl SyncHook for PrefixHook {
    fn transform(&self, _name: &str, content: &str, _t: &Path, _r: &Path) -> Result<String> {
        Ok(format!("[PRE]{content}"))
    }
}
struct SuffixHook;
impl SyncHook for SuffixHook {
    fn transform(&self, _name: &str, content: &str, _t: &Path, _r: &Path) -> Result<String> {
        Ok(format!("{content}[POST]"))
    }
}

#[test]
fn multiple_hooks_compose_in_order() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_site(
        root,
        &[("head.html", "X")],
        &[(
            "index.html",
            "<!-- fragment:head -->\nold\n<!-- /fragment:head -->",
        )],
    );
    let config = Config {
        fragments_dir: "_fragments".to_string(),
        ..Config::default()
    };

    let hooks: Vec<Box<dyn SyncHook>> = vec![Box::new(PrefixHook), Box::new(SuffixHook)];
    sync_all_with(root, &config, &hooks).unwrap();

    let result = fs::read_to_string(root.join("index.html")).unwrap();
    // PrefixHook ran first → "[PRE]X", then SuffixHook → "[PRE]X[POST]"
    assert!(
        result.contains("[PRE]X[POST]"),
        "expected composed hooks, got:\n{result}"
    );
}

// --- Hook errors propagate ---

struct FailingHook;
impl SyncHook for FailingHook {
    fn transform(&self, _name: &str, _content: &str, _t: &Path, _r: &Path) -> Result<String> {
        anyhow::bail!("intentional failure")
    }
}

#[test]
fn hook_error_propagates_and_halts_sync() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_site(
        root,
        &[("head.html", "<meta>")],
        &[(
            "index.html",
            "<!-- fragment:head -->\nold\n<!-- /fragment:head -->",
        )],
    );
    let config = Config {
        fragments_dir: "_fragments".to_string(),
        ..Config::default()
    };

    let hooks: Vec<Box<dyn SyncHook>> = vec![Box::new(FailingHook)];
    let err = sync_all_with(root, &config, &hooks).unwrap_err();
    let msg = format!("{err:#}");
    assert!(
        msg.contains("intentional failure"),
        "expected hook error, got: {msg}"
    );

    // Page should remain unmodified (sync halted before write).
    let result = fs::read_to_string(root.join("index.html")).unwrap();
    assert!(
        result.contains("\nold\n"),
        "page should be unchanged when hook fails"
    );
}

// --- check_all_with sees hook output for staleness ---

#[test]
fn check_all_with_hooks_matches_sync_all_with_hooks() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_site(
        root,
        &[("head.html", "<meta charset=\"utf-8\">")],
        &[(
            "index.html",
            "<!-- fragment:head -->\nold\n<!-- /fragment:head -->",
        )],
    );
    let config = Config {
        fragments_dir: "_fragments".to_string(),
        ..Config::default()
    };

    // First sync with the uppercase hook; then check with the same hook.
    let hooks: Vec<Box<dyn SyncHook>> = vec![Box::new(UppercaseHook)];
    sync_all_with(root, &config, &hooks).unwrap();

    let issues = check_all_with(root, &config, &hooks).unwrap();
    let stale: Vec<_> = issues
        .iter()
        .filter(|i| matches!(i, CheckIssue::Stale(_)))
        .collect();
    assert!(
        stale.is_empty(),
        "check with same hooks must report no staleness; got: {} stale",
        stale.len()
    );

    // Check WITHOUT hooks against a hook-synced page must report stale.
    let issues_no_hooks = check_all_with(root, &config, &[]).unwrap();
    let stale_no_hooks: Vec<_> = issues_no_hooks
        .iter()
        .filter(|i| matches!(i, CheckIssue::Stale(_)))
        .collect();
    assert!(
        !stale_no_hooks.is_empty(),
        "check without hooks should see hook-transformed content as stale"
    );
}
