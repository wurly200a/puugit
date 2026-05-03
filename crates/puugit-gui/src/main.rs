mod account_view;
mod app;
mod dialog;
mod subscription_view;
mod tree_view;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "puugit",
        options,
        Box::new(|cc| Box::new(app::PuugitApp::new(cc))),
    )
}
