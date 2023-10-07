#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
const VERSION: &str = env!("CARGO_PKG_VERSION");
mod model;
mod paint;
mod plugins;
mod ui;
mod util;

use log::LevelFilter;
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};

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
    use std::fs;

    use eframe::IconData;

    use crate::plugins::Plugin;
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1280., 841.)),
        multisampling: 0,
        renderer: eframe::Renderer::Glow,
        icon_data: Some(IconData::try_from_png_bytes(&include_bytes!("../build/linux/256x256.png")[..]).unwrap()),
        ..Default::default()
    };

    if let Ok(log_file) = Settings::get_log_file() {
        // delete log file when it is too big
        if let Ok(data) = fs::metadata(&log_file) {
            if data.len() > 1024 * 256 {
                fs::remove_file(&log_file).unwrap();
            }
        }

        let level = log::LevelFilter::Info;

        // Build a stderr logger.
        let stderr = ConsoleAppender::builder().target(Target::Stderr).build();

        // Logging to log file.
        let logfile = FileAppender::builder()
            // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
            .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
            .build(log_file)
            .unwrap();

        let config = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .appender(
                Appender::builder()
                    .filter(Box::new(ThresholdFilter::new(level)))
                    .build("stderr", Box::new(stderr)),
            )
            .build(Root::builder().appender("logfile").appender("stderr").build(LevelFilter::Info))
            .unwrap();

        // Use this to change log levels at runtime.
        // This means you can change the default log level to trace
        // if you are trying to debug an issue and need more logs on then turn it off
        // once you are done.
        let _handle = log4rs::init_config(config);
    } else {
        eprintln!("Failed to create log file");
    }

    if let Ok(settings_file) = Settings::get_settings_file() {
        if settings_file.exists() {
            if let Ok(settings) = Settings::load(&settings_file) {
                unsafe {
                    SETTINGS = settings;
                }
            }
        }
    }

    if let Ok(settings_file) = KeyBindings::get_keybindings_file() {
        if settings_file.exists() {
            if let Ok(key_bindings) = KeyBindings::load(&settings_file) {
                unsafe {
                    KEYBINDINGS = key_bindings;
                }
            }
        }
    }

    if let Ok(settings_file) = CharacterSets::get_character_sets_file() {
        if settings_file.exists() {
            if let Ok(character_sets) = CharacterSets::load(&settings_file) {
                unsafe {
                    CHARACTER_SETS = character_sets;
                }
            }
        }
    }

    if let Ok(settings_file) = MostRecentlyUsedFiles::get_mru_file() {
        if settings_file.exists() {
            if let Ok(character_sets) = MostRecentlyUsedFiles::load(&settings_file) {
                unsafe {
                    MRU_FILES = character_sets;
                }
            }
        }
    }

    unsafe {
        if KEYBINDINGS.key_bindings.is_empty() {
            KEYBINDINGS.key_bindings = Commands::default_keybindings();
        }

        if CHARACTER_SETS.character_sets.is_empty() {
            CHARACTER_SETS.character_sets.push(CharSetMapping::default());
        }
    }

    log::info!("Starting iCY DRAW {}", VERSION);
    Plugin::read_plugin_directory();
    if let Err(err) = eframe::run_native(&DEFAULT_TITLE, options, Box::new(|cc| Box::new(MainWindow::new(cc)))) {
        log::error!("Error returned by run_native: {}", err);
    }
    log::info!("shutting down.");
}
