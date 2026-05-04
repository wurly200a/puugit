use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RepoStatus {
    pub path: PathBuf,
    pub is_git_repo: bool,
    pub has_unstaged_changes: bool,
    pub has_staged_changes: bool,
    pub has_untracked_files: bool,
    pub unpushed_branches: Vec<UnpushedBranch>,
    pub stash_count: usize,
    pub unstaged_files: Vec<String>,
    pub staged_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub has_remote: bool,
    pub last_fetch_time: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone)]
pub struct UnpushedBranch {
    pub name: String,
    pub commit_count: usize,
}
