use eframe::egui;
use i18n_embed_fl::fl;
use icy_engine::AttributedChar;

use crate::{AnsiEditor, Message};

use super::{Event, Position, Tool};
#[derive(Default)]
pub struct PipetteTool {
    cur_char: Option<AttributedChar>,
}

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
        ui: &mut egui::Ui,
        _editor_opt: Option<&AnsiEditor>,
    ) -> Option<Message> {
        if let Some(ch) = self.cur_char {
            ui.label(fl!(
                crate::LANGUAGE_LOADER,
                "pipette_tool_char_code",
                code = (ch.ch as u32)
            ));
            ui.label(fl!(
                crate::LANGUAGE_LOADER,
                "pipette_tool_foreground",
                fg = ch.attribute.get_foreground()
            ));
            ui.label(fl!(
                crate::LANGUAGE_LOADER,
                "pipette_tool_background",
                bg = ch.attribute.get_background()
            ));
        }
        None
    }

    fn handle_hover(
        &mut self,
        _ui: &egui::Ui,
        response: egui::Response,
        editor: &mut AnsiEditor,
        cur: Position,
        _cur_abs: Position,
    ) -> egui::Response {
        self.cur_char = Some(editor.get_char(cur));
        response.on_hover_cursor(egui::CursorIcon::Crosshair)
    }

    fn handle_click(
        &mut self,
        editor: &mut AnsiEditor,
        button: i32,
        pos: Position,
        _pos_abs: Position,
        _response: &egui::Response,
    ) -> Event {
        if button == 1 {
            let ch = editor.get_char(pos);
            editor.set_caret_attribute(ch.attribute);
        }
        Event::None
    }
}
