pub mod status;

pub use status::{RepoStatus, UnpushedBranch};

use std::path::Path;

use git2::{BranchType, Repository, StatusOptions};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoStatusError {
    #[error("Not a git repository: {0}")]
    NotARepo(std::path::PathBuf),
    #[error("git2 error: {0}")]
    Git2Error(#[from] git2::Error),
}

pub fn get_repo_status(path: &Path) -> Result<RepoStatus, RepoStatusError> {
    let repo = Repository::open(path).map_err(|_| RepoStatusError::NotARepo(path.to_path_buf()))?;

    // --- file status ---
    let (has_unstaged_changes, has_staged_changes, has_untracked_files) = {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true).recurse_untracked_dirs(true);
        let statuses = repo.statuses(Some(&mut opts))?;

        let mut unstaged = false;
        let mut staged = false;
        let mut untracked = false;

        for entry in statuses.iter() {
            let s = entry.status();
            if s.intersects(
                git2::Status::WT_MODIFIED
                    | git2::Status::WT_DELETED
                    | git2::Status::WT_RENAMED
                    | git2::Status::WT_TYPECHANGE,
            ) {
                unstaged = true;
            }
            if s.intersects(
                git2::Status::INDEX_NEW
                    | git2::Status::INDEX_MODIFIED
                    | git2::Status::INDEX_DELETED
                    | git2::Status::INDEX_RENAMED
                    | git2::Status::INDEX_TYPECHANGE,
            ) {
                staged = true;
            }
            if s.contains(git2::Status::WT_NEW) {
                untracked = true;
            }
        }
        (unstaged, staged, untracked)
        // `statuses` dropped here, releasing borrow of `repo`
    };

    // --- unpushed branches ---
    let mut unpushed_branches = Vec::new();

    for branch_result in repo.branches(Some(BranchType::Local))? {
        let (branch, _) = branch_result?;
        let name = match branch.name()? {
            Some(n) => n.to_string(),
            None => continue,
        };

        let local_oid = match branch.get().target() {
            Some(oid) => oid,
            None => continue,
        };

        let upstream = match branch.upstream() {
            Ok(u) => u,
            Err(_) => continue, // no upstream → skip
        };

        let upstream_oid = match upstream.get().target() {
            Some(oid) => oid,
            None => continue,
        };

        let (ahead, _behind) = repo.graph_ahead_behind(local_oid, upstream_oid)?;
        if ahead > 0 {
            unpushed_branches.push(UnpushedBranch {
                name,
                commit_count: ahead,
            });
        }
    }

    // --- stash count ---
    let mut stash_count = 0usize;
    // stash_foreach requires &mut Repository; re-bind as mut after borrows are released
    let mut repo = repo;
    repo.stash_foreach(|_index, _msg, _oid| {
        stash_count += 1;
        true
    })?;

    Ok(RepoStatus {
        path: path.to_path_buf(),
        is_git_repo: true,
        has_unstaged_changes,
        has_staged_changes,
        has_untracked_files,
        unpushed_branches,
        stash_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_get_repo_status_valid_repo() {
        let path = match env::var("TEST_REPO_PATH") {
            Ok(p) => std::path::PathBuf::from(p),
            Err(_) => {
                eprintln!("TEST_REPO_PATH not set, skipping");
                return;
            }
        };

        let status = get_repo_status(&path).expect("should succeed");
        println!("RepoStatus for {:?}:", status.path);
        println!("  is_git_repo:          {}", status.is_git_repo);
        println!("  has_unstaged_changes: {}", status.has_unstaged_changes);
        println!("  has_staged_changes:   {}", status.has_staged_changes);
        println!("  has_untracked_files:  {}", status.has_untracked_files);
        println!("  stash_count:          {}", status.stash_count);
        println!("  unpushed_branches:");
        for b in &status.unpushed_branches {
            println!("    {} (+{})", b.name, b.commit_count);
        }
        assert!(status.is_git_repo);
    }

    #[test]
    fn test_get_repo_status_not_a_repo() {
        let dir = TempDir::new().unwrap();
        let result = get_repo_status(dir.path());
        assert!(
            matches!(result, Err(RepoStatusError::NotARepo(_))),
            "expected NotARepo, got {:?}",
            result
        );
    }

    #[test]
    fn test_get_repo_status_nonexistent() {
        let path = std::path::Path::new("/nonexistent/path/that/does/not/exist");
        let result = get_repo_status(path);
        assert!(result.is_err(), "expected error for nonexistent path");
    }
}
