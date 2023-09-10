#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![allow(clippy::arc_with_non_send_sync)]
use eframe::egui;
const VERSION: &str = env!("CARGO_PKG_VERSION");
mod model;
mod ui;
mod util;

use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use rust_embed::RustEmbed;
pub use ui::*;

lazy_static::lazy_static! {
    static ref DEFAULT_TITLE: String = format!("iCY DRAW {}", crate::VERSION);
}

#[derive(RustEmbed)]
#[folder = "i18n"] // path to the compiled localization resources
struct Localizations;

use once_cell::sync::Lazy;
static LANGUAGE_LOADER: Lazy<FluentLanguageLoader> = Lazy::new(|| {
    let loader = fluent_language_loader!();
    let requested_languages = DesktopLanguageRequester::requested_languages();
    let _result = i18n_embed::select(&loader, &Localizations, &requested_languages);
    loader
});

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
    )
    .unwrap();
}
