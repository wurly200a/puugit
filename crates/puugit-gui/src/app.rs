use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::account_view::AccountWindow;
use crate::add_repo_dialog::AddRepoDialog;
use crate::dialog::{CloningState, DeleteState, Dialog, DialogAction};
use crate::side_panel::{SidePanel, SidePanelAction};
use crate::subscription_view::{SubscriptionWindow, SubscriptionWindowResult};
use crate::tree_view::{NodeAction, NodeKind, TreeNode};

enum SyncKind {
    Save,
    Update,
}

struct SyncOp {
    kind: SyncKind,
    target_subscription: usize,
    receiver: std::sync::mpsc::Receiver<puugit_core::git_ops::SyncResult>,
}

pub struct SubscriptionTree {
    pub name: String,
    pub nodes: Vec<TreeNode>,
    pub repos: puugit_core::config::ReposConfig,
    pub repos_toml_path: PathBuf,
}

pub struct PuugitApp {
    subscriptions: Vec<SubscriptionTree>,
    selected_subscription: usize,
    selected_repo_id: Option<String>,
    error_message: Option<String>,
    dialog: Dialog,
    local_config: Option<puugit_core::config::LocalConfig>,
    local_config_path: Option<PathBuf>,
    account_window: AccountWindow,
    subscription_window: SubscriptionWindow,
    add_repo_dialog: AddRepoDialog,
    side_panel: SidePanel,
    sync_op: Option<SyncOp>,
    sync_message: Option<(bool, String)>,
}

impl PuugitApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles = [
            (egui::TextStyle::Body, egui::FontId::proportional(14.0)),
            (egui::TextStyle::Button, egui::FontId::proportional(14.0)),
            (egui::TextStyle::Heading, egui::FontId::proportional(16.0)),
            (egui::TextStyle::Monospace, egui::FontId::monospace(13.0)),
            (egui::TextStyle::Small, egui::FontId::proportional(12.0)),
        ]
        .into();
        cc.egui_ctx.set_style(style);

        match load_local_config() {
            Ok((path, config)) => {
                let subscriptions = build_tree(&config);
                Self {
                    subscriptions,
                    selected_subscription: 0,
                    selected_repo_id: None,
                    error_message: None,
                    dialog: Dialog::None,
                    local_config: Some(config),
                    local_config_path: Some(path),
                    account_window: AccountWindow::new(),
                    subscription_window: SubscriptionWindow::new(),
                    add_repo_dialog: AddRepoDialog::new(),
                    side_panel: SidePanel::new(),
                    sync_op: None,
                    sync_message: None,
                }
            }
            Err(msg) => Self {
                subscriptions: vec![],
                selected_subscription: 0,
                selected_repo_id: None,
                error_message: Some(msg),
                dialog: Dialog::None,
                local_config: None,
                local_config_path: None,
                account_window: AccountWindow::new(),
                subscription_window: SubscriptionWindow::new(),
                add_repo_dialog: AddRepoDialog::new(),
                side_panel: SidePanel::new(),
                sync_op: None,
                sync_message: None,
            },
        }
    }
}

