use crate::config::Config;
use anyhow::Result;
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// The tag name for each candidate (used for raw-text matching).
struct SharedBlock {
    name: String,
    tag: String,
    content: String, // scraper's html() output (for the fragment file)
}

/// Find the raw source span of a top-level `<tag ...>...</tag>` element
/// by scanning the source text. Returns (start, end) byte offsets.
/// Uses a simple depth counter to handle nested same-name tags.
fn find_tag_span(src: &str, tag: &str) -> Option<(usize, usize)> {
    let open_prefix = format!("<{}", tag);
    let close_tag = format!("</{}>", tag);

    let start = src.find(&open_prefix)?;
    // Verify the char after `<tag` is whitespace, `>`, or `/` (not another tag name)
    let after = src.as_bytes().get(start + open_prefix.len())?;
    if !matches!(after, b' ' | b'>' | b'/' | b'\n' | b'\r' | b'\t') {
        return None;
    }

    let mut depth = 0i32;
    let haystack = &src[start..];

    // Walk through the string tracking open/close tags of the same name
    let mut idx = 0;
    while idx < haystack.len() {
        if haystack[idx..].starts_with(&open_prefix) {
            let after_idx = idx + open_prefix.len();
            if after_idx < haystack.len() {
                let ch = haystack.as_bytes()[after_idx];
                if matches!(ch, b' ' | b'>' | b'/' | b'\n' | b'\r' | b'\t') {
                    depth += 1;
                }
            }
            idx += open_prefix.len();
        } else if haystack[idx..].starts_with(&close_tag) {
            depth -= 1;
            if depth == 0 {
                let end = start + idx + close_tag.len();
                return Some((start, end));
            }
            idx += close_tag.len();
        } else {
            // Advance by one UTF-8 character (not one byte) to avoid
            // landing inside a multi-byte character like 'ä' or 'ö'.
            idx += haystack[idx..].chars().next().map_or(1, |c| c.len_utf8());
        }
    }
    None
}

fn collect_html_files(root: &Path, fragments_dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| {
            let p = e.path();
            !p.starts_with(fragments_dir)
                && !p.starts_with(&root.join("_assets"))
                && !p.starts_with(&root.join("css"))
                && !p.starts_with(&root.join("fonts"))
        })
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .map(|x| x == "html")
                    .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect()
}

/// Scan HTML files in a site directory, detect shared DOM blocks,
/// extract them to _fragments/*.html, and insert marker comments.
pub fn extract_fragments(root: &Path, config: &Config) -> Result<usize> {
    let fragments_dir = root.join(&config.fragments_dir);

    let html_files = collect_html_files(root, &fragments_dir);

    if html_files.len() < 2 {
        println!("  Less than 2 HTML pages, skipping extraction.");
        return Ok(0);
    }

    // Read all page contents
    let pages: Vec<_> = html_files
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok().map(|c| (p.clone(), c)))
        .collect();

    // Candidate selectors: (fragment name, CSS selector, tag name)
    let candidates: &[(&str, &str, &str)] = &[
        ("nav", "nav", "nav"),
        ("footer", "footer", "footer"),
        ("header", "header", "header"),
        ("navbar", ".navbar", "div"),
        ("site-header", ".site-header", "div"),
        ("site-footer", ".site-footer", "div"),
    ];

    let mut shared_blocks: Vec<SharedBlock> = Vec::new();

    for (name, sel_str, tag_name) in candidates {
        let sel = match Selector::parse(sel_str) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Count how many pages each variant of outer HTML appears in
        let mut html_to_count: HashMap<String, usize> = HashMap::new();

        for (_path, content) in &pages {
            let doc = Html::parse_document(content);
            let mut seen: HashSet<String> = HashSet::new();
            for el in doc.select(&sel) {
                let outer = el.html();
                if seen.insert(outer.clone()) {
                    *html_to_count.entry(outer).or_insert(0) += 1;
                }
            }
        }

        // Take the variant appearing in the most pages (must be 2+)
        if let Some((content, count)) = html_to_count.into_iter().max_by_key(|(_, v)| *v) {
            if count >= 2 {
                shared_blocks.push(SharedBlock {
                    name: name.to_string(),
                    tag: tag_name.to_string(),
                    content,
                });
            }
        }
    }

    if shared_blocks.is_empty() {
        println!("  No shared blocks detected.");
        return Ok(0);
    }

    // Create _fragments/ directory
    fs::create_dir_all(&fragments_dir)?;

    // Write fragment files (using scraper's normalized HTML)
    for block in &shared_blocks {
        let frag_path = fragments_dir.join(format!("{}.html", block.name));
        fs::write(&frag_path, &block.content)?;
        println!(
            "  Extracted: {}/{}.html",
            config.fragments_dir, block.name
        );
    }

    // Insert markers into pages using raw source tag matching.
    // Scraper reorders attributes, so we cannot use its html() output to find
    // the element in the source. Instead, we parse with scraper to confirm the
    // page has the shared block, then use tag-level search on the raw text.
    let prefix = &config.marker_prefix;
    let mut modified_count = 0;

    for (path, content) in &pages {
        let doc = Html::parse_document(content);
        let mut modified = content.clone();
        let mut changed = false;

        for block in &shared_blocks {
            let open_marker = format!("<!-- {}:{} -->", prefix, block.name);
            let close_marker = format!("<!-- /{}:{} -->", prefix, block.name);

            // Skip if markers already present
            if modified.contains(&open_marker) {
                continue;
            }

            // Check with scraper that this page has the shared block
            let sel = match Selector::parse(&format!(
                "{}",
                if block.tag == "div" {
                    // For class-based selectors, use the original selector
                    match block.name.as_str() {
                        "navbar" => ".navbar",
                        "site-header" => ".site-header",
                        "site-footer" => ".site-footer",
                        _ => &block.tag,
                    }
                } else {
                    &block.tag
                }
            )) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let has_block = doc.select(&sel).any(|el| el.html() == block.content);
            if !has_block {
                continue;
            }

            // Find the element in the raw source using tag matching
            if let Some((start, end)) = find_tag_span(&modified, &block.tag) {
                let raw_block = &modified[start..end];
                let replacement = format!(
                    "{}\n{}\n{}",
                    open_marker, raw_block, close_marker
                );
                modified = format!("{}{}{}", &modified[..start], replacement, &modified[end..]);
                changed = true;
            }
        }

        if changed {
            fs::write(path, &modified)?;
            modified_count += 1;
        }
    }

    println!(
        "  {} fragment(s) extracted, {} page(s) marked.",
        shared_blocks.len(),
        modified_count
    );

    Ok(modified_count)
}
