use super::{Event, Position, Tool};
use crate::{to_message, AnsiEditor, Message};
use eframe::egui;
use i18n_embed_fl::fl;
use icy_engine_egui::TerminalCalc;

#[derive(Default)]
pub struct MoveLayer {
    start_offset: Position,
    drag_started: bool,
    drag_offset: Position,
}

impl MoveLayer {}

impl Tool for MoveLayer {
    fn get_icon_name(&self) -> &egui::Image<'static> {
        &super::icons::MOVE_SVG
    }
    fn use_caret(&self) -> bool {
        false
    }
    fn use_selection(&self) -> bool {
        false
    }
    fn show_ui(&mut self, _ctx: &egui::Context, _ui: &mut egui::Ui, _editor_opt: Option<&AnsiEditor>) -> Option<Message> {
        None
    }

    fn handle_drag_begin(&mut self, editor: &mut AnsiEditor, _response: &egui::Response) -> Event {
        self.drag_started = false;

        if let Some(layer) = editor.buffer_view.lock().get_edit_state_mut().get_cur_layer_mut() {
            self.start_offset = layer.get_offset();
            self.drag_started = true;
        }
        Event::None
    }

    fn handle_drag(&mut self, _ui: &egui::Ui, response: egui::Response, editor: &mut AnsiEditor, _calc: &TerminalCalc) -> egui::Response {
        if !self.drag_started {
            return response;
        }
        if let Some(layer) = editor.buffer_view.lock().get_edit_state_mut().get_cur_layer_mut() {
            self.drag_offset = self.start_offset + editor.drag_pos.cur_abs - editor.drag_pos.start_abs;
            layer.set_preview_offset(Some(self.drag_offset));
        }
        response.on_hover_cursor(egui::CursorIcon::Grabbing)
    }

    fn get_toolbar_location_text(&self, editor: &AnsiEditor) -> String {
        get_layer_offset_text(editor)
    }

    fn handle_hover(&mut self, _ui: &egui::Ui, response: egui::Response, _editor: &mut AnsiEditor, _cur: Position, _cur_abs: Position) -> egui::Response {
        response.on_hover_cursor(egui::CursorIcon::Move)
    }

    fn handle_drag_end(&mut self, editor: &mut AnsiEditor) -> Option<Message> {
        if !self.drag_started {
            return None;
        }
        to_message(editor.buffer_view.lock().get_edit_state_mut().move_layer(self.drag_offset))
    }
}

pub(super) fn get_layer_offset_text(editor: &AnsiEditor) -> String {
    if let Some(layer) = editor.buffer_view.lock().get_edit_state_mut().get_cur_layer() {
        let pos = layer.get_offset();
        fl!(crate::LANGUAGE_LOADER, "toolbar-layer_offset", line = pos.y, column = pos.x)
    } else {
        String::new()
    }
}
