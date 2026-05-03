pub mod clone;
pub mod remove;

pub use clone::{clone_repo, CloneOptions, CloneResult};
pub use remove::{check_before_remove, remove_repo, RemoveCheckResult, RemoveWarning};
