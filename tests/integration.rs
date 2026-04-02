use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn setup_site(dir: &Path, fragments: &[(&str, &str)], pages: &[(&str, &str)]) {
    let frag_dir = dir.join("fragments");
    fs::create_dir_all(&frag_dir).unwrap();
    for (name, content) in fragments {
        fs::write(frag_dir.join(name), content).unwrap();
    }
    for (name, content) in pages {
        fs::write(dir.join(name), content).unwrap();
    }
}

fn setup_site_with_config(dir: &Path, config: &str, fragments: &[(&str, &str)], pages: &[(&str, &str)]) {
    fs::write(dir.join("fragments.toml"), config).unwrap();
    let frag_dir_name = extract_fragments_dir(config);
    let frag_dir = dir.join(frag_dir_name);
    fs::create_dir_all(&frag_dir).unwrap();
    for (name, content) in fragments {
        fs::write(frag_dir.join(name), content).unwrap();
    }
    for (name, content) in pages {
        fs::write(dir.join(name), content).unwrap();
    }
}

fn extract_fragments_dir(config: &str) -> String {
    for line in config.lines() {
        if line.starts_with("fragments_dir") {
            let val = line.split('=').nth(1).unwrap().trim().trim_matches('"');
            return val.to_string();
        }
    }
    "fragments".to_string()
}

fn run_sync(dir: &Path) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_fragments"))
        .arg(dir.to_str().unwrap())
        .arg("sync")
        .output()
        .expect("failed to run fragments")
}

fn run_check(dir: &Path) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_fragments"))
        .arg(dir.to_str().unwrap())
        .arg("check")
        .output()
        .expect("failed to run fragments")
}

fn run_init(dir: &Path, file: &str) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_fragments"))
        .arg(dir.to_str().unwrap())
        .arg("init")
        .arg(file)
        .output()
        .expect("failed to run fragments")
}

// --- Core sync behavior ---

#[test]
fn sync_replaces_marker_regions() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[
            ("head.html", "<link rel=\"stylesheet\" href=\"styles.css\">"),
            ("body-open.html", "<nav>Site Nav</nav>"),
            ("body-close.html", "<footer>Footer</footer>"),
        ],
        &[(
            "index.html",
            r#"<!DOCTYPE html>
<html>
<head>
<!-- fragment:head -->
<link rel="stylesheet" href="old.css">
<!-- /fragment:head -->
</head>
<body>
<!-- fragment:body-open -->
<nav>Old Nav</nav>
<!-- /fragment:body-open -->
<h1>Hello</h1>
<!-- fragment:body-close -->
<footer>Old Footer</footer>
<!-- /fragment:body-close -->
</body>
</html>"#,
        )],
    );

    let output = run_sync(root);
    assert!(output.status.success(), "sync failed: {:?}", output);

    let result = fs::read_to_string(root.join("index.html")).unwrap();
    assert!(result.contains("<link rel=\"stylesheet\" href=\"styles.css\">"));
    assert!(result.contains("<nav>Site Nav</nav>"));
    assert!(result.contains("<footer>Footer</footer>"));
    assert!(!result.contains("old.css"));
    assert!(!result.contains("Old Nav"));
    assert!(!result.contains("Old Footer"));
}

#[test]
fn sync_skips_unchanged_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[("head.html", "<meta charset=\"utf-8\">")],
        &[(
            "index.html",
            "<!-- fragment:head -->\n<meta charset=\"utf-8\">\n<!-- /fragment:head -->\n",
        )],
    );

    let output = run_sync(root);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("updated 0 file(s)"));
}

#[test]
fn sync_reports_updated_count() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[("head.html", "<meta charset=\"utf-8\">")],
        &[
            ("a.html", "<!-- fragment:head -->\nold\n<!-- /fragment:head -->"),
            ("b.html", "<!-- fragment:head -->\nold\n<!-- /fragment:head -->"),
            ("c.html", "<p>No markers here</p>"),
        ],
    );

    let output = run_sync(root);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("updated 2 file(s)"));
}

#[test]
fn missing_markers_silently_skipped() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[("head.html", "<meta charset=\"utf-8\">")],
        &[(
            "simple.html",
            "<!DOCTYPE html>\n<html><body><p>No markers</p></body></html>",
        )],
    );

    let output = run_sync(root);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("updated 0 file(s)"));

    let result = fs::read_to_string(root.join("simple.html")).unwrap();
    assert!(result.contains("<p>No markers</p>"));
}

