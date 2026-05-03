pub mod config;
pub mod error;
pub mod git_ops;
pub mod repo_status;

pub use config::{LocalConfig, ReposConfig};
pub use error::{Error, Result};
pub use repo_status::{get_repo_status, RepoStatus, RepoStatusError, UnpushedBranch};

#[cfg(test)]
mod tests {
    use super::*;
    use config::local::Subscription;
    use config::repos::{Account, TreeNode};
    use tempfile::TempDir;

    fn temp_path(dir: &TempDir, name: &str) -> std::path::PathBuf {
        dir.path().join(name)
    }

    // ---- LocalConfig ----

    #[test]
    fn local_config_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = temp_path(&dir, "local.toml");

        let original = LocalConfig {
            machine_id: "win-main".to_string(),
            base_clone_dir: "D:/home/wurly/repos".to_string(),
            subscriptions: vec![
                Subscription {
                    name: "private".to_string(),
                    config_repo: "git@github.com:wurly/puugit-private.git".to_string(),
                    local_path: "~/.config/puugit/subscriptions/private".to_string(),
                },
                Subscription {
                    name: "work".to_string(),
                    config_repo: "git@github.com:wurly-work/puugit-work.git".to_string(),
                    local_path: "~/.config/puugit/subscriptions/work".to_string(),
                },
            ],
        };

        original.save(&path).unwrap();
        let loaded = LocalConfig::load(&path).unwrap();
        assert_eq!(original, loaded);
    }

    #[test]
    fn local_config_default_on_missing() {
        let dir = TempDir::new().unwrap();
        let path = temp_path(&dir, "nonexistent.toml");

        assert!(!path.exists());
        let config = LocalConfig::load(&path).unwrap();
        assert_eq!(config, LocalConfig::default());
        assert!(path.exists(), "ファイルが新規作成されていること");
    }

    // ---- ReposConfig ----

    #[test]
    fn repos_config_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = temp_path(&dir, "repos.toml");

        let original = ReposConfig {
            accounts: vec![
                Account {
                    name: "personal".to_string(),
                    host: "github.com".to_string(),
                    username: "wurly-personal".to_string(),
                    ssh_host_alias: Some("github-wurly200a".to_string()),
                },
                Account {
                    name: "work".to_string(),
                    host: "github.com".to_string(),
                    username: "wurly-work".to_string(),
                    ssh_host_alias: None,
                },
            ],
            tree: vec![TreeNode {
                name: "music".to_string(),
                url: None,
                account: None,
                children: vec![
                    TreeNode {
                        name: "xdx-rs".to_string(),
                        url: Some("git@github.com:wurly/xdx-rs.git".to_string()),
                        account: Some("personal".to_string()),
                        children: vec![],
                    },
                    TreeNode {
                        name: "some-synth".to_string(),
                        url: Some("git@github.com:wurly/some-synth.git".to_string()),
                        account: Some("personal".to_string()),
                        children: vec![],
                    },
                ],
            }],
        };

        original.save(&path).unwrap();
        let loaded = ReposConfig::load(&path).unwrap();
        assert_eq!(original, loaded);
    }

    #[test]
    fn repos_config_default_on_missing() {
        let dir = TempDir::new().unwrap();
        let path = temp_path(&dir, "nonexistent_repos.toml");

        assert!(!path.exists());
        let config = ReposConfig::load(&path).unwrap();
        assert_eq!(config, ReposConfig::default());
        assert!(path.exists(), "ファイルが新規作成されていること");
    }
}
