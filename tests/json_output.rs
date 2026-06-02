//! `--json` output for `check`, `list`, and `doctor`. These guarantee the
//! machine-readable contract agent/CI consumers depend on: stable `kind`
//! tags, an `ok` flag, and exit codes that still mirror the text mode.

use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Output;
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

fn run(dir: &Path, args: &[&str]) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_fragments"))
        .arg(dir.to_str().unwrap())
        .args(args)
        .output()
        .expect("failed to run fragments")
}

fn stdout_json(out: &Output) -> Value {
    let s = String::from_utf8(out.stdout.clone()).unwrap();
    serde_json::from_str(&s).unwrap_or_else(|e| panic!("stdout was not valid JSON ({e}): {s}"))
}

#[test]
fn list_json_reports_reference_counts() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_site(
        root,
        &[("nav.html", "<nav>n</nav>"), ("orphan.html", "<p>o</p>")],
        &[(
            "index.html",
            "<!-- fragment:nav -->\n\n<!-- /fragment:nav -->",
        )],
    );

    let out = run(root, &["list", "--json"]);
    assert!(out.status.success());
    let v = stdout_json(&out);

    assert_eq!(v["fragments_dir"], "_fragments");
    assert_eq!(v["total"], 2);
    assert_eq!(v["scanned_pages"], 1);

    let frags = v["fragments"].as_array().unwrap();
    let nav = frags.iter().find(|f| f["name"] == "nav").unwrap();
    assert_eq!(nav["pages"], 1);
    let orphan = frags.iter().find(|f| f["name"] == "orphan").unwrap();
    assert_eq!(orphan["pages"], 0);
}

#[test]
fn doctor_json_flags_orphans_and_exits_nonzero() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_site(
        root,
        &[("nav.html", "<nav>n</nav>"), ("unused.html", "<p>u</p>")],
        &[(
            "index.html",
            "<!-- fragment:nav -->\n\n<!-- /fragment:nav -->\n\
             <!-- fragment:ghost -->\n<!-- /fragment:ghost -->",
        )],
    );

    let out = run(root, &["doctor", "--json"]);
    // Issues present -> exit 1, matching the text mode.
    assert_eq!(out.status.code(), Some(1));
    let v = stdout_json(&out);

    assert_eq!(v["ok"], false);
    let kinds: Vec<&str> = v["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["kind"].as_str().unwrap())
        .collect();
    // unused.html fragment is referenced by nobody; ghost marker has no source.
    assert!(kinds.contains(&"orphan_fragment"));
    assert!(kinds.contains(&"orphan_marker"));
}

#[test]
fn check_json_ok_when_synced_and_stale_when_not() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_site(
        root,
        &[("nav.html", "<nav>n</nav>")],
        &[(
            "index.html",
            "<!-- fragment:nav -->\n\n<!-- /fragment:nav -->",
        )],
    );

    // Before sync: stale -> ok=false, exit 1.
    let out = run(root, &["check", "--json"]);
    assert_eq!(out.status.code(), Some(1));
    let v = stdout_json(&out);
    assert_eq!(v["ok"], false);
    let stale = v["issues"]
        .as_array()
        .unwrap()
        .iter()
        .any(|i| i["kind"] == "stale");
    assert!(stale, "expected a stale issue before sync");

    // After sync: clean -> ok=true, exit 0, empty issues.
    assert!(run(root, &["sync"]).status.success());
    let out = run(root, &["check", "--json"]);
    assert!(out.status.success());
    let v = stdout_json(&out);
    assert_eq!(v["ok"], true);
    assert_eq!(v["issues"].as_array().unwrap().len(), 0);
}
