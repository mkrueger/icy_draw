use eframe::{egui, epaint::Vec2};

use crate::{model::Tool, AnsiEditor, TerminalResult};

pub trait Document {
    fn get_title(&self) -> String;
    fn is_dirty(&self) -> bool;

    fn get_id(&self) -> usize;

    fn save(&mut self, file_name: &str) -> TerminalResult<()>;

    fn show_ui(
        &mut self,
        ui: &mut egui::Ui,
        cur_tool: &mut Box<dyn Tool>,
        options: &DocumentOptions,
    );

    fn destroy(&self, gl: &glow::Context);

    fn get_ansi_editor_mut(&mut self) -> Option<&mut AnsiEditor>;
    fn get_ansi_editor(&self) -> Option<&AnsiEditor>;

}

pub struct DocumentOptions {
    pub scale: Vec2,
}
