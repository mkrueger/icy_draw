use super::{Event, Position, Tool};
use crate::{AnsiEditor, Message};
use eframe::egui;
use icy_engine::TextPane;
use icy_engine_egui::TerminalCalc;

#[derive(Default)]
pub struct PasteTool {
    start_offset: Position,
    drag_started: bool,
    drag_offset: Position,
    last_tool: usize,
    closed: bool,
}

impl PasteTool {
    fn is_paste_layer_selected(editor: &AnsiEditor, cur: Position) -> Option<bool> {
        if let Some(layer) = editor.buffer_view.lock().get_buffer().layers.last() {
            if layer.role.is_paste() {
                let pos = cur;
                if pos.x >= 0
                    && pos.y >= 0
                    && pos.x < layer.get_width()
                    && pos.y < layer.get_height()
                {
                    return Some(true);
                }
            }
            return Some(false);
        }
        None
    }

    pub(crate) fn new(selected_tool: usize) -> Self {
        Self {
            last_tool: selected_tool,
            ..Default::default()
        }
    }
}

impl Tool for PasteTool {
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage {
        &super::icons::MOVE_SVG
    }
    fn use_caret(&self) -> bool {
        false
    }

    fn is_visible(&self) -> bool {
        false
    }

    fn is_exclusive(&self) -> bool {
        true
    }

    fn use_selection(&self) -> bool {
        false
    }

    fn show_ui(
        &mut self,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        editor: &AnsiEditor,
    ) -> Option<Message> {
        if let Some(layer) = editor.buffer_view.lock().get_edit_state().get_cur_layer() {
            self.closed = !layer.role.is_paste();
        }

        if self.closed {
            return Some(Message::SelectTool(self.last_tool));
        }
        ui.label("Show fancy paste ui");
        None
    }

    fn handle_drag_begin(&mut self, editor: &mut AnsiEditor, pos: Position) -> Event {
        self.drag_started = false;
        if let Some(selected) = PasteTool::is_paste_layer_selected(editor, pos) {
            if !selected {
                let layer = editor.get_cur_layer_index();
                editor
                    .buffer_view
                    .lock()
                    .get_edit_state_mut()
                    .merge_layer_down(layer)
                    .unwrap();
                self.closed = true;
                return Event::None;
            }
        }

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
        editor: &mut AnsiEditor,
        cur: Position,
    ) -> egui::Response {
        if let Some(selected) = PasteTool::is_paste_layer_selected(editor, cur) {
            if selected {
                return response.on_hover_cursor(egui::CursorIcon::Move);
            }
            return response.on_hover_cursor(egui::CursorIcon::PointingHand);
        }
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
