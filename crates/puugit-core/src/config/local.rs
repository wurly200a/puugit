use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use std::env;

use crate::error::{Error, Result};

/// Resolves the base config directory in priority order:
/// 1. $XDG_CONFIG_HOME
/// 2. $HOME/.config  (respects a custom HOME even on Windows)
/// 3. dirs::config_dir() fallback (AppData\Roaming on Windows)
fn config_base_dir() -> Option<PathBuf> {
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg));
    }
    if let Ok(home) = env::var("HOME") {
        return Some(PathBuf::from(home).join(".config"));
    }
    dirs::config_dir()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subscription {
    pub name: String,
    pub config_repo: String,
    /// SSH host alias used to clone config_repo (e.g. "github-wurly200a")
    pub config_account: String,
    pub local_path: String,
    /// Clone destination root for repos in this subscription
    pub base_clone_dir: String,
    /// Maps logical account label → SSH host alias (e.g. "private-wurly200a" → "github-wurly200a")
    #[serde(default)]
    pub account_map: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalConfig {
    pub machine_id: String,
    #[serde(default)]
    pub subscriptions: Vec<Subscription>,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            machine_id: String::new(),
            subscriptions: Vec::new(),
        }
    }
}

impl LocalConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            let config = Self::default();
            config.save(path)?;
            return Ok(config);
        }
        let text = fs::read_to_string(path)?;
        Ok(toml::from_str(&text)?)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)?;
        fs::write(path, text)?;
        Ok(())
    }

    pub fn default_path() -> Result<PathBuf> {
        let base = config_base_dir().ok_or(Error::ConfigDirNotFound)?;
        Ok(base.join("puugit").join("local.toml"))
    }
}
