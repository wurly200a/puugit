use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Instant;

use puugit_core::git_ops::{CloneResult, RemoveWarning};

const CLONE_TIMEOUT_SECS: u64 = 60;

pub enum Dialog {
    None,
    Cloning(CloningState),
    ConfirmDelete(DeleteState),
}

pub struct CloningState {
    pub repo_name: String,
    pub local_path: PathBuf,
    pub receiver: Receiver<CloneResult>,
    pub started_at: Instant,
    pub result: Option<CloneResult>,
}

pub struct DeleteState {
    pub repo_name: String,
    pub local_path: PathBuf,
    pub warnings: Vec<RemoveWarning>,
}

pub enum DialogAction {
    None,
    CloneSucceeded { local_path: PathBuf },
    CloneDismissed,
    DeleteConfirmed { local_path: PathBuf },
    DeleteCancelled,
}

impl Dialog {
    pub fn is_open(&self) -> bool {
        !matches!(self, Dialog::None)
    }

    pub fn show(&mut self, ctx: &egui::Context) -> DialogAction {
        match self {
            Dialog::None => DialogAction::None,
            Dialog::Cloning(state) => state.show(ctx),
            Dialog::ConfirmDelete(state) => state.show(ctx),
        }
    }
}

impl CloningState {
    fn show(&mut self, ctx: &egui::Context) -> DialogAction {
        // Poll for result or timeout
        if self.result.is_none() {
            if self.started_at.elapsed().as_secs() >= CLONE_TIMEOUT_SECS {
                self.result = Some(CloneResult::Timeout);
            } else if let Ok(r) = self.receiver.try_recv() {
                self.result = Some(r);
            } else {
                ctx.request_repaint();
            }
        }

        let mut action = DialogAction::None;

        egui::Window::new("Cloning...")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| match &self.result {
                None => {
                    let elapsed = self.started_at.elapsed().as_secs();
                    ui.label(format!("Cloning {}...", self.repo_name));
                    ui.label(format!(
                        "Please wait (timeout: {}s, elapsed: {}s)",
                        CLONE_TIMEOUT_SECS, elapsed
                    ));
                }
                Some(CloneResult::Success) => {
                    ui.colored_label(
                        egui::Color32::GREEN,
                        format!("✓ {} cloned successfully.", self.repo_name),
                    );
                    if ui.button("OK").clicked() {
                        action = DialogAction::CloneSucceeded {
                            local_path: self.local_path.clone(),
                        };
                    }
                }
                Some(CloneResult::Timeout) => {
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        format!("Clone timed out after {} seconds.", CLONE_TIMEOUT_SECS),
                    );
                    if ui.button("OK").clicked() {
                        action = DialogAction::CloneDismissed;
                    }
                }
                Some(CloneResult::Failed(msg)) => {
                    ui.colored_label(egui::Color32::RED, format!("Clone failed: {}", msg));
                    if ui.button("OK").clicked() {
                        action = DialogAction::CloneDismissed;
                    }
                }
            });

        action
    }
}

impl DeleteState {
    fn show(&mut self, ctx: &egui::Context) -> DialogAction {
        let mut action = DialogAction::None;

        egui::Window::new("Confirm Delete")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                if !self.warnings.is_empty() {
                    ui.colored_label(egui::Color32::RED, "⚠ Warnings:");
                    for w in &self.warnings {
                        ui.label(format!("  • {w}"));
                    }
                    ui.separator();
                }

                ui.label(format!("Delete '{}'?", self.repo_name));

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        action = DialogAction::DeleteCancelled;
                    }
                    let label = if self.warnings.is_empty() {
                        "Delete"
                    } else {
                        "Delete anyway"
                    };
                    if ui.button(label).clicked() {
                        action = DialogAction::DeleteConfirmed {
                            local_path: self.local_path.clone(),
                        };
                    }
                });
            });

        action
    }
}
