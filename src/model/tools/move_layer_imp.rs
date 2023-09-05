use super::{Event, Position, Tool};
use crate::{AnsiEditor, Message};
use eframe::egui;
use icy_engine_egui::TerminalCalc;

#[derive(Default)]
pub struct MoveLayer {
    start_offset: Position,
    drag_started: bool,
    drag_offset: Position,
}

impl MoveLayer {}

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

    fn handle_drag_begin(&mut self, editor: &mut AnsiEditor, _pos: Position) -> Event {
        self.drag_started = false;

        if let Some(layer) = editor
            .buffer_view
            .lock()
            .get_edit_state_mut()
            .get_cur_layer_mut()
        {
            self.start_offset = layer.get_offset();
            self.drag_started = true;
        }
        Event::None
    }

    fn handle_drag(
        &mut self,
        _ui: &egui::Ui,
        response: egui::Response,
        editor: &mut AnsiEditor,
        calc: &TerminalCalc,
        start: Position,
        _cur: Position,
    ) -> egui::Response {
        if !self.drag_started {
            return response;
        }
        if let Some(layer) = editor
            .buffer_view
            .lock()
            .get_edit_state_mut()
            .get_cur_layer_mut()
        {
            let mouse_pos = response.interact_pointer_pos().unwrap();
            let click_pos = calc.calc_click_pos(mouse_pos);
            let cp = Position::new(click_pos.x as i32, click_pos.y as i32) - self.start_offset;
            self.drag_offset = self.start_offset + cp - start;
            layer.set_preview_offset(Some(self.drag_offset));
        }
        response.on_hover_cursor(egui::CursorIcon::Grabbing)
    }

    fn handle_hover(
        &mut self,
        _ui: &egui::Ui,
        response: egui::Response,
        _editor: &mut AnsiEditor,
        _cur: Position,
    ) -> egui::Response {
        response.on_hover_cursor(egui::CursorIcon::Move)
    }

    fn handle_drag_end(
        &mut self,
        editor: &mut AnsiEditor,
        _start: Position,
        _cur: Position,
    ) -> Event {
        if !self.drag_started {
            return Event::None;
        }
        editor
            .buffer_view
            .lock()
            .get_edit_state_mut()
            .move_layer(self.drag_offset)
            .unwrap();
        Event::None
    }
}
