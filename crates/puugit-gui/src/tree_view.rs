#[allow(dead_code)]
pub enum NodeKind {
    Folder,
    Repo { url: String, cloned: bool },
}

pub struct TreeNode {
    pub name: String,
    pub kind: NodeKind,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
}

pub fn dummy_tree() -> Vec<TreeNode> {
    vec![
        TreeNode {
            name: "music".into(),
            kind: NodeKind::Folder,
            expanded: true,
            children: vec![
                TreeNode {
                    name: "xdx-rs".into(),
                    kind: NodeKind::Repo {
                        url: "git@github.com:wurly/xdx-rs.git".into(),
                        cloned: true,
                    },
                    expanded: false,
                    children: vec![],
                },
                TreeNode {
                    name: "some-synth".into(),
                    kind: NodeKind::Repo {
                        url: "git@github.com:wurly/some-synth.git".into(),
                        cloned: false,
                    },
                    expanded: false,
                    children: vec![],
                },
            ],
        },
        TreeNode {
            name: "work".into(),
            kind: NodeKind::Folder,
            expanded: true,
            children: vec![TreeNode {
                name: "project-a".into(),
                kind: NodeKind::Repo {
                    url: "git@github.com:wurly-work/project-a.git".into(),
                    cloned: true,
                },
                expanded: false,
                children: vec![],
            }],
        },
    ]
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
        NodeKind::Repo { cloned, .. } => {
            ui.horizontal(|ui| {
                ui.checkbox(cloned, "");
                if *cloned {
                    ui.colored_label(egui::Color32::GREEN, &node.name);
                } else {
                    ui.colored_label(egui::Color32::GRAY, &node.name);
                }
            });
        }
    }
}
