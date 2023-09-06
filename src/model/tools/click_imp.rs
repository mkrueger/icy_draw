use eframe::egui;
use egui_extras::RetainedImage;
use icy_engine::Rectangle;
use icy_engine_egui::TerminalCalc;

use crate::{AnsiEditor, Message};

use super::{Event, Position, Tool};

#[derive(Default)]
pub struct ClickTool {}

impl Tool for ClickTool {
    fn get_icon_name(&self) -> &'static RetainedImage {
        &super::icons::CURSOR_SVG
    }

    fn show_ui(
        &mut self,
        _ctx: &egui::Context,
        _ui: &mut egui::Ui,
        _buffer_opt: &AnsiEditor,
    ) -> Option<Message> {
        None
    }

    fn handle_click(&mut self, editor: &mut AnsiEditor, button: i32, pos: Position) -> Event {
        if button == 1 {
            editor.set_caret_position(pos);
            editor.buffer_view.lock().clear_selection();
        }
        Event::None
    }

    fn handle_drag(
        &mut self,
        _ui: &egui::Ui,
        response: egui::Response,
        editor: &mut AnsiEditor,
        _calc: &TerminalCalc
    ) -> egui::Response {

        if editor.drag_pos.start == editor.drag_pos.cur {
            editor.buffer_view.lock().clear_selection();
        } else {
            editor.buffer_view.lock().set_selection(Rectangle::from(
                editor.drag_pos.start_abs.x.min(editor.drag_pos.cur_abs.x),
                editor.drag_pos.start_abs.y.min(editor.drag_pos.cur_abs.y),
                (editor.drag_pos.cur_abs.x - editor.drag_pos.start_abs.x).abs(),
                (editor.drag_pos.cur_abs.y - editor.drag_pos.start_abs.y).abs(),
            ));
        }
        response
    }

    fn handle_hover(
        &mut self,
        ui: &egui::Ui,
        response: egui::Response,
        _editor: &mut AnsiEditor,
        _cur: Position,
    ) -> egui::Response {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Text);
        response
    }

    fn handle_drag_end(
        &mut self,
        editor: &mut AnsiEditor,
    ) -> Event {
        let mut cur = editor.drag_pos.cur;
        if editor.drag_pos.start < cur {
            cur += Position::new(1, 1);
        }

        if editor.drag_pos.start == cur {
            editor.buffer_view.lock().clear_selection();
        }

        Event::None
    }
}
