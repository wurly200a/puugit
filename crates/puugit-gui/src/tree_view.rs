use std::path::PathBuf;

use puugit_core::repo_status::RepoStatus;

#[allow(dead_code)]
pub enum NodeKind {
    Folder,
    Repo {
        url: String,
        local_path: Option<PathBuf>,
        status: Option<RepoStatus>,
    },
}

pub struct TreeNode {
    pub name: String,
    pub kind: NodeKind,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
}

pub fn initial_tree() -> Vec<TreeNode> {
    vec![
        TreeNode {
            name: "mi".into(),
            kind: NodeKind::Folder,
            expanded: true,
            children: vec![
                TreeNode {
                    name: "xdx-rs".into(),
                    kind: NodeKind::Repo {
                        url: "git@github.com:wurly/xdx-rs.git".into(),
                        local_path: Some(repo_path("project/mi/xdx-rs")),
                        status: None,
                    },
                    expanded: false,
                    children: vec![],
                },
                TreeNode {
                    name: "puugit".into(),
                    kind: NodeKind::Repo {
                        url: "git@github.com:wurly200a/puugit.git".into(),
                        local_path: Some(repo_path("puugit")),
                        status: None,
                    },
                    expanded: false,
                    children: vec![],
                },
                TreeNode {
                    name: "some-synth".into(),
                    kind: NodeKind::Repo {
                        url: "git@github.com:wurly/some-synth.git".into(),
                        local_path: None, // not cloned
                        status: None,
                    },
                    expanded: false,
                    children: vec![],
                },
            ],
        },
    ]
}

#[cfg(target_os = "windows")]
fn repo_path(rel: &str) -> PathBuf {
    PathBuf::from("D:/home/yushi").join(rel)
}

#[cfg(not(target_os = "windows"))]
fn repo_path(rel: &str) -> PathBuf {
    PathBuf::from("/home/yushi").join(rel)
}

pub fn show_node(ui: &mut egui::Ui, node: &mut TreeNode) {
    match &mut node.kind {
        NodeKind::Folder => {
            egui::CollapsingHeader::new(&node.name)
                .default_open(node.expanded)
                .show(ui, |ui| {
                    for child in &mut node.children {
                        show_node(ui, child);
                    }
                });
        }
        NodeKind::Repo {
            local_path, status, ..
        } => {
            ui.horizontal(|ui| {
                let mut cloned = local_path.is_some();
                ui.checkbox(&mut cloned, "");

                if local_path.is_some() {
                    ui.colored_label(egui::Color32::GREEN, &node.name);
                } else {
                    ui.colored_label(egui::Color32::GRAY, &node.name);
                }

                match (local_path, status) {
                    (None, _) => {
                        ui.label("(not cloned)");
                    }
                    (Some(_), None) => {
                        ui.colored_label(egui::Color32::GRAY, "(loading...)");
                    }
                    (Some(_), Some(s)) => {
                        show_badges(ui, s);
                    }
                }
            });
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
