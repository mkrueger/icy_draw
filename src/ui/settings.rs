use directories::ProjectDirs;
use eframe::{egui::Modifiers, epaint::Color32};
use icy_engine_egui::{BackgroundEffect, MarkerSettings, MonitorSettings};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs::{self, File},
    io::{self, BufReader, BufWriter},
    path::{Path, PathBuf},
};

use crate::{plugins::Plugin, TerminalResult};

const MAX_RECENT_FILES: usize = 10;

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    font_outline_style: usize,
    character_set: usize,

    pub show_layer_borders: bool,
    pub show_line_numbers: bool,

    pub monitor_settings: MonitorSettings,
    pub marker_settings: MarkerSettings,

    pub key_bindings: Vec<(String, eframe::egui::Key, Modifiers)>,

    recent_files: Vec<PathBuf>,

    pub character_sets: Vec<CharacterSet>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CharacterSet {
    pub font_name: String,
    pub table: Vec<Vec<char>>,
}

impl Default for CharacterSet {
    fn default() -> Self {
        let mut default_char_set = CharacterSet {
            font_name: "".to_string(),
            table: Vec::new(),
        };
        for i in crate::DEFAULT_CHAR_SET_TABLE {
            default_char_set
                .table
                .push(i.iter().fold(Vec::new(), |mut s, c| {
                    s.push(char::from_u32(*c as u32).unwrap());
                    s
                }));
        }
        default_char_set
    }
}

impl Settings {
    pub fn get_character_set_char(&self, _font_name: &str, ch: usize) -> char {
        let table_idx = 0;
        if table_idx >= self.character_sets.len() {
            return ' ';
        }
        let char_set = &self.character_sets[table_idx];
        if self.character_set >= char_set.table.len()
            || ch >= char_set.table[self.character_set].len()
        {
            return ' ';
        }
        char_set.table[self.character_set][ch]
    }

    pub fn set_character_set(character_set: usize) {
        unsafe {
            SETTINGS.character_set = character_set;
        }
    }

    pub fn get_character_set() -> usize {
        unsafe { SETTINGS.character_set }
    }

    pub fn set_font_outline_style(font_outline_style: usize) {
        unsafe {
            SETTINGS.font_outline_style = font_outline_style;
        }
    }

    pub fn get_font_outline_style() -> usize {
        unsafe { SETTINGS.font_outline_style }
    }

    pub fn add_recent_file(file: &Path) {
        unsafe {
            let file = file.to_path_buf();
            for i in 0..SETTINGS.recent_files.len() {
                if SETTINGS.recent_files[i] == file {
                    SETTINGS.recent_files.remove(i);
                    break;
                }
            }

            SETTINGS.recent_files.push(file);
            while SETTINGS.recent_files.len() > MAX_RECENT_FILES {
                SETTINGS.recent_files.remove(0);
            }
        }
    }

    pub fn clear_recent_files() {
        unsafe {
            SETTINGS.recent_files.clear();
        }
    }