impl eframe::App for PuugitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.0);

        // Poll running sync operation
        let sync_result = if let Some(op) = &self.sync_op {
            match op.receiver.try_recv() {
                Ok(result) => Some(result),
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint();
                    None
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    Some(puugit_core::git_ops::SyncResult::Failed(
                        "sync thread disconnected unexpectedly".to_string(),
                    ))
                }
            }
        } else {
            None
        };
        if let Some(result) = sync_result {
            let op = self.sync_op.take().unwrap();
            let (is_error, message) = match result {
                puugit_core::git_ops::SyncResult::Success(msg) => (false, msg),
                puugit_core::git_ops::SyncResult::Failed(msg) => (true, msg),
            };
            if !is_error && matches!(op.kind, SyncKind::Update) {
                if let Some(config) = &self.local_config {
                    let idx = op.target_subscription;
                    if let Some(new_tree) = load_subscription_tree(config, idx) {
                        if idx < self.subscriptions.len() {
                            self.subscriptions[idx] = new_tree;
                        } else {
                            self.subscriptions.push(new_tree);
                        }
                    }
                }
                self.side_panel.selected_repo = None;
                self.selected_repo_id = None;
            }
            self.sync_message = Some((is_error, message));
        }

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
            let idx = self.selected_subscription;
            if self.account_window.show(ctx, config, idx, &path) {
                needs_rebuild = true;
            }
        }
        if let (Some(config), Some(path)) = (&mut self.local_config, config_path) {
            match self.subscription_window.show(ctx, config, &path) {
                SubscriptionWindowResult::None => {}
                SubscriptionWindowResult::Modified => {
                    needs_rebuild = true;
                }
                SubscriptionWindowResult::Added(sub) => {
                    needs_rebuild = true;
                    let new_idx = config.subscriptions.len().saturating_sub(1);
                    self.selected_subscription = new_idx;
                    if self.sync_op.is_none() {
                        use puugit_core::config::resolve;
                        let local_path = resolve::expand_tilde(&sub.local_path);
                        let config_repo_url = resolve::resolve_config_repo_url(
                            &sub.config_repo,
                            &sub.config_account,
                        );
                        let opts = puugit_core::git_ops::SyncOptions {
                            local_path,
                            config_repo_url,
                        };
                        self.sync_message = None;
                        self.sync_op = Some(SyncOp {
                            kind: SyncKind::Update,
                            target_subscription: new_idx,
                            receiver: puugit_core::git_ops::update_config(opts),
                        });
                    }
                }
            }
        }
        let account_names: Vec<String> = self
            .local_config
            .as_ref()
            .and_then(|c| c.subscriptions.get(self.selected_subscription))
            .map(|sub| {
                let mut names: Vec<String> = sub.account_map.keys().cloned().collect();
                names.sort();
                names
            })
            .unwrap_or_default();

        let selected = self.selected_subscription;
        let add_repo_dialog = &mut self.add_repo_dialog;
        let subscriptions = &mut self.subscriptions;
        if let Some(sub) = subscriptions.get_mut(selected) {
            if add_repo_dialog.show(ctx, &mut sub.repos, &sub.repos_toml_path, &account_names) {
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
        let tree_names: Vec<String> = self
            .subscriptions
            .get(self.selected_subscription)
            .map(|s| s.repos.tree.iter().map(|t| t.name.clone()).collect())
            .unwrap_or_default();

        let side_panel_action = self.side_panel.show(ctx, &account_names, &tree_names);

        match side_panel_action {
            SidePanelAction::None => {}
            SidePanelAction::SaveEdit {
                old_tree,
                repo_name,
                new_url,
                new_account,
                new_tree,
            } => {
                let idx = self.selected_subscription;
                if let Some(sub) = self.subscriptions.get_mut(idx) {
                    sub.repos
                        .update_repo(&old_tree, &repo_name, new_url, new_account, new_tree);
                    sub.repos.save(&sub.repos_toml_path).ok();
                }
                let new_sub = self
                    .local_config
                    .as_ref()
                    .and_then(|c| load_subscription_tree(c, idx));
                if let Some(ns) = new_sub {
                    if idx < self.subscriptions.len() {
                        self.subscriptions[idx] = ns;
                    }
                }
                self.side_panel.selected_repo = None;
                self.selected_repo_id = None;
            }
            SidePanelAction::Delete {
                tree_name,
                repo_name,
            } => {
                let idx = self.selected_subscription;
                if let Some(sub) = self.subscriptions.get_mut(idx) {
                    sub.repos.remove_repo(&tree_name, &repo_name);
                    sub.repos.save(&sub.repos_toml_path).ok();
                }
                let new_sub = self
                    .local_config
                    .as_ref()
                    .and_then(|c| load_subscription_tree(c, idx));
                if let Some(ns) = new_sub {
                    if idx < self.subscriptions.len() {
                        self.subscriptions[idx] = ns;
                    }
                }
                self.side_panel.selected_repo = None;
                self.selected_repo_id = None;
            }
        }

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Subscriptions").clicked() {
                    self.subscription_window.open = true;
                }
            });
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
                if ui.button("Account Map").clicked() {
                    self.account_window.open = true;
                }
                if ui.button("Add Repo").clicked() {
                    self.add_repo_dialog.open = true;
                }
            });
            ui.horizontal(|ui| {
                if ui.button("\u{1f504} Reload").clicked() {
                    if let Some(config) = &self.local_config {
                        let idx = self.selected_subscription;
                        if let Some(new_tree) = load_subscription_tree(config, idx) {
                            if idx < self.subscriptions.len() {
                                self.subscriptions[idx] = new_tree;
                            }
                        }
                    }
                    self.side_panel.selected_repo = None;
                    self.selected_repo_id = None;
                }

                let syncing = self.sync_op.is_some();
                let has_sub = self
                    .local_config
                    .as_ref()
                    .and_then(|c| c.subscriptions.get(self.selected_subscription))
                    .is_some();

                if ui
                    .add_enabled(!syncing && has_sub, egui::Button::new("Save"))
                    .clicked()
                {
                    if let Some(opts) = self.sync_options() {
                        self.sync_message = None;
                        self.sync_op = Some(SyncOp {
                            kind: SyncKind::Save,
                            target_subscription: self.selected_subscription,
                            receiver: puugit_core::git_ops::save_config(opts),
                        });
                    }
                }
                if ui
                    .add_enabled(!syncing && has_sub, egui::Button::new("Update & Load"))
                    .clicked()
                {
                    if let Some(opts) = self.sync_options() {
                        self.sync_message = None;
                        self.sync_op = Some(SyncOp {
                            kind: SyncKind::Update,
                            target_subscription: self.selected_subscription,
                            receiver: puugit_core::git_ops::update_config(opts),
                        });
                    }
                }
                if syncing {
                    ui.spinner();
                }
            });
            if let Some((is_error, msg)) = &self.sync_message {
                ui.horizontal(|ui| {
                    let color = if *is_error {
                        egui::Color32::RED
                    } else {
                        egui::Color32::from_rgb(0, 160, 0)
                    };
                    ui.colored_label(color, msg);
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(msg) = &self.error_message {
                ui.colored_label(egui::Color32::RED, msg);
                return;
            }

            let mut actions: Vec<NodeAction> = Vec::new();

            ui.set_enabled(!self.dialog.is_open());

            let sub_name = self
                .subscriptions
                .get(self.selected_subscription)
                .map(|s| s.name.clone())
                .unwrap_or_default();
            let selected_repo_id = self.selected_repo_id.clone();

            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(sub) = self.subscriptions.get_mut(self.selected_subscription) {
                    for node in &mut sub.nodes {
                        crate::tree_view::show_node(
                            ui,
                            node,
                            &mut actions,
                            &selected_repo_id,
                            &sub_name,
                        );
                    }
                }
            });

            for action in actions {
                match action {
                    NodeAction::Select {
                        name,
                        local_path,
                        repo_id,
                        cloned,
                        url,
                        account,
                        tree_name,
                    } => {
                        self.side_panel
                            .select(name, local_path, cloned, url, account, tree_name);
                        self.selected_repo_id = Some(repo_id);
                    }
                    other if !self.dialog.is_open() => {
                        self.handle_action(other);
                    }
                    _ => {}
                }
            }
        });
    }
}

