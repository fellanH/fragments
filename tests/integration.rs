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

fn setup_site_with_config(
    dir: &Path,
    config: &str,
    fragments: &[(&str, &str)],
    pages: &[(&str, &str)],
) {
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
    "_fragments".to_string()
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
            (
                "a.html",
                "<!-- fragment:head -->\nold\n<!-- /fragment:head -->",
            ),
            (
                "b.html",
                "<!-- fragment:head -->\nold\n<!-- /fragment:head -->",
            ),
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
    assert!(
        !output.status.success(),
        "check should fail for stale files"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("stale"));
}

#[test]
fn check_detects_unpaired_open_marker() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[("head.html", "<meta>")],
        &[("broken.html", "<!-- fragment:head -->\nno close marker\n")],
    );

    let output = run_check(root);
    assert!(
        !output.status.success(),
        "check should fail on unpaired open"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unpaired open"), "stderr: {stderr}");
    assert!(stderr.contains("head"));
}

#[test]
fn check_detects_unpaired_close_marker() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[("head.html", "<meta>")],
        &[("broken.html", "<p>orphan</p>\n<!-- /fragment:head -->")],
    );

    let output = run_check(root);
    assert!(
        !output.status.success(),
        "check should fail on unpaired close"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unpaired close"), "stderr: {stderr}");
    assert!(stderr.contains("head"));
}

#[test]
fn check_detects_duplicate_marker_pair() {
    // Same fragment name has two open+close pairs in one page. Only the
    // first pair gets synced (replace_marker_region uses first-find), so
    // subsequent pairs silently drift stale.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[("nav.html", "<nav>Nav</nav>")],
        &[(
            "two-navs.html",
            "<!-- fragment:nav -->\n<nav>A</nav>\n<!-- /fragment:nav -->\n\
             <p>middle</p>\n\
             <!-- fragment:nav -->\n<nav>B</nav>\n<!-- /fragment:nav -->",
        )],
    );

    let output = run_check(root);
    assert!(
        !output.status.success(),
        "check should fail on duplicate marker pair"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("duplicate marker pair"), "stderr: {stderr}");
    assert!(stderr.contains("nav"));
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
            (
                "testimonials.html",
                "<blockquote>Great product!</blockquote>",
            ),
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
fn exclude_dirs_skips_listed_subdirectories() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::write(
        root.join("fragments.toml"),
        "exclude_dirs = [\"dist\", \"build\"]\n",
    )
    .unwrap();

    let frag_dir = root.join("_fragments");
    fs::create_dir_all(&frag_dir).unwrap();
    fs::write(frag_dir.join("head.html"), "<meta charset=\"utf-8\">").unwrap();

    let stale_marker = "<!-- fragment:head -->\nstale\n<!-- /fragment:head -->";
    fs::write(root.join("index.html"), stale_marker).unwrap();
    fs::create_dir_all(root.join("dist")).unwrap();
    fs::write(root.join("dist").join("old.html"), stale_marker).unwrap();
    fs::create_dir_all(root.join("build")).unwrap();
    fs::write(root.join("build").join("old.html"), stale_marker).unwrap();

    let output = run_sync(root);
    assert!(output.status.success());

    // Root index.html got synced; dist/ and build/ pages were skipped.
    let root_idx = fs::read_to_string(root.join("index.html")).unwrap();
    assert!(root_idx.contains("<meta charset=\"utf-8\">"));

    let dist_old = fs::read_to_string(root.join("dist/old.html")).unwrap();
    assert!(
        dist_old.contains("stale"),
        "dist/ should not have been scanned"
    );
    let build_old = fs::read_to_string(root.join("build/old.html")).unwrap();
    assert!(
        build_old.contains("stale"),
        "build/ should not have been scanned"
    );
}