#[test]
fn content_outside_markers_is_preserved() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let page = r#"<!DOCTYPE html>
<html>
<head>
<title>My Page</title>
<!-- fragment:head -->
<link rel="stylesheet" href="old.css">
<!-- /fragment:head -->
<meta name="custom" content="preserved">
</head>
<body>
<main>
  <h1>Page-specific content</h1>
  <p>This should never be touched.</p>
</main>
</body>
</html>"#;

    setup_site(
        root,
        &[("head.html", "<link rel=\"stylesheet\" href=\"new.css\">")],
        &[("index.html", page)],
    );

    run_sync(root);

    let result = fs::read_to_string(root.join("index.html")).unwrap();
    assert!(result.contains("<title>My Page</title>"));
    assert!(result.contains("<meta name=\"custom\" content=\"preserved\">"));
    assert!(result.contains("<h1>Page-specific content</h1>"));
    assert!(result.contains("new.css"));
}

// --- Check command ---

#[test]
fn check_detects_stale_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[("head.html", "<meta charset=\"utf-8\">")],
        &[(
            "index.html",
            "<!-- fragment:head -->\nstale content\n<!-- /fragment:head -->",
        )],
    );

    let output = run_check(root);
    assert!(!output.status.success(), "check should fail for stale files");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("stale"));
}

#[test]
fn check_passes_when_up_to_date() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[("head.html", "<meta charset=\"utf-8\">")],
        &[(
            "index.html",
            "<!-- fragment:head -->\nstale\n<!-- /fragment:head -->",
        )],
    );

    let sync_out = run_sync(root);
    assert!(sync_out.status.success());

    let output = run_check(root);
    assert!(output.status.success(), "check should pass after sync");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("up to date"));
}

// --- Dynamic fragment discovery ---

#[test]
fn arbitrary_fragment_names_work() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[
            ("cta.html", "<a href=\"/buy\">Buy Now</a>"),
            ("testimonials.html", "<blockquote>Great product!</blockquote>"),
        ],
        &[(
            "pricing.html",
            r#"<h1>Pricing</h1>
<!-- fragment:cta -->
<a href="/old">Old CTA</a>
<!-- /fragment:cta -->
<h2>Reviews</h2>
<!-- fragment:testimonials -->
<p>placeholder</p>
<!-- /fragment:testimonials -->"#,
        )],
    );

    let output = run_sync(root);
    assert!(output.status.success());

    let result = fs::read_to_string(root.join("pricing.html")).unwrap();
    assert!(result.contains("<a href=\"/buy\">Buy Now</a>"));
    assert!(result.contains("<blockquote>Great product!</blockquote>"));
    assert!(!result.contains("Old CTA"));
    assert!(!result.contains("placeholder"));
}

// --- Manifest config ---

#[test]
fn custom_marker_prefix() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site_with_config(
        root,
        "marker_prefix = \"sync\"\nfragments_dir = \"fragments\"\n",
        &[("nav.html", "<nav>Custom Nav</nav>")],
        &[(
            "index.html",
            "<!-- sync:nav -->\n<nav>Old</nav>\n<!-- /sync:nav -->",
        )],
    );

    let output = run_sync(root);
    assert!(output.status.success());

    let result = fs::read_to_string(root.join("index.html")).unwrap();
    assert!(result.contains("<nav>Custom Nav</nav>"));
}

#[test]
fn custom_fragments_dir() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site_with_config(
        root,
        "fragments_dir = \"inject\"\n",
        &[("head.html", "<meta charset=\"utf-8\">")],
        &[(
            "index.html",
            "<!-- fragment:head -->\nold\n<!-- /fragment:head -->",
        )],
    );

    let output = run_sync(root);
    assert!(output.status.success());

    let result = fs::read_to_string(root.join("index.html")).unwrap();
    assert!(result.contains("<meta charset=\"utf-8\">"));
}

#[test]
fn no_config_file_uses_defaults() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[("nav.html", "<nav>Nav</nav>")],
        &[(
            "index.html",
            "<!-- fragment:nav -->\n<nav>Old</nav>\n<!-- /fragment:nav -->",
        )],
    );

    let output = run_sync(root);
    assert!(output.status.success());

    let result = fs::read_to_string(root.join("index.html")).unwrap();
    assert!(result.contains("<nav>Nav</nav>"));
}

