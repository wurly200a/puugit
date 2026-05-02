use crate::tree_view::{self, NodeKind, TreeNode};

pub struct PuugitApp {
    tree: Vec<TreeNode>,
}

impl PuugitApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut tree = tree_view::initial_tree();
        load_statuses(&mut tree);
        Self { tree }
    }
}

impl eframe::App for PuugitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for node in &mut self.tree {
                    tree_view::show_node(ui, node);
                }
            });
        });
    }
}

fn load_statuses(nodes: &mut Vec<TreeNode>) {
    for node in nodes.iter_mut() {
        match &mut node.kind {
            NodeKind::Repo {
                local_path: Some(path),
                status,
                ..
            } => {
                *status = puugit_core::repo_status::get_repo_status(path).ok();
            }
            NodeKind::Folder => load_statuses(&mut node.children),
            _ => {}
        }
    }
}
