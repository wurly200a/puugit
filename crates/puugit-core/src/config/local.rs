use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subscription {
    pub name: String,
    pub config_repo: String,
    pub local_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalConfig {
    pub machine_id: String,
    pub base_clone_dir: String,
    #[serde(default)]
    pub account_keys: HashMap<String, String>,
    #[serde(default)]
    pub subscriptions: Vec<Subscription>,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            machine_id: String::new(),
            base_clone_dir: String::new(),
            account_keys: HashMap::new(),
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
        let base = dirs::config_dir().ok_or(Error::ConfigDirNotFound)?;
        Ok(base.join("puugit").join("local.toml"))
    }
}
