use std::path::PathBuf;
use std::time::SystemTime;

use puugit_core::repo_status::RepoStatus;

pub struct SidePanel {
    pub selected_repo: Option<SelectedRepo>,
    show_all_unstaged: bool,
    show_all_staged: bool,
    show_all_untracked: bool,
}

pub struct SelectedRepo {
    pub name: String,
    pub local_path: PathBuf,
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
        }
    }

    pub fn select(&mut self, name: String, local_path: PathBuf) {
        let status = puugit_core::repo_status::get_repo_status(&local_path).ok();
        self.selected_repo = Some(SelectedRepo {
            name,
            local_path,
            status,
        });
        self.show_all_unstaged = false;
        self.show_all_staged = false;
        self.show_all_untracked = false;
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        let mut do_fetch = false;

        {
            let selected_repo = &self.selected_repo;
            let show_all_unstaged = &mut self.show_all_unstaged;
            let show_all_staged = &mut self.show_all_staged;
            let show_all_untracked = &mut self.show_all_untracked;

            egui::SidePanel::right("detail_panel")
                .min_width(300.0)
                .show(ctx, |ui| {
                    do_fetch = show_contents(
                        ui,
                        selected_repo,
                        show_all_unstaged,
                        show_all_staged,
                        show_all_untracked,
                    );
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
    }
}

fn show_contents(
    ui: &mut egui::Ui,
    selected_repo: &Option<SelectedRepo>,
    show_all_unstaged: &mut bool,
    show_all_staged: &mut bool,
    show_all_untracked: &mut bool,
) -> bool {
    let Some(selected) = selected_repo else {
        ui.centered_and_justified(|ui| {
            ui.label("Select a repository to view details");
        });
        return false;
    };

    let mut do_fetch = false;

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Header
        ui.label(egui::RichText::new(&selected.name).strong());
        ui.label(
            egui::RichText::new(selected.local_path.to_string_lossy().as_ref())
                .small()
                .color(egui::Color32::GRAY),
        );

        ui.separator();

        // Fetch info
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
                    ui.colored_label(egui::Color32::GREEN, "\u{2705} Clean - safe to remove");
                }
            }
        }
    });

    do_fetch
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
