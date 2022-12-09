mod ansi_editor;
mod main_window;
use std::error::Error;

pub use main_window::*;

mod document;
pub use document::*;

mod font_editor;
pub use font_editor::*;

mod palette_editor;
pub use palette_editor::*;

mod tool_switcher;
pub use tool_switcher::*;

pub type TerminalResult<T> = Result<T, Box<dyn Error>>;
