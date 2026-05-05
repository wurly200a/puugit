use std::path::PathBuf;
use std::time::SystemTime;

use puugit_core::repo_status::RepoStatus;

pub enum SidePanelAction {
    None,
    SaveEdit {
        old_tree: String,
        repo_name: String,
        new_url: String,
        new_account: String,
        new_tree: String,
    },
    Delete {
        tree_name: String,
        repo_name: String,
    },
}

pub struct SidePanel {
    pub selected_repo: Option<SelectedRepo>,
    show_all_unstaged: bool,
    show_all_staged: bool,
    show_all_untracked: bool,

    edit_mode: bool,
    edit_url: String,
    edit_account: String,
    edit_tree: String,
    edit_tree_is_new: bool,
    edit_tree_new_name: String,

    show_delete_confirm: bool,
}

pub struct SelectedRepo {
    pub name: String,
    pub local_path: PathBuf,
    pub cloned: bool,
    pub url: String,
    pub account: String,
    pub tree_name: String,
    pub status: Option<RepoStatus>,
}

const FILES_PREVIEW_COUNT: usize = 5;

impl SidePanel {
    pub fn new() -> Self {
        Self {
            selected_repo: None,
            show_all_unstaged: false,
            show_all_staged: false,
            show_all_untracked: false,
            edit_mode: false,
            edit_url: String::new(),
            edit_account: String::new(),
            edit_tree: String::new(),
            edit_tree_is_new: false,
            edit_tree_new_name: String::new(),
            show_delete_confirm: false,
        }
    }

    pub fn select(
        &mut self,
        name: String,
        local_path: PathBuf,
        cloned: bool,
        url: String,
        account: String,
        tree_name: String,
    ) {
        let status = if cloned {
            puugit_core::repo_status::get_repo_status(&local_path).ok()
        } else {
            None
        };
        self.selected_repo = Some(SelectedRepo {
            name,
            local_path,
            cloned,
            url,
            account,
            tree_name,
            status,
        });
        self.show_all_unstaged = false;
        self.show_all_staged = false;
        self.show_all_untracked = false;
        self.edit_mode = false;
        self.show_delete_confirm = false;
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        account_names: &[String],
        tree_names: &[String],
    ) -> SidePanelAction {
        if self.selected_repo.is_none() {
            self.edit_mode = false;
            self.show_delete_confirm = false;
        }

        let mut do_fetch = false;
        let mut action = SidePanelAction::None;

        {
            let selected_repo = &self.selected_repo;
            let show_all_unstaged = &mut self.show_all_unstaged;
            let show_all_staged = &mut self.show_all_staged;
            let show_all_untracked = &mut self.show_all_untracked;
            let edit_mode = &mut self.edit_mode;
            let edit_url = &mut self.edit_url;
            let edit_account = &mut self.edit_account;
            let edit_tree = &mut self.edit_tree;
            let edit_tree_is_new = &mut self.edit_tree_is_new;
            let edit_tree_new_name = &mut self.edit_tree_new_name;
            let show_delete_confirm = &mut self.show_delete_confirm;

            egui::SidePanel::right("detail_panel")
                .min_width(300.0)
                .show(ctx, |ui| {
                    let (df, a) = show_contents(
                        ui,
                        selected_repo,
                        show_all_unstaged,
                        show_all_staged,
                        show_all_untracked,
                        edit_mode,
                        edit_url,
                        edit_account,
                        edit_tree,
                        edit_tree_is_new,
                        edit_tree_new_name,
                        show_delete_confirm,
                        account_names,
                        tree_names,
                    );
                    do_fetch = df;
                    action = a;
                });
        }

        if do_fetch {
            if let Some(ref mut selected) = self.selected_repo {
                let _ = std::process::Command::new("git")
                    .args(["fetch", "--all"])
                    .current_dir(&selected.local_path)
                    .output();
                let local_path = selected.local_path.clone();
                selected.status = puugit_core::repo_status::get_repo_status(&local_path).ok();
            }
        }

        action
    }
}