    pub(crate) fn get_font_diretory() -> TerminalResult<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "GitHub", "icy_draw") {
            let dir = proj_dirs.config_dir().join("data/fonts");
            if !dir.exists() && fs::create_dir_all(&dir).is_err() {
                return Err(IcyDrawError::ErrorCreatingDirectory(format!("{dir:?}")).into());
            }
            return Ok(dir);
        }
        Err(IcyDrawError::ErrorCreatingDirectory("font directory".to_string()).into())
    }

    pub(crate) fn get_tdf_diretory() -> TerminalResult<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "GitHub", "icy_draw") {
            let dir = proj_dirs.config_dir().join("data/tdf");

            if !dir.exists() && fs::create_dir_all(&dir).is_err() {
                return Err(IcyDrawError::ErrorCreatingDirectory(format!("{dir:?}")).into());
            }
            return Ok(dir);
        }
        Err(IcyDrawError::ErrorCreatingDirectory("font directory".to_string()).into())
    }

    pub(crate) fn get_palettes_diretory() -> TerminalResult<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "GitHub", "icy_draw") {
            let dir = proj_dirs.config_dir().join("data/palettes");

            if !dir.exists() && fs::create_dir_all(&dir).is_err() {
                return Err(IcyDrawError::ErrorCreatingDirectory(format!("{dir:?}")).into());
            }
            return Ok(dir);
        }
        Err(IcyDrawError::ErrorCreatingDirectory("font directory".to_string()).into())
    }

    pub(crate) fn get_auto_save_diretory() -> TerminalResult<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "GitHub", "icy_draw") {
            let dir = proj_dirs.config_dir().join("autosave");

            if !dir.exists() && fs::create_dir_all(&dir).is_err() {
                return Err(IcyDrawError::ErrorCreatingDirectory(format!("{dir:?}")).into());
            }
            return Ok(dir);
        }
        Err(IcyDrawError::ErrorCreatingDirectory("font directory".to_string()).into())
    }

    pub(crate) fn get_log_file() -> TerminalResult<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "GitHub", "icy_draw") {
            let dir = proj_dirs.config_dir().join("icy_draw.log");
            return Ok(dir);
        }
        Err(IcyDrawError::ErrorCreatingDirectory("log_file".to_string()).into())
    }

    pub(crate) fn get_settings_file() -> TerminalResult<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "GitHub", "icy_draw") {
            let dir = proj_dirs.config_dir().join("settings.json");
            return Ok(dir);
        }
        Err(IcyDrawError::ErrorCreatingDirectory("log_file".to_string()).into())
    }

    pub(crate) fn get_plugin_directory() -> TerminalResult<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "GitHub", "icy_draw") {
            let dir = proj_dirs.config_dir().join("data/plugins");

            if !dir.exists() {
                if fs::create_dir_all(&dir).is_err() {
                    return Err(IcyDrawError::ErrorCreatingDirectory(format!("{dir:?}")).into());
                }
                fs::write(
                    dir.join("elite-writing.lua"),
                    include_bytes!("../plugins/elite-writing.lua.txt"),
                )?;
            }
            return Ok(dir);
        }
        Err(IcyDrawError::ErrorCreatingDirectory("font directory".to_string()).into())
    }

    pub(crate) fn load(path: &PathBuf) -> io::Result<Settings> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let u = serde_json::from_reader(reader)?;

        // Return the `User`.
        Ok(u)
    }

    pub(crate) fn save() -> io::Result<()> {
        let Ok(path) = Settings::get_settings_file() else {
            return Ok(());
        };

        unsafe {
            let file = File::create(path)?;
            let reader = BufWriter::new(file);

            serde_json::to_writer_pretty(reader, &SETTINGS)?;

            Ok(())
        }
    }

    pub(crate) fn get_recent_files(&mut self) -> &Vec<PathBuf> {
        self.recent_files.retain(|p| p.exists());
        &self.recent_files
    }
}

pub static mut PLUGINS: Vec<Plugin> = Vec::new();

pub static mut SETTINGS: Settings = Settings {
    font_outline_style: 0,
    character_set: 5,
    show_layer_borders: true,
    show_line_numbers: false,
    recent_files: Vec::new(),
    key_bindings: Vec::new(),
    character_sets: Vec::new(),
    monitor_settings: MonitorSettings {
        use_filter: false,
        monitor_type: 0,
        gamma: 50.,
        contrast: 50.,
        saturation: 50.,
        brightness: 30.,
        light: 40.,
        blur: 30.,
        curvature: 10.,
        scanlines: 10.,
        background_effect: BackgroundEffect::Checkers,
        selection_fg: Color32::from_rgb(0xAB, 0x00, 0xAB),
        selection_bg: Color32::from_rgb(0xAB, 0xAB, 0xAB),
    },
    marker_settings: MarkerSettings {
        reference_image_alpha: 0.2,
        raster_alpha: 0.2,
        raster_color: Color32::from_rgb(0xAB, 0xAB, 0xAB),
        guide_alpha: 0.2,
        guide_color: Color32::from_rgb(0xAB, 0xAB, 0xAB),

        border_color: Color32::from_rgb(64, 69, 74),
    },
};

#[derive(Debug, Clone)]
pub enum IcyDrawError {
    Error(String),
    ErrorCreatingDirectory(String),
}

impl std::fmt::Display for IcyDrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IcyDrawError::Error(err) => write!(f, "Error: {err}"),
            IcyDrawError::ErrorCreatingDirectory(dir) => {
                write!(f, "Error creating directory: {dir}")
            }
        }
    }
}

impl Error for IcyDrawError {
    fn description(&self) -> &str {
        "use std::display"
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
