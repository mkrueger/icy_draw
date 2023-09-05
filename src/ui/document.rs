use eframe::{egui, epaint::Vec2};
use icy_engine::{editor::UndoState, EngineResult};

use crate::{model::Tool, AnsiEditor, Message, TerminalResult};

pub trait ClipboardHandler {
    fn can_cut(&self) -> bool {
        false
    }
    fn cut(&mut self) -> EngineResult<()> {
        Ok(())
    }

    fn can_copy(&self) -> bool {
        false
    }
    fn copy(&mut self) -> EngineResult<()> {
        Ok(())
    }

    fn can_paste(&self) -> bool {
        false
    }
    fn paste(&mut self) -> EngineResult<()> {
        Ok(())
    }
}

pub trait Document: UndoState + ClipboardHandler {
    fn get_title(&self) -> String;
    fn is_dirty(&self) -> bool;

    fn save(&mut self, file_name: &str) -> TerminalResult<()>;

    fn show_ui(
        &mut self,
        ui: &mut egui::Ui,
        cur_tool: &mut Box<dyn Tool>,
        selected_tool: usize,
        options: &DocumentOptions,
    ) -> Option<Message>;

    fn destroy(&self, gl: &glow::Context);

    fn get_ansi_editor_mut(&mut self) -> Option<&mut AnsiEditor>;
    fn get_ansi_editor(&self) -> Option<&AnsiEditor>;
}

pub struct DocumentOptions {
    pub scale: Vec2,
}