#[test]
fn max_depth_caps_walk_depth() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::write(root.join("fragments.toml"), "max_depth = 2\n").unwrap();

    let frag_dir = root.join("_fragments");
    fs::create_dir_all(&frag_dir).unwrap();
    fs::write(frag_dir.join("head.html"), "<meta charset=\"utf-8\">").unwrap();

    let stale = "<!-- fragment:head -->\nstale\n<!-- /fragment:head -->";

    // depth 1: root/index.html — should be scanned
    fs::write(root.join("index.html"), stale).unwrap();
    // depth 3: root/a/b/deep.html — beyond max_depth=2
    fs::create_dir_all(root.join("a/b")).unwrap();
    fs::write(root.join("a/b/deep.html"), stale).unwrap();

    let output = run_sync(root);
    assert!(output.status.success());

    let shallow = fs::read_to_string(root.join("index.html")).unwrap();
    assert!(shallow.contains("<meta charset=\"utf-8\">"));

    let deep = fs::read_to_string(root.join("a/b/deep.html")).unwrap();
    assert!(
        deep.contains("stale"),
        "deep page should not have been scanned"
    );
}

#[test]
fn max_depth_default_reaches_typical_blog_layout() {
    // Blog posts at <root>/blog/<slug>/index.html sit at depth 3.
    // Default max_depth must reach them.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(root, &[("head.html", "<meta>")], &[]);
    fs::create_dir_all(root.join("blog/post-one")).unwrap();
    fs::write(
        root.join("blog/post-one/index.html"),
        "<!-- fragment:head -->\nstale\n<!-- /fragment:head -->",
    )
    .unwrap();

    let output = run_sync(root);
    assert!(output.status.success());

    let post = fs::read_to_string(root.join("blog/post-one/index.html")).unwrap();
    assert!(
        post.contains("<meta>"),
        "blog post at depth 3 should sync under default max_depth"
    );
}

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

// --- target_dir: separate fragments from HTML files ---

#[test]
fn target_dir_scans_subdirectory() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::write(root.join("fragments.toml"), "target_dir = \"www\"\n").unwrap();

    let frag_dir = root.join("_fragments");
    fs::create_dir_all(&frag_dir).unwrap();
    fs::write(frag_dir.join("head.html"), "<meta charset=\"utf-8\">").unwrap();
    fs::write(frag_dir.join("nav.html"), "<nav>Main Nav</nav>").unwrap();

    let www = root.join("www");
    fs::create_dir_all(&www).unwrap();
    fs::write(
        www.join("index.html"),
        "<!-- fragment:head -->\nold\n<!-- /fragment:head -->\n<!-- fragment:nav -->\nold\n<!-- /fragment:nav -->",
    )
    .unwrap();

    let output = run_sync(root);
    assert!(output.status.success(), "sync failed: {:?}", output);

    let result = fs::read_to_string(www.join("index.html")).unwrap();
    assert!(result.contains("<meta charset=\"utf-8\">"));
    assert!(result.contains("<nav>Main Nav</nav>"));
    assert!(!result.contains("\nold\n"));
}

#[test]
fn target_dir_check_works() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::write(root.join("fragments.toml"), "target_dir = \"www\"\n").unwrap();

    let frag_dir = root.join("_fragments");
    fs::create_dir_all(&frag_dir).unwrap();
    fs::write(frag_dir.join("head.html"), "<meta charset=\"utf-8\">").unwrap();

    let www = root.join("www");
    fs::create_dir_all(&www).unwrap();
    fs::write(
        www.join("index.html"),
        "<!-- fragment:head -->\nstale\n<!-- /fragment:head -->",
    )
    .unwrap();

    let check_out = run_check(root);
    assert!(!check_out.status.success(), "check should fail for stale");

    run_sync(root);

    let check_out = run_check(root);
    assert!(check_out.status.success(), "check should pass after sync");
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

// --- List command ---

fn run_list(dir: &Path) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_fragments"))
        .arg(dir.to_str().unwrap())
        .arg("list")
        .output()
        .expect("failed to run fragments")
}

