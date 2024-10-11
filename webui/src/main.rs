#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod framework;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            // .with_decorations(false)
            .with_resizable(true)
            .with_inner_size([400.0, 200.0])
            .with_always_on_top()
            .with_transparent(true),
        ..Default::default()
    };
    eframe::run_native(
        "WdAssistant", // unused title
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<framework::WdApp>::default())
        }),
    )
}
