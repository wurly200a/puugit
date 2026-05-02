use crate::tree_view::{self, TreeNode};

pub struct PuugitApp {
    tree: Vec<TreeNode>,
}

impl PuugitApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            tree: tree_view::dummy_tree(),
        }
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
