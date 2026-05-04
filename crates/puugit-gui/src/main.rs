#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod account_view;
mod add_repo_dialog;
mod app;
mod dialog;
mod side_panel;
mod subscription_view;
mod tree_view;

fn is_wsl() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/version")
            .map(|v| v.to_lowercase().contains("microsoft"))
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    false
}

fn main() -> eframe::Result<()> {
    // WSLg's Weston/RDP compositor does not support hardware EGL over Wayland.
    // Fall back to X11/XWayland which works reliably on WSL.
    if is_wsl() {
        std::env::remove_var("WAYLAND_DISPLAY");
    }
    let icon = include_bytes!("../icons/puugit-icon.png");
    let image = image::load_from_memory(icon).unwrap();
    let rgba = image.to_rgba8();
    let (w, h) = rgba.dimensions();
    let icon_data = egui::IconData {
        rgba: rgba.into_raw(),
        width: w,
        height: h,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("puugit")
            .with_inner_size([1000.0, 700.0])
            .with_icon(std::sync::Arc::new(icon_data)),
        ..Default::default()
    };
    eframe::run_native(
        "puugit",
        options,
        Box::new(|cc| Box::new(app::PuugitApp::new(cc))),
    )
}
