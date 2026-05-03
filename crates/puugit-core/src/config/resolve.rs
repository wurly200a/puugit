use std::path::{Path, PathBuf};

use super::repos::Account;

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

/// Replaces the host in a `git@host:...` URL with the account's ssh_host_alias when set.
pub fn resolve_clone_url(url: &str, account_name: &str, accounts: &[Account]) -> String {
    let alias = accounts
        .iter()
        .find(|a| a.name == account_name)
        .and_then(|a| a.ssh_host_alias.as_deref());

    let Some(alias) = alias else {
        return url.to_string();
    };

    // Match "git@<host>:<path>" and replace <host> with alias
    if let Some(rest) = url.strip_prefix("git@") {
        if let Some(colon_pos) = rest.find(':') {
            return format!("git@{}:{}", alias, &rest[colon_pos + 1..]);
        }
    }
    url.to_string()
}

pub fn resolve_local_path(repo_name: &str, tree_name: &str, base_clone_dir: &Path) -> PathBuf {
    base_clone_dir.join(tree_name).join(repo_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn expand_tilde_replaces_home() {
        let result = expand_tilde("~/foo/bar");
        assert!(
            result.to_string_lossy().contains("foo/bar")
                || result.to_string_lossy().contains("foo\\bar")
        );
        assert!(!result.to_string_lossy().starts_with('~'));
    }

    #[test]
    fn expand_tilde_passthrough() {
        let p = "/absolute/path";
        assert_eq!(expand_tilde(p), PathBuf::from(p));
    }

    #[test]
    fn resolve_local_path_default() {
        let result = resolve_local_path("myrepo", "group", Path::new("/base"));
        assert_eq!(result, PathBuf::from("/base/group/myrepo"));
    }
}
