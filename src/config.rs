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
    /// Directory containing target files (relative to root). Defaults to "."
    pub target_dir: String,
    /// Top-level directories under `target_dir` to skip when scanning.
    /// Match is by path prefix. Empty by default — config over convention.
    /// Format-specific layers (e.g. pagekit) re-add common defaults like
    /// `css`, `fonts`, `_assets` for their domain.
    pub exclude_dirs: Vec<String>,
    /// Maximum directory depth to scan from `target_dir`. Files deeper
    /// than this are invisible to fragments.
    pub max_depth: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            marker_prefix: "fragment".to_string(),
            // Default `_fragments` (with underscore prefix) so the folder
            // is excluded from deploy by static-site hosts that treat
            // underscore-prefixed dirs as infrastructure (CF Pages,
            // Eleventy, Jekyll, etc.). This convention is the dominant
            // pattern across consumers.
            fragments_dir: "_fragments".to_string(),
            target_dir: ".".to_string(),
            // Format-agnostic primitive: no built-in defaults. Each
            // consumer declares the dirs they want skipped. Website-shaped
            // defaults (`css`, `fonts`, `_assets`, `dist`, `build`, etc.)
            // belong in pagekit's config layer or per-project fragments.toml.
            exclude_dirs: Vec::new(),
            max_depth: 5,
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
