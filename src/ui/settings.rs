use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs::{self, File},
    io::{self, BufReader, BufWriter},
    path::{Path, PathBuf},
};

use crate::TerminalResult;

const MAX_RECENT_FILES: usize = 10;

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    font_outline_style: usize,
    character_set: usize,

    custom_palette: IcePalette,

    recent_files: Vec<PathBuf>,
}

impl Settings {
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

    pub fn get_custom_palette() -> &'static mut IcePalette {
        unsafe { &mut SETTINGS.custom_palette }
    }

    pub fn set_custom_palette(pal: IcePalette) {
        unsafe {
            SETTINGS.custom_palette = pal;
        }
    }

    pub fn add_recent_file(file: &Path) {
        unsafe {
            if !file.exists() {
                return;
            }
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
                return Err(Box::new(IcyDrawError::ErrorCreatingDirectory(format!(
                    "{dir:?}"
                ))));
            }
            return Ok(dir);
        }
        Err(Box::new(IcyDrawError::ErrorCreatingDirectory(
            "font directory".to_string(),
        )))
    }

    pub(crate) fn get_tdf_diretory() -> TerminalResult<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "GitHub", "icy_draw") {
            let dir = proj_dirs.config_dir().join("data/tdf");

            if !dir.exists() && fs::create_dir_all(&dir).is_err() {
                return Err(Box::new(IcyDrawError::ErrorCreatingDirectory(format!(
                    "{dir:?}"
                ))));
            }
            return Ok(dir);
        }
        Err(Box::new(IcyDrawError::ErrorCreatingDirectory(
            "font directory".to_string(),
        )))
    }

    pub(crate) fn get_palettes_diretory() -> TerminalResult<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "GitHub", "icy_draw") {
            let dir = proj_dirs.config_dir().join("data/palettes");

            if !dir.exists() && fs::create_dir_all(&dir).is_err() {
                return Err(Box::new(IcyDrawError::ErrorCreatingDirectory(format!(
                    "{dir:?}"
                ))));
            }
            return Ok(dir);
        }
        Err(Box::new(IcyDrawError::ErrorCreatingDirectory(
            "font directory".to_string(),
        )))
    }

    pub(crate) fn get_auto_save_diretory() -> TerminalResult<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "GitHub", "icy_draw") {
            let dir = proj_dirs.config_dir().join("autosave");

            if !dir.exists() && fs::create_dir_all(&dir).is_err() {
                return Err(Box::new(IcyDrawError::ErrorCreatingDirectory(format!(
                    "{dir:?}"
                ))));
            }
            return Ok(dir);
        }
        Err(Box::new(IcyDrawError::ErrorCreatingDirectory(
            "font directory".to_string(),
        )))
    }

    pub(crate) fn get_log_file() -> TerminalResult<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "GitHub", "icy_draw") {
            let dir = proj_dirs.config_dir().join("icy_draw.log");
            return Ok(dir);
        }
        Err(Box::new(IcyDrawError::ErrorCreatingDirectory(
            "log_file".to_string(),
        )))
    }

    pub(crate) fn get_settings_file() -> TerminalResult<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "GitHub", "icy_draw") {
            let dir = proj_dirs.config_dir().join("settings.json");
            return Ok(dir);
        }
        Err(Box::new(IcyDrawError::ErrorCreatingDirectory(
            "log_file".to_string(),
        )))
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

pub static mut SETTINGS: Settings = Settings {
    font_outline_style: 0,
    custom_palette: IcePalette {
        title: String::new(),
        colors: Vec::new(),
    },
    character_set: 5,
    recent_files: Vec::new(),
};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct IceColor {
    pub name: Option<String>,
    pub color: (u8, u8, u8),
}

impl IceColor {
    pub fn get_rgb(&self) -> (u8, u8, u8) {
        self.color
    }

    pub(crate) fn get_name(&self) -> String {
        if let Some(name) = &self.name {
            name.clone()
        } else {
            self.get_rgb_text()
        }
    }

    pub(crate) fn get_rgb_text(&self) -> String {
        let (r, g, b) = self.get_rgb();
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> IceColor {
        IceColor {
            name: None,
            color: (r, g, b),
        }
    }

    pub fn set_name(&mut self, name: String) {
        if name.is_empty() {
            self.name = None;
        } else {
            self.name = Some(name);
        }
    }

    pub(crate) fn set_rgb(&mut self, r: u8, g: u8, b: u8) {
        self.color = (r, g, b);
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct IcePalette {
    pub title: String,
    pub colors: Vec<IceColor>,
}

impl IcePalette {
    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }

    pub fn push_rgb(&mut self, r: u8, g: u8, b: u8) {
        self.colors.push(IceColor::from_rgb(r, g, b));
    }
}

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
