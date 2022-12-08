use eframe::egui;
use crate::TerminalResult;

pub trait Document {
    fn get_title(&self) -> String;
    fn is_dirty(&self) -> bool;

    fn save(&mut self, file_name: &str) -> TerminalResult<()>;

    fn show_ui(&mut self, ui: &mut egui::Ui);
}