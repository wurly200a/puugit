use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

pub struct CloneOptions {
    pub url: String,
    pub local_path: PathBuf,
    pub timeout_secs: u64,
}

#[derive(Debug)]
pub enum CloneResult {
    Success,
    Timeout,
    Failed(String),
}

pub fn clone_repo(options: CloneOptions) -> Receiver<CloneResult> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        if let Some(parent) = options.local_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                let _ = tx.send(CloneResult::Failed(format!("mkdir failed: {e}")));
                return;
            }
        }

        let local_path_str = options.local_path.to_string_lossy().into_owned();
        let result = std::process::Command::new("git")
            .args(["clone", &options.url, &local_path_str])
            .output();

        match result {
            Ok(output) if output.status.success() => {
                let _ = tx.send(CloneResult::Success);
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let _ = tx.send(CloneResult::Failed(if stderr.is_empty() {
                    format!("git clone exited with {}", output.status)
                } else {
                    stderr
                }));
            }
            Err(e) => {
                let _ = tx.send(CloneResult::Failed(format!("failed to run git: {e}")));
            }
        }
    });

    // Wrap the receiver so the GUI can poll with timeout
    let (timeout_tx, timeout_rx) = mpsc::channel();
    let timeout = Duration::from_secs(options.timeout_secs);
    std::thread::spawn(move || {
        let deadline = Instant::now() + timeout;
        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(result) => {
                    let _ = timeout_tx.send(result);
                    return;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if Instant::now() >= deadline {
                        let _ = timeout_tx.send(CloneResult::Timeout);
                        return;
                    }
                }
            }
        }
    });

    timeout_rx
}
