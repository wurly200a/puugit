# CLAUDE.md - puugit Project Guide

## Project Overview

**puugit** (puu = Finnish for "tree") is a GUI tool for managing multiple
Git repositories across multiple machines and accounts.
Built with Rust + egui (native GUI, no WebView).

## Repository Structure

```
puugit/
„¥„Ÿ„Ÿ Cargo.toml                  # Workspace root
„¤„Ÿ„Ÿ crates/
    „¥„Ÿ„Ÿ puugit-core/            # Core logic (no GUI dependency)
    „    „¤„Ÿ„Ÿ src/
    „        „¥„Ÿ„Ÿ config/
    „        „    „¥„Ÿ„Ÿ local.rs    # LocalConfig (local.toml)
    „        „    „¥„Ÿ„Ÿ repos.rs    # ReposConfig (repos.toml)
    „        „    „¤„Ÿ„Ÿ resolve.rs  # Path/URL resolution logic
    „        „¥„Ÿ„Ÿ git_ops/
    „        „    „¥„Ÿ„Ÿ clone.rs    # Clone via git command (not git2-rs)
    „        „    „¤„Ÿ„Ÿ remove.rs   # Safe remove with pre-checks
    „        „¥„Ÿ„Ÿ repo_status.rs  # Repo status via git2-rs
    „        „¤„Ÿ„Ÿ ssh_config.rs   # ~/.ssh/config parser
    „¤„Ÿ„Ÿ puugit-gui/             # egui frontend
        „¤„Ÿ„Ÿ src/
            „¥„Ÿ„Ÿ main.rs
            „¥„Ÿ„Ÿ app.rs               # Main app state and update loop
            „¥„Ÿ„Ÿ tree_view.rs         # Tree rendering
            „¥„Ÿ„Ÿ dialog.rs            # Clone/remove dialogs
            „¥„Ÿ„Ÿ add_repo_dialog.rs   # Add repository dialog
            „¥„Ÿ„Ÿ account_view.rs      # Account management window
            „¤„Ÿ„Ÿ subscription_view.rs # Subscription management window
```

## Architecture

### Config Files

**`~/.config/puugit/local.toml`** (machine-specific, never synced)

```toml
machine_id = "my-machine"

[account_keys]
personal = "github-<personal-account>"  # ssh Host alias in ~/.ssh/config
work     = "github-<work-account>"

[[subscriptions]]
name = "private"
config_repo = "git@github.com:<user>/puugit-private.git"
account = "personal"
local_path = "~/.config/puugit/subscriptions/private"
base_clone_dir = "/path/to/your/projects/private"
```

**`~/.config/puugit/subscriptions/<name>/repos.toml`** (synced via Git)

```toml
[[accounts]]
name = "personal"
host = "github.com"
username = "<personal-account>"

[[tree]]
name = "mygroup"

[[tree.children]]
name = "myrepo"
url = "git@github.com:<user>/myrepo.git"
account = "personal"
```

### Key Design Decisions

- **git operations**: clone/remove use `std::process::Command` (git CLI),
  NOT git2-rs. Reason: git2-rs vendored libgit2 cannot handle OpenSSH
  format keys on Windows.
- **repo status**: uses git2-rs (fast, no shell overhead).
- **SSH auth**: resolved via `~/.ssh/config` Host aliases.
  `account_keys` in local.toml maps account name to ssh Host alias.
  URL host is replaced with the alias before passing to git CLI.
- **local_path**: deliberately removed. All repos clone to
  `base_clone_dir/<tree_name>/<repo_name>` automatically.
- **Subscriptions**: each subscription has its own `base_clone_dir`
  to avoid tree name conflicts across subscriptions.

### URL Resolution

```
repos.toml url:  git@github.com:<user>/myrepo.git
account:         "personal"
account_keys:    personal = "github-<personal-account>"
resolved url:    git@github-<personal-account>:<user>/myrepo.git
```

## Development Rules

- Source code comments and commit messages: **English only**
- Currently in early development phase
- Direct commit/push to `main` is OK for now
- After initial phase: PRs required against `main`
  - PR description: English first, then Japanese
  - Run `cargo fmt --check` before `gh pr create`

## Build & Run

```bash
cargo build                    # build all crates
cargo run -p puugit-gui        # run the GUI
cargo test -p puugit-core      # run core tests
cargo fmt --check              # check formatting before PR
```

## Platform Notes

- Developed on both Windows and Linux/Ubuntu
- Both must work. Path handling uses `dirs` crate + `expand_tilde()`.
- Windows note: `$HOME` and `%USERPROFILE%` may differ depending on
  the environment. git CLI respects `$HOME`; Windows OpenSSH respects
  `%USERPROFILE%`. SSH auth goes through git CLI to avoid this issue.