#[allow(clippy::too_many_arguments)]
fn show_contents(
    ui: &mut egui::Ui,
    selected_repo: &Option<SelectedRepo>,
    show_all_unstaged: &mut bool,
    show_all_staged: &mut bool,
    show_all_untracked: &mut bool,
    edit_mode: &mut bool,
    edit_url: &mut String,
    edit_account: &mut String,
    edit_tree: &mut String,
    edit_tree_is_new: &mut bool,
    edit_tree_new_name: &mut String,
    show_delete_confirm: &mut bool,
    account_names: &[String],
    tree_names: &[String],
) -> (bool, SidePanelAction) {
    let Some(selected) = selected_repo else {
        ui.centered_and_justified(|ui| {
            ui.label("Select a repository to view details");
        });
        return (false, SidePanelAction::None);
    };

    let mut do_fetch = false;
    let mut action = SidePanelAction::None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Header row with action buttons
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(&selected.name).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if *show_delete_confirm {
                    if ui.button("Cancel").clicked() {
                        *show_delete_confirm = false;
                    }
                    if ui.button("OK").clicked() {
                        action = SidePanelAction::Delete {
                            tree_name: selected.tree_name.clone(),
                            repo_name: selected.name.clone(),
                        };
                        *show_delete_confirm = false;
                    }
                } else if *edit_mode {
                    if ui.button("Cancel").clicked() {
                        *edit_mode = false;
                    }
                    if ui.button("Save").clicked() {
                        let new_tree = if *edit_tree_is_new {
                            edit_tree_new_name.trim().to_string()
                        } else {
                            edit_tree.clone()
                        };
                        if !edit_url.trim().is_empty() && !new_tree.is_empty() {
                            action = SidePanelAction::SaveEdit {
                                old_tree: selected.tree_name.clone(),
                                repo_name: selected.name.clone(),
                                new_url: edit_url.trim().to_string(),
                                new_account: edit_account.clone(),
                                new_tree,
                            };
                            *edit_mode = false;
                        }
                    }
                } else {
                    if ui.button("Delete").clicked() {
                        *show_delete_confirm = true;
                    }
                    if ui.button("Edit").clicked() {
                        *edit_mode = true;
                        *edit_url = selected.url.clone();
                        *edit_account = selected.account.clone();
                        *edit_tree = selected.tree_name.clone();
                        *edit_tree_is_new = false;
                        edit_tree_new_name.clear();
                    }
                }
            });
        });

        if *show_delete_confirm {
            ui.colored_label(
                egui::Color32::from_rgb(255, 180, 0),
                format!(
                    "Delete '{}' from repos.toml?\n(Local files will NOT be deleted)",
                    selected.name
                ),
            );
        }

        ui.separator();

        if *edit_mode {
            egui::Grid::new("side_edit_grid")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label("URL:");
                    ui.add(egui::TextEdit::singleline(edit_url).desired_width(f32::INFINITY));
                    ui.end_row();

                    ui.label("Account:");
                    let acc_display = if edit_account.is_empty() {
                        "(none)".to_string()
                    } else {
                        edit_account.clone()
                    };
                    egui::ComboBox::from_id_source("side_edit_account")
                        .selected_text(acc_display)
                        .show_ui(ui, |ui| {
                            for name in account_names {
                                ui.selectable_value(edit_account, name.clone(), name);
                            }
                        });
                    ui.end_row();

                    ui.label("Tree:");
                    let tree_display = if *edit_tree_is_new {
                        "[ + New Tree ]".to_string()
                    } else {
                        edit_tree.clone()
                    };
                    egui::ComboBox::from_id_source("side_edit_tree")
                        .selected_text(tree_display)
                        .show_ui(ui, |ui| {
                            for name in tree_names {
                                if ui
                                    .selectable_label(
                                        !*edit_tree_is_new && edit_tree.as_str() == name,
                                        name,
                                    )
                                    .clicked()
                                {
                                    *edit_tree = name.clone();
                                    *edit_tree_is_new = false;
                                }
                            }
                            if ui
                                .selectable_label(*edit_tree_is_new, "[ + New Tree ]")
                                .clicked()
                            {
                                *edit_tree_is_new = true;
                            }
                        });
                    ui.end_row();

                    if *edit_tree_is_new {
                        ui.label("Tree Name:");
                        ui.add(
                            egui::TextEdit::singleline(edit_tree_new_name)
                                .desired_width(f32::INFINITY),
                        );
                        ui.end_row();
                    }
                });

            ui.separator();
        } else {
            egui::Grid::new("side_info_grid")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label("URL:");
                    ui.label(
                        egui::RichText::new(&selected.url)
                            .small()
                            .color(egui::Color32::LIGHT_GRAY),
                    );
                    ui.end_row();

                    ui.label("Account:");
                    ui.label(if selected.account.is_empty() {
                        "(none)"
                    } else {
                        &selected.account
                    });
                    ui.end_row();

                    ui.label("Tree:");
                    ui.label(&selected.tree_name);
                    ui.end_row();

                    ui.label("Path:");
                    if selected.cloned {
                        let path_str = selected.local_path.to_string_lossy();
                        if ui.link(path_str.as_ref()).clicked() {
                            open_in_file_manager(&selected.local_path);
                        }
                    } else {
                        ui.label(
                            egui::RichText::new(selected.local_path.to_string_lossy().as_ref())
                                .color(egui::Color32::GRAY),
                        );
                    }
                    ui.end_row();
                });

            ui.separator();

            if selected.cloned {
                let fetch_text = selected
                    .status
                    .as_ref()
                    .and_then(|s| s.last_fetch_time)
                    .map(|t| format!("Last fetch: {}", format_elapsed(t)))
                    .unwrap_or_else(|| "Last fetch: never".to_string());
                ui.label(fetch_text);

                if ui.button("\u{1f504} Fetch").clicked() {
                    do_fetch = true;
                }

                ui.separator();

                match &selected.status {
                    None => {
                        ui.label("Status unavailable");
                    }
                    Some(status) => {
                        let any_issue = show_status(
                            ui,
                            status,
                            show_all_unstaged,
                            show_all_staged,
                            show_all_untracked,
                        );
                        if !any_issue {
                            ui.colored_label(
                                egui::Color32::GREEN,
                                "\u{2705} Clean - safe to remove",
                            );
                        }
                    }
                }
            }
        }
    });

    (do_fetch, action)
}

