pub mod ansi_editor;

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

mod char_table;
pub use char_table::*;

mod icons;
pub use icons::*;

mod new_file_dialog;
pub use new_file_dialog::*;

mod edit_sauce_dialog;
pub use edit_sauce_dialog::*;

mod layer_view;
pub use layer_view::*;

pub type TerminalResult<T> = Result<T, Box<dyn Error>>;
