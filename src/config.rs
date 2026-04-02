use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::Path;

const CONFIG_FILE: &str = "fragments.toml";

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub marker_prefix: String,
    pub fragments_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            marker_prefix: "fragment".to_string(),
            fragments_dir: "fragments".to_string(),
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
