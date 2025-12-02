mod app;
mod canvas;
mod history;
mod svg_doc;

use app::SvgEditorApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "SVG Editor",
        options,
        Box::new(|cc| Ok(Box::new(SvgEditorApp::new(cc)))),
    )
}
