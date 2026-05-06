use std::path::Path;

use puugit_core::config::repos::TreeNode as RepoTreeNode;
use puugit_core::config::ReposConfig;

pub struct AddRepoDialog {
    pub open: bool,
    was_open: bool,
    url_input: String,
    name_input: String,
    selected_account: usize,
    tree_idx: usize,
    new_tree_name: String,
    error_message: Option<String>,
}

impl AddRepoDialog {
    pub fn new() -> Self {
        Self {
            open: false,
            was_open: false,
            url_input: String::new(),
            name_input: String::new(),
            selected_account: 0,
            tree_idx: 0,
            new_tree_name: String::new(),
            error_message: None,
        }
    }

    /// Returns true if repos.toml was modified.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        repos: &mut ReposConfig,
        repos_toml_path: &Path,
        account_labels: &[String],
    ) -> bool {
        if self.open && !self.was_open {
            self.url_input.clear();
            self.name_input.clear();
            self.selected_account = 0;
            self.tree_idx = 0;
            self.new_tree_name.clear();
            self.error_message = None;
        }
        self.was_open = self.open;

        if !self.open {
            return false;
        }

        let mut open = true;
        let mut modified = false;

        let url_input = &mut self.url_input;
        let name_input = &mut self.name_input;
        let selected_account = &mut self.selected_account;
        let tree_idx = &mut self.tree_idx;
        let new_tree_name = &mut self.new_tree_name;
        let error_message = &mut self.error_message;
        let new_tree_sentinel = repos.tree.len();
        let mut url_changed = false;
        let mut add_clicked = false;

        egui::Window::new("Add Repository")
            .open(&mut open)
            .resizable(false)
            .min_width(420.0)
            .show(ctx, |ui| {
                egui::Grid::new("add_repo_grid")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("URL:");
                        url_changed = ui
                            .add(egui::TextEdit::singleline(url_input).desired_width(300.0))
                            .changed();
                        ui.end_row();

                        ui.label("Name:");
                        ui.add(egui::TextEdit::singleline(name_input).desired_width(300.0));
                        ui.end_row();

                        ui.label("Account:");
                        let account_text = account_labels
                            .get(*selected_account)
                            .map(|s| s.as_str())
                            .unwrap_or("(none)");
                        egui::ComboBox::from_id_source("add_repo_account")
                            .selected_text(account_text)
                            .show_ui(ui, |ui| {
                                for (i, label) in account_labels.iter().enumerate() {
                                    ui.selectable_value(selected_account, i, label);
                                }
                            });
                        ui.end_row();

                        ui.label("Tree:");
                        let tree_text = if *tree_idx == new_tree_sentinel {
                            "[ + New Tree ]".to_string()
                        } else {
                            repos
                                .tree
                                .get(*tree_idx)
                                .map(|t| t.name.clone())
                                .unwrap_or_default()
                        };
                        egui::ComboBox::from_id_source("add_repo_tree")
                            .selected_text(tree_text)
                            .show_ui(ui, |ui| {
                                for (i, t) in repos.tree.iter().enumerate() {
                                    ui.selectable_value(tree_idx, i, &t.name);
                                }
                                ui.selectable_value(tree_idx, new_tree_sentinel, "[ + New Tree ]");
                            });
                        ui.end_row();

                        if *tree_idx == new_tree_sentinel {
                            ui.label("Tree Name:");
                            ui.add(egui::TextEdit::singleline(new_tree_name).desired_width(300.0));
                            ui.end_row();
                        }
                    });

                if let Some(ref msg) = *error_message {
                    ui.colored_label(egui::Color32::RED, msg);
                }

                add_clicked = ui.button("Add").clicked();
            });

        if url_changed {
            *name_input = extract_name_from_url(url_input);
        }

        if add_clicked {
            match do_add(
                url_input,
                name_input,
                *selected_account,
                *tree_idx,
                new_tree_name,
                new_tree_sentinel,
                repos,
                repos_toml_path,
                account_labels,
            ) {
                Ok(()) => {
                    modified = true;
                    open = false;
                }
                Err(msg) => {
                    *error_message = Some(msg);
                }
            }
        }

        self.open = open;
        modified
    }
}

fn do_add(
    url: &str,
    name: &str,
    selected_account: usize,
    tree_idx: usize,
    new_tree_name: &str,
    new_tree_sentinel: usize,
    repos: &mut ReposConfig,
    repos_toml_path: &Path,
    account_labels: &[String],
) -> Result<(), String> {
    let url = url.trim().to_string();
    let name = name.trim().to_string();

    if url.is_empty() {
        return Err("URL is required.".into());
    }
    if name.is_empty() {
        return Err("Name is required.".into());
    }

    for group in &repos.tree {
        if group.children.iter().any(|c| c.name == name) {
            return Err(format!("Repository '{}' already exists.", name));
        }
    }

    let account = account_labels.get(selected_account).cloned();

    let new_child = RepoTreeNode {
        name,
        url: Some(url),
        account,
        children: vec![],
    };

    if tree_idx == new_tree_sentinel {
        let tree_name = new_tree_name.trim().to_string();
        if tree_name.is_empty() {
            return Err("Tree name is required.".into());
        }
        repos.tree.push(RepoTreeNode {
            name: tree_name,
            url: None,
            account: None,
            children: vec![new_child],
        });
    } else {
        repos.tree[tree_idx].children.push(new_child);
    }

    repos
        .save(repos_toml_path)
        .map_err(|e| format!("Failed to save repos.toml: {e}"))
}

fn extract_name_from_url(url: &str) -> String {
    url.rsplit('/')
        .next()
        .unwrap_or("")
        .trim_end_matches(".git")
        .to_string()
}
