pub mod clone;
pub mod remove;
pub mod sync;

pub use clone::{clone_repo, CloneOptions, CloneResult};
pub use remove::{check_before_remove, remove_repo, RemoveCheckResult, RemoveWarning};
pub use sync::{save_config, update_config, SyncOptions, SyncResult};
