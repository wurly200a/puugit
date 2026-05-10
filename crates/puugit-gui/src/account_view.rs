use std::path::Path;

pub struct AccountWindow {
    pub open: bool,
    was_open: bool,
    ssh_aliases: Vec<String>,
    new_label: String,
    new_alias_idx: usize,
}

impl AccountWindow {
    pub fn new() -> Self {
        Self {
            open: false,
            was_open: false,
            ssh_aliases: Vec::new(),
            new_label: String::new(),
            new_alias_idx: 0,
        }
    }

    /// Shows the Account Map window for the currently selected subscription.
    /// Returns true if account_map was modified (caller should rebuild tree).
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        config: &mut puugit_core::config::LocalConfig,
        selected_idx: usize,
        config_path: &Path,
    ) -> bool {
        if self.open && !self.was_open {
            self.ssh_aliases = puugit_core::ssh_config::parse_ssh_config()
                .into_iter()
                .map(|e| e.alias)
                .collect();
            self.new_alias_idx = 0;
        }
        self.was_open = self.open;

        if !self.open {
            return false;
        }

        let Some(sub) = config.subscriptions.get_mut(selected_idx) else {
            self.open = false;
            return false;
        };

        let title = format!("Account Map - {}", sub.name);
        let mut open = true;
        let mut changes: Vec<(String, String)> = Vec::new();
        let mut to_delete: Option<String> = None;

        let ssh_aliases = &self.ssh_aliases;
        let new_alias_idx = &mut self.new_alias_idx;
        let new_label = &mut self.new_label;

        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .min_width(440.0)
            .show(ctx, |ui| {
                egui::Grid::new("account_map_grid")
                    .num_columns(3)
                    .spacing([12.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Label");
                        ui.strong("SSH Alias");
                        ui.label("");
                        ui.end_row();

                        let mut labels: Vec<String> = sub.account_map.keys().cloned().collect();
                        labels.sort();

                        for label in &labels {
                            let old_alias = sub.account_map.get(label).cloned().unwrap_or_default();
                            let mut current_idx = ssh_aliases
                                .iter()
                                .position(|a| a == &old_alias)
                                .unwrap_or(0);

                            ui.label(label);

                            let selected_text = ssh_aliases
                                .get(current_idx)
                                .map(|s| s.as_str())
                                .unwrap_or(old_alias.as_str());

                            egui::ComboBox::from_id_source(label.as_str())
                                .selected_text(selected_text)
                                .show_ui(ui, |ui| {
                                    for (i, alias) in ssh_aliases.iter().enumerate() {
                                        ui.selectable_value(&mut current_idx, i, alias);
                                    }
                                });

                            if let Some(new_alias) = ssh_aliases.get(current_idx) {
                                if new_alias != &old_alias {
                                    changes.push((label.clone(), new_alias.clone()));
                                }
                            }

                            if ui.button("Delete").clicked() {
                                to_delete = Some(label.clone());
                            }

                            ui.end_row();
                        }
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("New:");
                    ui.add(egui::TextEdit::singleline(new_label).desired_width(120.0));

                    let alias_text = ssh_aliases
                        .get(*new_alias_idx)
                        .map(|s| s.as_str())
                        .unwrap_or("(no aliases)");

                    egui::ComboBox::from_id_source("new_alias_combo")
                        .selected_text(alias_text)
                        .show_ui(ui, |ui| {
                            for (i, alias) in ssh_aliases.iter().enumerate() {
                                ui.selectable_value(new_alias_idx, i, alias);
                            }
                        });

                    let can_add = !new_label.is_empty() && !ssh_aliases.is_empty();
                    if ui.add_enabled(can_add, egui::Button::new("Add")).clicked() {
                        if let Some(alias) = ssh_aliases.get(*new_alias_idx) {
                            changes.push((new_label.clone(), alias.clone()));
                            new_label.clear();
                        }
                    }
                });
            });

        self.open = open;

        let mut modified = false;

        // Re-borrow sub after closures are done
        if let Some(sub) = config.subscriptions.get_mut(selected_idx) {
            if let Some(label) = to_delete {
                sub.account_map.remove(&label);
                modified = true;
            }
            for (label, alias) in changes {
                sub.account_map.insert(label, alias);
                modified = true;
            }
        }

        if modified {
            if let Err(e) = config.save(config_path) {
                eprintln!("Failed to save local.toml: {e}");
            }
        }

        modified
    }
}
