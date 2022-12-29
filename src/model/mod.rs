#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::too_many_lines,
    clippy::cast_lossless,
    clippy::cast_precision_loss
)]

mod editor;
pub use editor::*;

mod tool;
pub use tool::*;
