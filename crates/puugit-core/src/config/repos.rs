use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Account {
    pub name: String,
    pub host: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TreeNode {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReposConfig {
    #[serde(default)]
    pub accounts: Vec<Account>,
    #[serde(default)]
    pub tree: Vec<TreeNode>,
}

impl Default for ReposConfig {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            tree: Vec::new(),
        }
    }
}

impl ReposConfig {
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

    pub fn subscription_path(subscription_local_path: &str) -> PathBuf {
        PathBuf::from(subscription_local_path).join("repos.toml")
    }

    pub fn update_repo(
        &mut self,
        old_tree: &str,
        repo_name: &str,
        new_url: String,
        new_account: String,
        new_tree: String,
    ) {
        let mut node: Option<TreeNode> = None;
        for tree in &mut self.tree {
            if tree.name == old_tree {
                if let Some(pos) = tree.children.iter().position(|c| c.name == repo_name) {
                    node = Some(tree.children.remove(pos));
                    break;
                }
            }
        }
        let Some(mut n) = node else {
            return;
        };
        n.url = Some(new_url);
        n.account = if new_account.is_empty() {
            None
        } else {
            Some(new_account)
        };
        if let Some(tree) = self.tree.iter_mut().find(|t| t.name == new_tree) {
            tree.children.push(n);
        } else {
            self.tree.push(TreeNode {
                name: new_tree,
                url: None,
                account: None,
                children: vec![n],
            });
        }
        self.cleanup_empty_trees();
    }

    pub fn remove_repo(&mut self, tree_name: &str, repo_name: &str) {
        for tree in &mut self.tree {
            if tree.name == tree_name {
                tree.children.retain(|c| c.name != repo_name);
                break;
            }
        }
        self.cleanup_empty_trees();
    }

    fn cleanup_empty_trees(&mut self) {
        self.tree.retain(|t| !t.children.is_empty());
    }
}