impl PuugitApp {
    fn sync_options(&self) -> Option<puugit_core::git_ops::SyncOptions> {
        use puugit_core::config::resolve;
        let config = self.local_config.as_ref()?;
        let sub = config.subscriptions.get(self.selected_subscription)?;
        let local_path = resolve::expand_tilde(&sub.local_path);
        let config_repo_url =
            resolve::resolve_config_repo_url(&sub.config_repo, &sub.config_account);
        Some(puugit_core::git_ops::SyncOptions {
            local_path,
            config_repo_url,
        })
    }

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
            NodeAction::Select {
                name,
                local_path,
                repo_id,
                cloned,
                url,
                account,
                tree_name,
            } => {
                self.side_panel
                    .select(name, local_path, cloned, url, account, tree_name);
                self.selected_repo_id = Some(repo_id);
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

fn load_subscription_tree(
    local: &puugit_core::config::LocalConfig,
    idx: usize,
) -> Option<SubscriptionTree> {
    use puugit_core::config::resolve;

    let sub = local.subscriptions.get(idx)?;
    let base_clone_dir = resolve::expand_tilde(&sub.base_clone_dir);
    let sub_dir = resolve::expand_tilde(&sub.local_path);
    let repos_toml = sub_dir.join("repos.toml");

    if !repos_toml.exists() {
        eprintln!(
            "Warning: repos.toml not found for subscription '{}' at {}, skipping",
            sub.name,
            repos_toml.display()
        );
        return None;
    }

    let repos = match puugit_core::config::ReposConfig::load(&repos_toml) {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "Warning: failed to load repos.toml for subscription '{}': {e}, skipping",
                sub.name
            );
            return None;
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
                    &sub.account_map,
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
                    raw_url: child.url.clone().unwrap_or_default(),
                    account: child.account.clone().unwrap_or_default(),
                    tree_name: tree_group.name.clone(),
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

    Some(SubscriptionTree {
        name: sub.name.clone(),
        nodes: folder_nodes,
        repos,
        repos_toml_path: repos_toml,
    })
}

fn build_tree(local: &puugit_core::config::LocalConfig) -> Vec<SubscriptionTree> {
    (0..local.subscriptions.len())
        .filter_map(|i| load_subscription_tree(local, i))
        .collect()
}
