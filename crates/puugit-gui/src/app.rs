use crate::tree_view::{NodeKind, TreeNode};

pub struct PuugitApp {
    tree: Vec<TreeNode>,
    error_message: Option<String>,
}

impl PuugitApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        match build_tree() {
            Ok(tree) => Self {
                tree,
                error_message: None,
            },
            Err(msg) => Self {
                tree: vec![],
                error_message: Some(msg),
            },
        }
    }
}

impl eframe::App for PuugitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(msg) = &self.error_message {
                ui.colored_label(egui::Color32::RED, msg);
                return;
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                for node in &mut self.tree {
                    crate::tree_view::show_node(ui, node);
                }
            });
        });
    }
}

fn build_tree() -> Result<Vec<TreeNode>, String> {
    use puugit_core::config::resolve;

    let local_path = puugit_core::config::LocalConfig::default_path()
        .map_err(|e| format!("Failed to resolve config path: {e}"))?;

    if !local_path.exists() {
        return Err(
            "No configuration found. Please create ~/.config/puugit/local.toml".to_string(),
        );
    }

    let local = puugit_core::config::LocalConfig::load(&local_path)
        .map_err(|e| format!("Failed to load local.toml: {e}"))?;

    let base_clone_dir = resolve::expand_tilde(&local.base_clone_dir);

    let mut top_nodes: Vec<TreeNode> = Vec::new();

    for sub in &local.subscriptions {
        let sub_dir = resolve::expand_tilde(&sub.local_path);
        let repos_toml = sub_dir.join("repos.toml");

        if !repos_toml.exists() {
            eprintln!(
                "Warning: repos.toml not found for subscription '{}' at {}, skipping",
                sub.name,
                repos_toml.display()
            );
            continue;
        }

        let repos = match puugit_core::config::ReposConfig::load(&repos_toml) {
            Ok(r) => r,
            Err(e) => {
                eprintln!(
                    "Warning: failed to load repos.toml for subscription '{}': {e}, skipping",
                    sub.name
                );
                continue;
            }
        };

        for tree_group in &repos.tree {
            let mut children: Vec<TreeNode> = Vec::new();

            for child in &tree_group.children {
                let local_repo_path =
                    resolve::resolve_local_path(child, &tree_group.name, &base_clone_dir);

                let url = {
                    let raw = child.url.as_deref().unwrap_or("");
                    match &child.account {
                        Some(acc) => resolve::resolve_clone_url(raw, acc, &repos.accounts),
                        None => raw.to_string(),
                    }
                };

                let cloned = local_repo_path.exists();
                let status = if cloned {
                    puugit_core::repo_status::get_repo_status(&local_repo_path).ok()
                } else {
                    None
                };

                children.push(TreeNode {
                    name: child.name.clone(),
                    kind: NodeKind::Repo {
                        url,
                        local_path: if cloned { Some(local_repo_path) } else { None },
                        status,
                    },
                    children: vec![],
                    expanded: false,
                });
            }

            top_nodes.push(TreeNode {
                name: tree_group.name.clone(),
                kind: NodeKind::Folder,
                children,
                expanded: true,
            });
        }
    }

    Ok(top_nodes)
}
