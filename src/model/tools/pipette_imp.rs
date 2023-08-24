use eframe::egui;

use crate::AnsiEditor;

use super::{Event, Position, Tool, ToolUiResult};
pub struct PipetteTool {}

impl Tool for PipetteTool {
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage {
        &super::icons::DROPPER_SVG
    }
    fn use_caret(&self) -> bool {
        false
    }
    fn use_selection(&self) -> bool {
        false
    }
    fn show_ui(
        &mut self,
        _ctx: &egui::Context,
        _ui: &mut egui::Ui,
        _buffer_opt: &mut AnsiEditor,
    ) -> ToolUiResult {
        ToolUiResult::default()
    }

    fn handle_click(&mut self, editor: &mut AnsiEditor, button: i32, pos: Position) -> Event {
        if button == 1 {
            let ch = editor.get_char(pos);
            editor.set_caret_attribute(ch.attribute);
        }
        Event::None
    }
}
