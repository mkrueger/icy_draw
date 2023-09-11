use std::{error::Error, fs, path::PathBuf};

use directories::ProjectDirs;
use icy_engine::Color;

use crate::TerminalResult;

pub struct Settings {
    font_outline_style: usize,
    character_set: usize,

    custom_palette: IcePalette,
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
}

pub static mut SETTINGS: Settings = Settings {
    font_outline_style: 0,
    custom_palette: IcePalette {
        title: String::new(),
        colors: Vec::new(),
    },
    character_set: 5,
};

#[derive(Default)]
pub struct IceColor {
    pub name: Option<String>,
    pub color: Color,
}

impl IceColor {
    pub fn get_rgb(&self) -> (u8, u8, u8) {
        self.color.get_rgb()
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
            color: Color::new(r, g, b),
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
        self.color = Color::new(r, g, b);
    }
}

#[derive(Default)]
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
