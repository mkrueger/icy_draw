#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

use eframe::egui;
const VERSION: &str = env!("CARGO_PKG_VERSION");
mod model;
mod ui;

pub use ui::*;

lazy_static::lazy_static! {
    static ref DEFAULT_TITLE: String = format!("iCY DRAW {}", crate::VERSION);
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1280., 841.)),
        multisampling: 0,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        &DEFAULT_TITLE,
        options,
        Box::new(|cc| Box::new(MainWindow::new(cc))),
    );
}