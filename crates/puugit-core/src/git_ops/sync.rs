use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

pub struct SyncOptions {
    pub local_path: PathBuf,
    /// SSH-alias-resolved URL for the config repo (used only for initial clone)
    pub config_repo_url: String,
}

#[derive(Debug)]
pub enum SyncResult {
    Success(String),
    Failed(String),
}

fn git_in(path: &PathBuf, args: &[&str]) -> Result<std::process::Output, String> {
    std::process::Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .map_err(|e| format!("failed to run git: {e}"))
}

fn stderr_of(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

/// Commits repos.toml if changed, then pushes.
pub fn save_config(options: SyncOptions) -> Receiver<SyncResult> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let path = &options.local_path;

        match git_in(path, &["add", "repos.toml"]) {
            Err(e) => {
                tx.send(SyncResult::Failed(e)).ok();
                return;
            }
            Ok(o) if !o.status.success() => {
                tx.send(SyncResult::Failed(format!("git add: {}", stderr_of(&o))))
                    .ok();
                return;
            }
            _ => {}
        }

        // exit 0 = nothing staged, exit 1 = staged changes exist
        let has_staged = match git_in(path, &["diff", "--cached", "--quiet"]) {
            Ok(o) => !o.status.success(),
            Err(e) => {
                tx.send(SyncResult::Failed(e)).ok();
                return;
            }
        };

        if has_staged {
            match git_in(path, &["commit", "-m", "Update repos.toml [puugit]"]) {
                Err(e) => {
                    tx.send(SyncResult::Failed(e)).ok();
                    return;
                }
                Ok(o) if !o.status.success() => {
                    tx.send(SyncResult::Failed(format!("git commit: {}", stderr_of(&o))))
                        .ok();
                    return;
                }
                _ => {}
            }
        }

        match git_in(path, &["push"]) {
            Err(e) => {
                tx.send(SyncResult::Failed(e)).ok();
            }
            Ok(o) if !o.status.success() => {
                tx.send(SyncResult::Failed(format!("git push: {}", stderr_of(&o))))
                    .ok();
            }
            Ok(_) => {
                let msg = if has_staged {
                    "Saved: committed and pushed"
                } else {
                    "Saved: nothing to commit, pushed"
                };
                tx.send(SyncResult::Success(msg.to_string())).ok();
            }
        }
    });
    rx
}

/// Clones config repo if not present, otherwise pulls with rebase. Returns Success so the GUI can reload repos.toml.
pub fn update_config(options: SyncOptions) -> Receiver<SyncResult> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let path = &options.local_path;

        if !path.join(".git").exists() {
            if let Some(parent) = path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    tx.send(SyncResult::Failed(format!("mkdir: {e}"))).ok();
                    return;
                }
            }
            let output = std::process::Command::new("git")
                .args(["clone", &options.config_repo_url, &path.to_string_lossy()])
                .output();
            match output {
                Err(e) => {
                    tx.send(SyncResult::Failed(format!("git clone: {e}"))).ok();
                }
                Ok(o) if !o.status.success() => {
                    tx.send(SyncResult::Failed(format!(
                        "git clone: {}",
                        String::from_utf8_lossy(&o.stderr).trim()
                    )))
                    .ok();
                }
                Ok(_) => {
                    tx.send(SyncResult::Success(
                        "Cloned config repo and loaded repos.toml".to_string(),
                    ))
                    .ok();
                }
            }
        } else {
            match git_in(path, &["pull", "--rebase"]) {
                Err(e) => {
                    tx.send(SyncResult::Failed(e)).ok();
                }
                Ok(o) if !o.status.success() => {
                    tx.send(SyncResult::Failed(format!(
                        "git pull --rebase: {}",
                        stderr_of(&o)
                    )))
                    .ok();
                }
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    let msg = if stdout.is_empty() {
                        "Updated".to_string()
                    } else {
                        stdout
                    };
                    tx.send(SyncResult::Success(msg)).ok();
                }
            }
        }
    });
    rx
}