#[test]
fn list_shows_fragments_with_reference_counts() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[
            ("nav.html", "<nav>Nav</nav>"),
            ("footer.html", "<footer>F</footer>"),
            ("orphan.html", "<p>not used</p>"),
        ],
        &[
            (
                "a.html",
                "<!-- fragment:nav -->old<!-- /fragment:nav -->\n<!-- fragment:footer -->old<!-- /fragment:footer -->",
            ),
            (
                "b.html",
                "<!-- fragment:footer -->old<!-- /fragment:footer -->",
            ),
        ],
    );

    let output = run_list(root);
    assert!(output.status.success(), "list failed: {:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("nav"));
    assert!(stdout.contains("footer"));
    assert!(stdout.contains("orphan"));
    assert!(stdout.contains("(unreferenced)"), "stdout: {stdout}");
    assert!(stdout.contains("scanned 2 page(s)"));
}

// --- Config command ---

fn run_config(dir: &Path) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_fragments"))
        .arg(dir.to_str().unwrap())
        .arg("config")
        .output()
        .expect("failed to run fragments")
}

#[test]
fn config_print_shows_defaults_when_no_toml() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("_fragments")).unwrap();

    let output = run_config(root);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("marker_prefix = \"fragment\""));
    // Default folder is `_fragments` (underscore prefix) so static-site
    // hosts treat it as infrastructure and skip it during deploy.
    assert!(stdout.contains("fragments_dir = \"_fragments\""));
    assert!(stdout.contains("max_depth = 5"));
    // fragments core has no built-in exclude defaults — config-over-convention.
    // Format-specific defaults (e.g. css/fonts for HTML sites) live in pagekit
    // or per-project fragments.toml.
    assert!(stdout.contains("exclude_dirs = []"));
}

#[test]
fn config_print_reflects_overrides() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("frags")).unwrap();
    fs::write(
        root.join("fragments.toml"),
        "marker_prefix = \"sync\"\nfragments_dir = \"frags\"\nmax_depth = 9\n",
    )
    .unwrap();

    let output = run_config(root);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("marker_prefix = \"sync\""));
    assert!(stdout.contains("fragments_dir = \"frags\""));
    assert!(stdout.contains("max_depth = 9"));
}

// --- Doctor command ---

fn run_doctor(dir: &Path) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_fragments"))
        .arg(dir.to_str().unwrap())
        .arg("doctor")
        .output()
        .expect("failed to run fragments")
}

#[test]
fn doctor_clean_site_reports_no_issues() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[("nav.html", "<nav>Nav</nav>")],
        &[(
            "a.html",
            "<!-- fragment:nav -->\n<nav>Nav</nav>\n<!-- /fragment:nav -->",
        )],
    );

    let _ = run_sync(root);
    let output = run_doctor(root);
    assert!(output.status.success(), "doctor failed: {:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("no issues found"));
}

#[test]
fn doctor_reports_orphan_fragment() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[
            ("nav.html", "<nav>Nav</nav>"),
            ("orphan.html", "<p>nobody uses me</p>"),
        ],
        &[("a.html", "<!-- fragment:nav -->old<!-- /fragment:nav -->")],
    );

    let output = run_doctor(root);
    assert!(
        !output.status.success(),
        "doctor should exit nonzero on issues"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("orphan fragment"), "stdout: {stdout}");
    assert!(stdout.contains("orphan.html"));
}

#[test]
fn doctor_reports_orphan_marker() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Page references "missing" but no fragments/missing.html exists.
    setup_site(
        root,
        &[("nav.html", "<nav>Nav</nav>")],
        &[(
            "a.html",
            "<!-- fragment:missing -->stale<!-- /fragment:missing -->",
        )],
    );

    let output = run_doctor(root);
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("orphan marker"), "stdout: {stdout}");
    assert!(stdout.contains("missing"));
}

#[test]
fn doctor_reports_unpaired_marker() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    setup_site(
        root,
        &[("nav.html", "<nav>Nav</nav>")],
        &[("a.html", "<!-- fragment:nav -->\nno close marker\n")],
    );

    let output = run_doctor(root);
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("unpaired open"), "stdout: {stdout}");
}
