use super::{Event, Position, Tool};
use crate::{AnsiEditor, Message};
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
        _buffer_opt: &AnsiEditor,
    ) -> Option<Message> {
        None
    }

    fn handle_drag_begin(
        &mut self,
        editor: &mut AnsiEditor,
        _start: Position,
        _cur: Position,
    ) -> Event {
        let cur_layer = editor.get_cur_layer();
        if let Some(layer) = editor
            .buffer_view
            .lock()
            .get_buffer()
            .layers
            .get(cur_layer)
        {
            self.pos = layer.get_offset();
        }
        Event::None
    }

    fn handle_drag(&mut self, editor: &mut AnsiEditor, start: Position, cur: Position) -> Event {
        let cur_layer = editor.get_cur_layer();
        if let Some(layer) = editor
            .buffer_view
            .lock()
            .get_buffer_mut()
            .layers
            .get_mut(cur_layer)
        {
            layer.set_offset(self.pos + cur - start);
        }
        Event::None
    }
}
