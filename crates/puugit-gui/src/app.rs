use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::account_view::AccountWindow;
use crate::dialog::{CloningState, DeleteState, Dialog, DialogAction};
use crate::subscription_view::SubscriptionWindow;
use crate::tree_view::{NodeAction, NodeKind, TreeNode};

pub struct SubscriptionTree {
    pub name: String,
    pub nodes: Vec<TreeNode>,
}

pub struct PuugitApp {
    subscriptions: Vec<SubscriptionTree>,
    selected_subscription: usize,
    error_message: Option<String>,
    dialog: Dialog,
    local_config: Option<puugit_core::config::LocalConfig>,
    local_config_path: Option<PathBuf>,
    account_window: AccountWindow,
    subscription_window: SubscriptionWindow,
}

impl PuugitApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        match load_local_config() {
            Ok((path, config)) => {
                let subscriptions = build_tree(&config);
                Self {
                    subscriptions,
                    selected_subscription: 0,
                    error_message: None,
                    dialog: Dialog::None,
                    local_config: Some(config),
                    local_config_path: Some(path),
                    account_window: AccountWindow::new(),
                    subscription_window: SubscriptionWindow::new(),
                }
            }
            Err(msg) => Self {
                subscriptions: vec![],
                selected_subscription: 0,
                error_message: Some(msg),
                dialog: Dialog::None,
                local_config: None,
                local_config_path: None,
                account_window: AccountWindow::new(),
                subscription_window: SubscriptionWindow::new(),
            },
        }
    }
}

impl eframe::App for PuugitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dialog_action = self.dialog.show(ctx);
        match dialog_action {
            DialogAction::None => {}
            DialogAction::CloneSucceeded { local_path } => {
                self.dialog = Dialog::None;
                refresh_all(&mut self.subscriptions, &local_path, true);
            }
            DialogAction::CloneDismissed => {
                self.dialog = Dialog::None;
            }
            DialogAction::DeleteConfirmed { local_path } => {
                self.dialog = Dialog::None;
                if let Err(e) = puugit_core::git_ops::remove_repo(&local_path) {
                    eprintln!("remove_repo failed: {e}");
                }
                refresh_all(&mut self.subscriptions, &local_path, false);
            }
            DialogAction::DeleteCancelled => {
                self.dialog = Dialog::None;
            }
        }

        let config_path = self.local_config_path.clone();
        let mut needs_rebuild = false;

        if let (Some(config), Some(path)) = (&mut self.local_config, config_path.clone()) {
            if self.account_window.show(ctx, config, &path) {
                needs_rebuild = true;
            }
        }
        if let (Some(config), Some(path)) = (&mut self.local_config, config_path) {
            if self.subscription_window.show(ctx, config, &path) {
                needs_rebuild = true;
            }
        }
        if needs_rebuild {
            if let Some(config) = &self.local_config {
                self.subscriptions = build_tree(config);
                if self.selected_subscription >= self.subscriptions.len() {
                    self.selected_subscription = 0;
                }
            }
        }

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("")
                    .selected_text(
                        self.subscriptions
                            .get(self.selected_subscription)
                            .map(|s| s.name.as_str())
                            .unwrap_or("(none)"),
                    )
                    .show_ui(ui, |ui| {
                        for (i, sub) in self.subscriptions.iter().enumerate() {
                            ui.selectable_value(&mut self.selected_subscription, i, &sub.name);
                        }
                    });
                if ui.button("Accounts").clicked() {
                    self.account_window.open = true;
                }
                if ui.button("Subscriptions").clicked() {
                    self.subscription_window.open = true;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(msg) = &self.error_message {
                ui.colored_label(egui::Color32::RED, msg);
                return;
            }

            let mut actions: Vec<NodeAction> = Vec::new();

            ui.set_enabled(!self.dialog.is_open());

            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(sub) = self.subscriptions.get_mut(self.selected_subscription) {
                    for node in &mut sub.nodes {
                        crate::tree_view::show_node(ui, node, &mut actions);
                    }
                }
            });

            if let Some(action) = actions.into_iter().next() {
                if !self.dialog.is_open() {
                    self.handle_action(action);
                }
            }
        });
    }
}

