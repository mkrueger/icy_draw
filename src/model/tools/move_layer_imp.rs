use super::{Event, Position, Tool, ToolUiResult};
use crate::ansi_editor::BufferView;
use eframe::egui;
use std::sync::{Arc, Mutex};
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
        _buffer_opt: Option<std::sync::Arc<std::sync::Mutex<BufferView>>>,
    ) -> ToolUiResult {
        ToolUiResult::default()
    }

    fn handle_drag_begin(
        &mut self,
        buffer_view: Arc<Mutex<BufferView>>,
        _start: Position,
        _cur: Position,
    ) -> Event {
        if let Some(layer) = buffer_view.lock().editor.get_cur_layer() {
            self.pos = layer.get_offset();
        }
        Event::None
    }

    fn handle_drag(
        &mut self,
        buffer_view: Arc<Mutex<BufferView>>,
        start: Position,
        cur: Position,
    ) -> Event {
        if let Some(layer) = buffer_view.lock().editor.get_cur_layer_mut() {
            layer.set_offset(self.pos + cur - start);
        }
        Event::None
    }
}