fn open_in_file_manager(path: &std::path::Path) {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .ok();
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .ok();
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn().ok();
    }
}

fn show_status(
    ui: &mut egui::Ui,
    status: &RepoStatus,
    show_all_unstaged: &mut bool,
    show_all_staged: &mut bool,
    show_all_untracked: &mut bool,
) -> bool {
    let mut any_issue = false;

    if !status.unpushed_branches.is_empty() {
        any_issue = true;
        ui.colored_label(egui::Color32::RED, "\u{1f534} Unpushed Commits");
        ui.indent("unpushed", |ui| {
            for branch in &status.unpushed_branches {
                let n = branch.commit_count;
                ui.label(format!(
                    "  {}  (+{} commit{})",
                    branch.name,
                    n,
                    if n == 1 { "" } else { "s" }
                ));
            }
        });
    }

    if status.stash_count > 0 {
        any_issue = true;
        ui.colored_label(
            egui::Color32::RED,
            format!("\u{1f534} Stash Entries: {}", status.stash_count),
        );
    }

    if !status.unstaged_files.is_empty() {
        any_issue = true;
        ui.colored_label(
            egui::Color32::YELLOW,
            format!(
                "\u{1f7e1} Unstaged Changes ({} files)",
                status.unstaged_files.len()
            ),
        );
        show_file_list(
            ui,
            &status.unstaged_files,
            show_all_unstaged,
            "unstaged_files",
        );
    }

    if !status.staged_files.is_empty() {
        any_issue = true;
        ui.colored_label(
            egui::Color32::YELLOW,
            format!(
                "\u{1f7e1} Staged Changes ({} files)",
                status.staged_files.len()
            ),
        );
        show_file_list(ui, &status.staged_files, show_all_staged, "staged_files");
    }

    if !status.untracked_files.is_empty() {
        any_issue = true;
        ui.colored_label(
            egui::Color32::YELLOW,
            format!(
                "\u{1f7e1} Untracked Files ({} files)",
                status.untracked_files.len()
            ),
        );
        show_file_list(
            ui,
            &status.untracked_files,
            show_all_untracked,
            "untracked_files",
        );
    }

    if !status.has_remote {
        any_issue = true;
        ui.colored_label(egui::Color32::GRAY, "\u{26aa} No remote configured");
    }

    any_issue
}

fn show_file_list(ui: &mut egui::Ui, files: &[String], show_all: &mut bool, id: &str) {
    let count = files.len();
    let shown = if *show_all {
        count
    } else {
        count.min(FILES_PREVIEW_COUNT)
    };

    ui.indent(id, |ui| {
        for file in &files[..shown] {
            ui.label(file.as_str());
        }
        if !*show_all && count > FILES_PREVIEW_COUNT {
            let remaining = count - FILES_PREVIEW_COUNT;
            if ui.small_button(format!("+ {remaining} more")).clicked() {
                *show_all = true;
            }
        }
    });
}

fn format_elapsed(time: SystemTime) -> String {
    match SystemTime::now().duration_since(time) {
        Ok(d) => {
            let secs = d.as_secs();
            if secs < 60 {
                "just now".to_string()
            } else if secs < 3600 {
                let m = secs / 60;
                format!("{m} minute{} ago", if m == 1 { "" } else { "s" })
            } else if secs < 86400 {
                let h = secs / 3600;
                format!("{h} hour{} ago", if h == 1 { "" } else { "s" })
            } else {
                let days = secs / 86400;
                format!("{days} day{} ago", if days == 1 { "" } else { "s" })
            }
        }
        Err(_) => "just now".to_string(),
    }
}
