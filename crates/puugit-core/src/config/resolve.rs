use std::path::{Path, PathBuf};

use super::repos::{Account, TreeNode};

/// Resolves home directory: $HOME env var first, then dirs::home_dir().
fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(dirs::home_dir)
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        return home_dir().unwrap_or_else(|| PathBuf::from("~"));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

pub fn resolve_local_path(child: &TreeNode, tree_name: &str, base_clone_dir: &Path) -> PathBuf {
    if let Some(path) = &child.local_path {
        expand_tilde(path)
    } else {
        base_clone_dir.join(tree_name).join(&child.name)
    }
}

pub fn resolve_clone_url(url: &str, account_name: &str, accounts: &[Account]) -> String {
    let account = match accounts.iter().find(|a| a.name == account_name) {
        Some(a) => a,
        None => return url.to_string(),
    };
    let alias = match &account.ssh_host_alias {
        Some(a) => a,
        None => return url.to_string(),
    };
    // Replace the host portion: "git@github.com:..." → "git@github-alias:..."
    url.replacen(&account.host, alias, 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::repos::{Account, TreeNode};
    use std::path::PathBuf;

    fn accounts() -> Vec<Account> {
        vec![Account {
            name: "personal".into(),
            host: "github.com".into(),
            username: "wurly200a".into(),
            ssh_host_alias: Some("github-wurly200a".into()),
        }]
    }

    #[test]
    fn expand_tilde_replaces_home() {
        let result = expand_tilde("~/foo/bar");
        assert!(result.to_string_lossy().contains("foo/bar") || result.to_string_lossy().contains("foo\\bar"));
        assert!(!result.to_string_lossy().starts_with('~'));
    }

    #[test]
    fn expand_tilde_passthrough() {
        let p = "/absolute/path";
        assert_eq!(expand_tilde(p), PathBuf::from(p));
    }

    #[test]
    fn resolve_local_path_explicit() {
        let child = TreeNode {
            name: "repo".into(),
            url: None,
            account: None,
            local_path: Some("/explicit/path".into()),
            children: vec![],
        };
        let result = resolve_local_path(&child, "group", Path::new("/base"));
        assert_eq!(result, PathBuf::from("/explicit/path"));
    }

    #[test]
    fn resolve_local_path_default() {
        let child = TreeNode {
            name: "myrepo".into(),
            url: None,
            account: None,
            local_path: None,
            children: vec![],
        };
        let result = resolve_local_path(&child, "group", Path::new("/base"));
        assert_eq!(result, PathBuf::from("/base/group/myrepo"));
    }

    #[test]
    fn resolve_clone_url_replaces_host() {
        let url = "git@github.com:wurly200a/xdx-rs.git";
        let result = resolve_clone_url(url, "personal", &accounts());
        assert_eq!(result, "git@github-wurly200a:wurly200a/xdx-rs.git");
    }

    #[test]
    fn resolve_clone_url_unknown_account() {
        let url = "git@github.com:wurly200a/xdx-rs.git";
        let result = resolve_clone_url(url, "unknown", &accounts());
        assert_eq!(result, url);
    }
}
