mod app;
mod config;
mod file_browser;
mod file_loader;
mod parser;
mod renderer;
mod ui;

use std::path::PathBuf;

use app::MarkdownViewerApp;

fn main() -> eframe::Result {
    let initial_file = std::env::args().nth(1).map(PathBuf::from);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Markdown Viewer")
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([400.0, 300.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "Markdown Viewer",
        native_options,
        Box::new(|cc| {
            Ok(Box::new(MarkdownViewerApp::new(cc, initial_file)))
        }),
    )
}
