use eframe::egui;

use crate::{AnsiEditor, Message};

use super::{Position, Tool};
pub struct FlipTool {}

impl Tool for FlipTool {
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage {
        &super::icons::FILL_SVG
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
        _editor_opt: Option<&AnsiEditor>,
    ) -> Option<Message> {
        None
    }

    fn handle_hover(
        &mut self,
        _ui: &egui::Ui,
        response: egui::Response,
        _editor: &mut AnsiEditor,
        _cur: Position,
        _cur_abs: Position,
    ) -> egui::Response {
        response.on_hover_cursor(egui::CursorIcon::Crosshair)
    }
    fn handle_click(
        &mut self,
        editor: &mut AnsiEditor,
        button: i32,
        pos: Position,
        _pos_abs: Position,
        _response: &egui::Response,
    ) -> Option<Message> {
        if button == 1 {
            let mut ch = editor.get_char(pos);

            if ch.ch as u8 == 222 {
                ch.ch = '\u{00DD}';
            } else if ch.ch as u8 == 221 {
                ch.ch = '\u{00DB}';
            } else {
                ch.ch = '\u{00DE}';
            }

            editor.set_char(pos, ch);
        }
        None
    }
} //   [176, 177, 178, 219, 223, 220, 221, 222, 254, 250 ],
