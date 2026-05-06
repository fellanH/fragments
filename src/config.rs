use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const CONFIG_FILE: &str = "fragments.toml";

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub marker_prefix: String,
    pub fragments_dir: String,
    /// Directory containing target HTML files (relative to root). Defaults to "."
    pub target_dir: String,
    /// Top-level directories under `target_dir` to skip when scanning for
    /// HTML pages. Match is by path prefix. Defaults to common asset and
    /// tooling folders; extend for project-specific layouts (build, dist,
    /// public, etc.).
    pub exclude_dirs: Vec<String>,
    /// Maximum directory depth to scan from `target_dir` for HTML files.
    /// Sites organized deeper than this are silently invisible — raise it
    /// if your tree is deeper than the default.
    pub max_depth: usize,
    /// Extract subcommand configuration.
    pub extract: ExtractConfig,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ExtractConfig {
    /// Custom candidate selectors for `fragments extract`. User entries
    /// are APPENDED to the six built-in candidates (nav, footer, header,
    /// .navbar, .site-header, .site-footer) — adding one doesn't remove
    /// the others.
    pub candidates: Vec<ExtractCandidate>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExtractCandidate {
    /// Fragment file basename produced by extract (`<name>.html`).
    pub name: String,
    /// CSS selector used to locate the element in the parsed DOM.
    pub selector: String,
    /// HTML tag name of the element. Used to walk the raw source when
    /// inserting marker pairs (scraper normalizes attributes, so we can't
    /// match by parsed `.html()` output against the source string).
    pub tag: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            marker_prefix: "fragment".to_string(),
            fragments_dir: "fragments".to_string(),
            target_dir: ".".to_string(),
            exclude_dirs: vec![
                "node_modules".to_string(),
                "tools".to_string(),
                "css".to_string(),
                "fonts".to_string(),
                "_assets".to_string(),
            ],
            max_depth: 5,
            extract: ExtractConfig::default(),
        }
    }
}

impl Config {
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join(CONFIG_FILE);
        if path.exists() {
            let text = fs::read_to_string(&path)?;
            Ok(toml::from_str(&text)?)
        } else {
            Ok(Self::default())
        }
    }
}