// --- Init command ---

#[test]
fn init_creates_page_with_markers() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[
            ("head.html", "<meta charset=\"utf-8\">"),
            ("body-open.html", "<nav>Nav</nav>"),
            ("cta.html", "<a>Buy</a>"),
        ],
        &[],
    );

    let output = run_init(root, "about.html");
    assert!(output.status.success(), "init failed: {:?}", output);

    let result = fs::read_to_string(root.join("about.html")).unwrap();
    assert!(result.contains("<!DOCTYPE html>"));
    assert!(result.contains("<!-- fragment:head -->"));
    assert!(result.contains("<!-- /fragment:head -->"));
    assert!(result.contains("<!-- fragment:body-open -->"));
    assert!(result.contains("<!-- fragment:cta -->"));
    assert!(result.contains("<title>about</title>"));
}

#[test]
fn init_refuses_to_overwrite() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[("head.html", "<meta>")],
        &[("index.html", "<p>existing</p>")],
    );

    let output = run_init(root, "index.html");
    assert!(!output.status.success(), "init should refuse to overwrite");

    let result = fs::read_to_string(root.join("index.html")).unwrap();
    assert!(result.contains("<p>existing</p>"), "original content preserved");
}

#[test]
fn init_then_sync_fills_markers() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[
            ("head.html", "<link rel=\"stylesheet\" href=\"styles.css\">"),
            ("body-close.html", "<footer>Footer</footer>"),
        ],
        &[],
    );

    let init_out = run_init(root, "new-page.html");
    assert!(init_out.status.success());

    let sync_out = run_sync(root);
    assert!(sync_out.status.success());

    let result = fs::read_to_string(root.join("new-page.html")).unwrap();
    assert!(result.contains("<link rel=\"stylesheet\" href=\"styles.css\">"));
    assert!(result.contains("<footer>Footer</footer>"));
}

#[test]
fn init_with_custom_prefix() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site_with_config(
        root,
        "marker_prefix = \"sync\"\nfragments_dir = \"fragments\"\n",
        &[("nav.html", "<nav>Nav</nav>")],
        &[],
    );

    let output = run_init(root, "page.html");
    assert!(output.status.success());

    let result = fs::read_to_string(root.join("page.html")).unwrap();
    assert!(result.contains("<!-- sync:nav -->"));
    assert!(result.contains("<!-- /sync:nav -->"));
}

#[test]
fn init_creates_agents_md() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[("head.html", "<meta>")],
        &[],
    );

    let output = run_init(root, "index.html");
    assert!(output.status.success());

    let agents = fs::read_to_string(root.join("fragments/AGENTS.md")).unwrap();
    assert!(agents.contains("fragments"));
    assert!(agents.contains("<!-- fragment:<name> -->"));
}

#[test]
fn init_agents_md_uses_custom_prefix() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site_with_config(
        root,
        "marker_prefix = \"sync\"\nfragments_dir = \"fragments\"\n",
        &[("nav.html", "<nav>Nav</nav>")],
        &[],
    );

    run_init(root, "page.html");

    let agents = fs::read_to_string(root.join("fragments/AGENTS.md")).unwrap();
    assert!(agents.contains("<!-- sync:<name> -->"));
}

#[test]
fn init_does_not_overwrite_agents_md() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[("head.html", "<meta>")],
        &[],
    );

    fs::write(root.join("fragments/AGENTS.md"), "custom content").unwrap();

    run_init(root, "index.html");

    let agents = fs::read_to_string(root.join("fragments/AGENTS.md")).unwrap();
    assert_eq!(agents, "custom content");
}

// --- Backwards compat: old html-sync prefix via config ---

#[test]
fn legacy_html_sync_prefix_via_config() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site_with_config(
        root,
        "marker_prefix = \"html-sync\"\n",
        &[("head.html", "<meta charset=\"utf-8\">")],
        &[(
            "index.html",
            "<!-- html-sync:head -->\nold\n<!-- /html-sync:head -->",
        )],
    );

    let output = run_sync(root);
    assert!(output.status.success());

    let result = fs::read_to_string(root.join("index.html")).unwrap();
    assert!(result.contains("<meta charset=\"utf-8\">"));
}
