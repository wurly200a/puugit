use std::collections::HashMap;
use std::path::PathBuf;

use puugit_core::repo_status::RepoStatus;

pub enum NodeKind {
    Folder,
    Repo {
        url: String,
        raw_url: String,
        account: String,
        tree_name: String,
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
    Select {
        name: String,
        local_path: PathBuf,
        repo_id: String,
        cloned: bool,
        url: String,
        account: String,
        tree_name: String,
    },
}

pub fn show_node(
    ui: &mut egui::Ui,
    node: &mut TreeNode,
    actions: &mut Vec<NodeAction>,
    selected_repo_id: &Option<String>,
    parent_id: &str,
    disk_usage: &HashMap<PathBuf, u64>,
    disk_usage_calculating: bool,
) {
    match &mut node.kind {
        NodeKind::Folder => {
            let child_parent_id = format!("{}/{}", parent_id, node.name);

            let (folder_total, has_partial) = folder_disk_usage(&node.children, disk_usage);
            let header_text = if folder_total > 0 {
                let suffix = if has_partial && disk_usage_calculating {
                    " …"
                } else {
                    ""
                };
                format!("{} [{}{}]", node.name, format_size(folder_total), suffix)
            } else if disk_usage_calculating {
                format!("{} […]", node.name)
            } else {
                node.name.clone()
            };

            egui::CollapsingHeader::new(&header_text)
                .id_source(&child_parent_id)
                .default_open(node.expanded)
                .show(ui, |ui| {
                    for child in &mut node.children {
                        show_node(
                            ui,
                            child,
                            actions,
                            selected_repo_id,
                            &child_parent_id,
                            disk_usage,
                            disk_usage_calculating,
                        );
                    }
                });
        }
        NodeKind::Repo {
            url,
            raw_url,
            account,
            tree_name,
            local_path,
            cloned,
            status,
        } => {
            let was_cloned = *cloned;
            let mut checked = was_cloned;

            let repo_id = format!("{}/{}", parent_id, node.name);
            let is_selected = selected_repo_id.as_deref() == Some(repo_id.as_str());

            let bg_idx = ui.painter().add(egui::Shape::Noop);

            let mut content_x = 0.0_f32;
            let inner = ui.horizontal(|ui| {
                let cb = ui.checkbox(&mut checked, "");
                content_x = cb.rect.max.x;

                let color = if *cloned {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::GRAY
                };
                ui.label(egui::RichText::new(&node.name).color(color));

                if *cloned {
                    match status {
                        None => {
                            ui.colored_label(egui::Color32::GRAY, "(status unavailable)");
                        }
                        Some(s) => show_badges(ui, s),
                    }
                    if let Some(&size) = disk_usage.get(local_path.as_path()) {
                        ui.weak(format_size(size));
                    } else if disk_usage_calculating {
                        ui.weak("…");
                    }
                } else {
                    ui.label("(not cloned)");
                }
            });

            let row_rect = inner.response.rect;
            let content_rect =
                egui::Rect::from_min_max(egui::pos2(content_x, row_rect.min.y), row_rect.max);
            let row_response = ui.interact(
                content_rect,
                ui.id().with(repo_id.as_str()),
                egui::Sense::click(),
            );

            let bg_color = if row_response.hovered() {
                egui::Color32::from_white_alpha(15)
            } else {
                egui::Color32::TRANSPARENT
            };

            ui.painter().set(
                bg_idx,
                egui::Shape::rect_filled(content_rect, 2.0, bg_color),
            );

            if is_selected {
                ui.painter().line_segment(
                    [
                        egui::pos2(content_rect.min.x, content_rect.max.y - 1.0),
                        egui::pos2(content_rect.max.x, content_rect.max.y - 1.0),
                    ],
                    egui::Stroke::new(0.5, egui::Color32::WHITE),
                );
            }

            if row_response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            if row_response.clicked() {
                actions.push(NodeAction::Select {
                    name: node.name.clone(),
                    local_path: local_path.clone(),
                    repo_id,
                    cloned: *cloned,
                    url: raw_url.clone(),
                    account: account.clone(),
                    tree_name: tree_name.clone(),
                });
            }

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

fn folder_disk_usage(
    children: &[TreeNode],
    disk_usage: &HashMap<PathBuf, u64>,
) -> (u64, bool) {
    let mut total = 0u64;
    let mut any_missing = false;
    for child in children {
        if let NodeKind::Repo {
            local_path,
            cloned: true,
            ..
        } = &child.kind
        {
            match disk_usage.get(local_path.as_path()) {
                Some(&size) => total += size,
                None => any_missing = true,
            }
        }
    }
    (total, any_missing)
}

fn format_size(bytes: u64) -> String {
    const GIB: u64 = 1 << 30;
    const MIB: u64 = 1 << 20;
    const KIB: u64 = 1 << 10;
    if bytes >= GIB {
        format!("{:.1} GB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
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
