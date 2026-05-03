use std::path::PathBuf;

use puugit_core::repo_status::RepoStatus;

#[allow(dead_code)]
pub enum NodeKind {
    Folder,
    Repo {
        url: String,
        local_path: PathBuf,
        cloned: bool,
        status: Option<RepoStatus>,
    },
}

pub struct TreeNode {
    pub name: String,
    pub kind: NodeKind,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
}

pub enum NodeAction {
    Clone {
        url: String,
        local_path: PathBuf,
        repo_name: String,
    },
    Remove {
        local_path: PathBuf,
        repo_name: String,
    },
}

pub fn show_node(ui: &mut egui::Ui, node: &mut TreeNode, actions: &mut Vec<NodeAction>) {
    match &mut node.kind {
        NodeKind::Folder => {
            egui::CollapsingHeader::new(&node.name)
                .default_open(node.expanded)
                .show(ui, |ui| {
                    for child in &mut node.children {
                        show_node(ui, child, actions);
                    }
                });
        }
        NodeKind::Repo {
            url,
            local_path,
            cloned,
            status,
        } => {
            let was_cloned = *cloned;
            let mut checked = was_cloned;

            ui.horizontal(|ui| {
                ui.checkbox(&mut checked, "");

                if *cloned {
                    ui.colored_label(egui::Color32::GREEN, &node.name);
                } else {
                    ui.colored_label(egui::Color32::GRAY, &node.name);
                }

                if *cloned {
                    match status {
                        None => {
                            ui.colored_label(egui::Color32::GRAY, "(status unavailable)");
                        }
                        Some(s) => show_badges(ui, s),
                    }
                } else {
                    ui.label("(not cloned)");
                }
            });

            if checked != was_cloned {
                if checked {
                    actions.push(NodeAction::Clone {
                        url: url.clone(),
                        local_path: local_path.clone(),
                        repo_name: node.name.clone(),
                    });
                } else {
                    actions.push(NodeAction::Remove {
                        local_path: local_path.clone(),
                        repo_name: node.name.clone(),
                    });
                }
            }
        }
    }
}

fn show_badges(ui: &mut egui::Ui, s: &RepoStatus) {
    let mut any = false;

    if !s.unpushed_branches.is_empty() {
        ui.colored_label(
            egui::Color32::YELLOW,
            format!("[!] unpushed:{}", s.unpushed_branches.len()),
        );
        any = true;
    }
    if s.has_unstaged_changes {
        ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "[M] unstaged");
        any = true;
    }
    if s.has_staged_changes {
        ui.colored_label(egui::Color32::from_rgb(100, 200, 255), "[S] staged");
        any = true;
    }
    if s.has_untracked_files {
        ui.colored_label(egui::Color32::GRAY, "[?] untracked");
        any = true;
    }
    if s.stash_count > 0 {
        ui.colored_label(
            egui::Color32::from_rgb(200, 150, 255),
            format!("[stash:{}]", s.stash_count),
        );
        any = true;
    }
    if !any {
        ui.colored_label(egui::Color32::GREEN, "[clean]");
    }
}
