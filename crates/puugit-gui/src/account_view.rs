use std::path::Path;

pub struct AccountWindow {
    pub open: bool,
    was_open: bool,
    ssh_aliases: Vec<String>,
    new_name: String,
    new_alias_idx: usize,
}

impl AccountWindow {
    pub fn new() -> Self {
        Self {
            open: false,
            was_open: false,
            ssh_aliases: Vec::new(),
            new_name: String::new(),
            new_alias_idx: 0,
        }
    }

    /// Shows the account management window.
    /// Returns true if account_keys was modified (caller should rebuild tree).
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        config: &mut puugit_core::config::LocalConfig,
        config_path: &Path,
    ) -> bool {
        // Refresh SSH aliases each time the window is opened
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

        let mut open = true;
        let mut changes: Vec<(String, String)> = Vec::new();
        let mut to_delete: Option<String> = None;

        // Split borrows of self fields before closure
        let ssh_aliases = &self.ssh_aliases;
        let new_alias_idx = &mut self.new_alias_idx;
        let new_name = &mut self.new_name;

        egui::Window::new("Accounts")
            .open(&mut open)
            .resizable(false)
            .min_width(400.0)
            .show(ctx, |ui| {
                egui::Grid::new("accounts_grid")
                    .num_columns(3)
                    .spacing([12.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Account");
                        ui.strong("SSH Alias");
                        ui.label("");
                        ui.end_row();

                        let mut keys: Vec<String> = config.account_keys.keys().cloned().collect();
                        keys.sort();

                        for name in &keys {
                            let old_alias =
                                config.account_keys.get(name).cloned().unwrap_or_default();
                            let mut current_idx = ssh_aliases
                                .iter()
                                .position(|a| a == &old_alias)
                                .unwrap_or(0);

                            ui.label(name);

                            let selected_text = ssh_aliases
                                .get(current_idx)
                                .map(|s| s.as_str())
                                .unwrap_or(old_alias.as_str());

                            egui::ComboBox::from_id_source(name.as_str())
                                .selected_text(selected_text)
                                .show_ui(ui, |ui| {
                                    for (i, alias) in ssh_aliases.iter().enumerate() {
                                        ui.selectable_value(&mut current_idx, i, alias);
                                    }
                                });

                            // Detect selection change
                            if let Some(new_alias) = ssh_aliases.get(current_idx) {
                                if new_alias != &old_alias {
                                    changes.push((name.clone(), new_alias.clone()));
                                }
                            }

                            if ui.button("Delete").clicked() {
                                to_delete = Some(name.clone());
                            }

                            ui.end_row();
                        }
                    });

                ui.separator();

                // Add new account row
                ui.horizontal(|ui| {
                    ui.label("New:");
                    ui.add(egui::TextEdit::singleline(new_name).desired_width(100.0));

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

                    let can_add = !new_name.is_empty() && !ssh_aliases.is_empty();
                    if ui.add_enabled(can_add, egui::Button::new("Add")).clicked() {
                        if let Some(alias) = ssh_aliases.get(*new_alias_idx) {
                            changes.push((new_name.clone(), alias.clone()));
                            new_name.clear();
                        }
                    }
                });
            });

        self.open = open;

        // Apply all pending changes
        let mut modified = false;

        if let Some(name) = to_delete {
            config.account_keys.remove(&name);
            modified = true;
        }
        for (name, alias) in changes {
            config.account_keys.insert(name, alias);
            modified = true;
        }
        if modified {
            if let Err(e) = config.save(config_path) {
                eprintln!("Failed to save local.toml: {e}");
            }
        }

        modified
    }
}
