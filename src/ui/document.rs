use std::sync::{Arc, Mutex};

use eframe::{egui};
use crate::TerminalResult;

use super::ansi_editor::BufferView;

pub trait Document {
    fn get_title(&self) -> String;
    fn is_dirty(&self) -> bool;

    fn save(&mut self, file_name: &str) -> TerminalResult<()>;

    fn show_ui(&mut self, ui: &mut egui::Ui);

    fn destroy(&self, gl: &glow::Context);
    
    fn get_buffer_view(&self) -> Option<Arc<Mutex<BufferView>>>;
}