impl PuugitApp {
    fn handle_action(&mut self, action: NodeAction) {
        match action {
            NodeAction::Clone {
                url,
                local_path,
                repo_name,
            } => {
                let options = puugit_core::git_ops::CloneOptions {
                    url,
                    local_path: local_path.clone(),
                    timeout_secs: 60,
                };
                let receiver = puugit_core::git_ops::clone_repo(options);
                self.dialog = Dialog::Cloning(CloningState {
                    repo_name,
                    local_path,
                    receiver,
                    started_at: Instant::now(),
                    result: None,
                });
            }
            NodeAction::Remove {
                local_path,
                repo_name,
            } => {
                let warnings = match puugit_core::git_ops::check_before_remove(&local_path) {
                    Ok(r) => r.warnings,
                    Err(e) => {
                        eprintln!("check_before_remove failed: {e}");
                        vec![]
                    }
                };
                self.dialog = Dialog::ConfirmDelete(DeleteState {
                    repo_name,
                    local_path,
                    warnings,
                });
            }
        }
    }
}

fn refresh_all(subs: &mut Vec<SubscriptionTree>, target: &Path, cloned: bool) {
    for sub in subs.iter_mut() {
        refresh_node(&mut sub.nodes, target, cloned);
    }
}

fn refresh_node(nodes: &mut Vec<TreeNode>, target: &Path, cloned: bool) {
    for node in nodes.iter_mut() {
        match &mut node.kind {
            NodeKind::Repo {
                local_path,
                cloned: node_cloned,
                status,
                ..
            } => {
                if local_path.as_path() == target {
                    *node_cloned = cloned;
                    *status = if cloned {
                        puugit_core::repo_status::get_repo_status(local_path).ok()
                    } else {
                        None
                    };
                }
            }
            NodeKind::Folder => {
                refresh_node(&mut node.children, target, cloned);
            }
        }
    }
}

fn load_local_config() -> Result<(PathBuf, puugit_core::config::LocalConfig), String> {
    let path = puugit_core::config::LocalConfig::default_path()
        .map_err(|e| format!("Failed to resolve config path: {e}"))?;

    if !path.exists() {
        return Err(
            "No configuration found. Please create ~/.config/puugit/local.toml".to_string(),
        );
    }

    let config = puugit_core::config::LocalConfig::load(&path)
        .map_err(|e| format!("Failed to load local.toml: {e}"))?;

    Ok((path, config))
}

fn build_tree(local: &puugit_core::config::LocalConfig) -> Vec<SubscriptionTree> {
    use puugit_core::config::resolve;

    let mut result: Vec<SubscriptionTree> = Vec::new();

    for sub in &local.subscriptions {
        let base_clone_dir = resolve::expand_tilde(&sub.base_clone_dir);
        let sub_dir = resolve::expand_tilde(&sub.local_path);
        let repos_toml = sub_dir.join("repos.toml");

        if !repos_toml.exists() {
            eprintln!(
                "Warning: repos.toml not found for subscription '{}' at {}, skipping",
                sub.name,
                repos_toml.display()
            );
            continue;
        }

        let repos = match puugit_core::config::ReposConfig::load(&repos_toml) {
            Ok(r) => r,
            Err(e) => {
                eprintln!(
                    "Warning: failed to load repos.toml for subscription '{}': {e}, skipping",
                    sub.name
                );
                continue;
            }
        };

        let mut folder_nodes: Vec<TreeNode> = Vec::new();

        for tree_group in &repos.tree {
            let mut children: Vec<TreeNode> = Vec::new();

            for child in &tree_group.children {
                let repo_path =
                    resolve::resolve_local_path(&child.name, &tree_group.name, &base_clone_dir);

                let url = match &child.account {
                    Some(acc) => resolve::resolve_clone_url(
                        child.url.as_deref().unwrap_or(""),
                        acc,
                        &local.account_keys,
                        &repos.accounts,
                    ),
                    None => child.url.clone().unwrap_or_default(),
                };

                let cloned = repo_path.exists();
                let status = if cloned {
                    puugit_core::repo_status::get_repo_status(&repo_path).ok()
                } else {
                    None
                };

                children.push(TreeNode {
                    name: child.name.clone(),
                    kind: NodeKind::Repo {
                        url,
                        local_path: repo_path,
                        cloned,
                        status,
                    },
                    children: vec![],
                    expanded: false,
                });
            }

            folder_nodes.push(TreeNode {
                name: tree_group.name.clone(),
                kind: NodeKind::Folder,
                children,
                expanded: true,
            });
        }

        result.push(SubscriptionTree {
            name: sub.name.clone(),
            nodes: folder_nodes,
        });
    }

    result
}
