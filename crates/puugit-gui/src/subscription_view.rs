use std::path::Path;

use puugit_core::config::local::Subscription;

pub struct SubscriptionWindow {
    pub open: bool,
    was_open: bool,
    new_name: String,
    new_config_repo: String,
    new_account_idx: usize,
    new_local_path: String,
    new_base_clone_dir: String,
}

impl SubscriptionWindow {
    pub fn new() -> Self {
        Self {
            open: false,
            was_open: false,
            new_name: String::new(),
            new_config_repo: String::new(),
            new_account_idx: 0,
            new_local_path: String::new(),
            new_base_clone_dir: String::new(),
        }
    }

    /// Shows the subscription management window.
    /// Returns true if subscriptions were modified (caller should rebuild tree).
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        config: &mut puugit_core::config::LocalConfig,
        config_path: &Path,
    ) -> bool {
        if self.open && !self.was_open {
            // Reset new-entry form when window opens
            self.new_name.clear();
            self.new_config_repo.clear();
            self.new_account_idx = 0;
            self.new_local_path.clear();
            self.new_base_clone_dir.clear();
        }
        self.was_open = self.open;

        if !self.open {
            return false;
        }

        // Pre-compute account name list before closures borrow config
        let account_names: Vec<String> = {
            let mut v: Vec<String> = config.account_keys.keys().cloned().collect();
            v.sort();
            v
        };

        let mut open = true;
        let mut modified = false;
        let mut to_delete: Option<usize> = None;
        let mut to_add: Option<Subscription> = None;

        // Borrow new-entry fields before the Window closure
        let new_name = &mut self.new_name;
        let new_config_repo = &mut self.new_config_repo;
        let new_account_idx = &mut self.new_account_idx;
        let new_local_path = &mut self.new_local_path;
        let new_base_clone_dir = &mut self.new_base_clone_dir;

        egui::Window::new("Subscriptions")
            .open(&mut open)
            .resizable(true)
            .min_width(480.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for (i, sub) in config.subscriptions.iter_mut().enumerate() {
                            ui.group(|ui| {
                                egui::Grid::new(format!("sub_{}", i))
                                    .num_columns(2)
                                    .spacing([8.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Name:");
                                        if ui
                                            .add(
                                                egui::TextEdit::singleline(&mut sub.name)
                                                    .desired_width(200.0),
                                            )
                                            .changed()
                                        {
                                            modified = true;
                                        }
                                        ui.end_row();

                                        ui.label("Config Repo:");
                                        if ui
                                            .add(
                                                egui::TextEdit::singleline(&mut sub.config_repo)
                                                    .desired_width(300.0),
                                            )
                                            .changed()
                                        {
                                            modified = true;
                                        }
                                        ui.end_row();

                                        ui.label("Account:");
                                        let mut acc_idx = account_names
                                            .iter()
                                            .position(|a| a == &sub.account)
                                            .unwrap_or(0);
                                        egui::ComboBox::from_id_source(format!("sub_acc_{}", i))
                                            .selected_text(
                                                account_names
                                                    .get(acc_idx)
                                                    .map(|s| s.as_str())
                                                    .unwrap_or(&sub.account),
                                            )
                                            .show_ui(ui, |ui| {
                                                for (j, acc) in account_names.iter().enumerate() {
                                                    ui.selectable_value(&mut acc_idx, j, acc);
                                                }
                                            });
                                        if let Some(new_acc) = account_names.get(acc_idx) {
                                            if new_acc != &sub.account {
                                                sub.account = new_acc.clone();
                                                modified = true;
                                            }
                                        }
                                        ui.end_row();

                                        ui.label("Base Clone Dir:");
                                        if ui
                                            .add(
                                                egui::TextEdit::singleline(&mut sub.base_clone_dir)
                                                    .desired_width(300.0),
                                            )
                                            .changed()
                                        {
                                            modified = true;
                                        }
                                        ui.end_row();

                                        ui.label("Local Path:");
                                        if ui
                                            .add(
                                                egui::TextEdit::singleline(&mut sub.local_path)
                                                    .desired_width(300.0),
                                            )
                                            .changed()
                                        {
                                            modified = true;
                                        }
                                        ui.end_row();
                                    });

                                if ui
                                    .add(
                                        egui::Button::new("Delete")
                                            .fill(egui::Color32::from_rgb(180, 60, 60)),
                                    )
                                    .clicked()
                                {
                                    to_delete = Some(i);
                                }
                            });
                            ui.add_space(4.0);
                        }
                    });

                ui.separator();
                ui.label("Add new subscription:");

                egui::Grid::new("sub_new")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.add(egui::TextEdit::singleline(new_name).desired_width(200.0));
                        ui.end_row();

                        ui.label("Config Repo:");
                        ui.add(egui::TextEdit::singleline(new_config_repo).desired_width(300.0));
                        ui.end_row();

                        ui.label("Account:");
                        egui::ComboBox::from_id_source("new_sub_acc")
                            .selected_text(
                                account_names
                                    .get(*new_account_idx)
                                    .map(|s| s.as_str())
                                    .unwrap_or("(none)"),
                            )
                            .show_ui(ui, |ui| {
                                for (j, acc) in account_names.iter().enumerate() {
                                    ui.selectable_value(new_account_idx, j, acc);
                                }
                            });
                        ui.end_row();

                        ui.label("Base Clone Dir:");
                        ui.add(egui::TextEdit::singleline(new_base_clone_dir).desired_width(300.0));
                        ui.end_row();

                        ui.label("Local Path:");
                        ui.add(egui::TextEdit::singleline(new_local_path).desired_width(300.0));
                        ui.end_row();
                    });

                let can_add = !new_name.is_empty()
                    && !new_config_repo.is_empty()
                    && !new_base_clone_dir.is_empty()
                    && !new_local_path.is_empty()
                    && !account_names.is_empty();

                if ui.add_enabled(can_add, egui::Button::new("Add")).clicked() {
                    let account = account_names
                        .get(*new_account_idx)
                        .cloned()
                        .unwrap_or_default();
                    to_add = Some(Subscription {
                        name: new_name.clone(),
                        config_repo: new_config_repo.clone(),
                        account,
                        local_path: new_local_path.clone(),
                        base_clone_dir: new_base_clone_dir.clone(),
                    });
                    new_name.clear();
                    new_config_repo.clear();
                    new_local_path.clear();
                    new_base_clone_dir.clear();
                }
            });

        self.open = open;

        // Apply mutations after closure
        if let Some(i) = to_delete {
            config.subscriptions.remove(i);
            modified = true;
        }
        if let Some(sub) = to_add {
            config.subscriptions.push(sub);
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
