use std::fmt;
use std::path::Path;

use crate::repo_status::{get_repo_status, RepoStatusError};

pub struct RemoveCheckResult {
    pub can_remove_safely: bool,
    pub warnings: Vec<RemoveWarning>,
}

#[derive(Debug)]
pub enum RemoveWarning {
    UnpushedBranches(Vec<String>),
    UncommittedChanges,
    UntrackedFiles,
    StashEntries(usize),
}

impl fmt::Display for RemoveWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnpushedBranches(branches) => {
                write!(f, "Unpushed branches: {}", branches.join(", "))
            }
            Self::UncommittedChanges => write!(f, "Uncommitted changes"),
            Self::UntrackedFiles => write!(f, "Untracked files"),
            Self::StashEntries(n) => write!(f, "Stash entries: {n}"),
        }
    }
}

pub fn check_before_remove(path: &Path) -> Result<RemoveCheckResult, RepoStatusError> {
    let status = get_repo_status(path)?;
    let mut warnings = Vec::new();

    if !status.unpushed_branches.is_empty() {
        let names = status
            .unpushed_branches
            .iter()
            .map(|b| b.name.clone())
            .collect();
        warnings.push(RemoveWarning::UnpushedBranches(names));
    }
    if status.has_unstaged_changes || status.has_staged_changes {
        warnings.push(RemoveWarning::UncommittedChanges);
    }
    if status.has_untracked_files {
        warnings.push(RemoveWarning::UntrackedFiles);
    }
    if status.stash_count > 0 {
        warnings.push(RemoveWarning::StashEntries(status.stash_count));
    }

    let can_remove_safely = warnings.is_empty();
    Ok(RemoveCheckResult {
        can_remove_safely,
        warnings,
    })
}

pub fn remove_repo(path: &Path) -> std::io::Result<()> {
    std::fs::remove_dir_all(path)
}
