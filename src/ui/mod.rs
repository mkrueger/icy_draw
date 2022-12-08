mod buffer_view;
mod terminal_window;
mod main_window;
use std::error::Error;

pub use main_window::*;

mod document;
pub use document::*;

mod font_editor;
pub use font_editor::*;

pub type TerminalResult<T> = Result<T, Box<dyn Error>>;
