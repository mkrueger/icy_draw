use super::{Event, Position, Tool, ToolUiResult};
use crate::AnsiEditor;
use eframe::egui;

pub struct MoveLayer {
    pub pos: Position,
}

impl Tool for MoveLayer {
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage {
        &super::icons::MOVE_SVG
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

    fn handle_drag_begin(
        &mut self,
        editor: &mut AnsiEditor,
        _start: Position,
        _cur: Position,
    ) -> Event {
        if let Some(layer) = editor.buffer_view.lock().buf.layers.get(editor.cur_layer) {
            self.pos = layer.get_offset();
        }
        Event::None
    }

    fn handle_drag(&mut self, editor: &mut AnsiEditor, start: Position, cur: Position) -> Event {
        if let Some(layer) = editor.buffer_view.lock().buf.layers.get_mut(editor.cur_layer) {
            layer.set_offset(self.pos + cur - start);
        }
        Event::None
    }
}
