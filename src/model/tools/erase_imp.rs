use eframe::egui;
use i18n_embed_fl::fl;
use icy_engine::{editor::AtomicUndoGuard, AttributedChar, TextAttribute};
use icy_engine_egui::TerminalCalc;

use crate::{AnsiEditor, Event, Message};

use super::{Position, Tool};

#[derive(PartialEq, Eq)]
pub enum EraseType {
    Shade,
    Solid,
}

pub struct EraseTool {
    size: i32,
    brush_type: EraseType,
    undo_op: Option<AtomicUndoGuard>,
}

impl Default for EraseTool {
    fn default() -> Self {
        Self {
            size: 3,
            brush_type: crate::model::erase_imp::EraseType::Solid,
            undo_op: None,
        }
    }
}

impl EraseTool {
    fn eraser(&self, editor: &mut AnsiEditor, pos: Position) {
        let mid = Position::new(-(self.size / 2), -(self.size / 2));

        let center = pos + mid;
        let gradient = ['\u{00DB}', '\u{00B2}', '\u{00B1}', '\u{00B0}', ' '];
        let use_selection = editor
            .buffer_view
            .lock()
            .get_edit_state()
            .is_something_selected();
        let offset = if let Some(layer) = editor.buffer_view.lock().get_edit_state().get_cur_layer()
        {
            layer.get_offset()
        } else {
            Position::default()
        };

        for y in 0..self.size {
            for x in 0..self.size {
                let pos = center + Position::new(x, y);
                if use_selection
                    && !editor
                        .buffer_view
                        .lock()
                        .get_edit_state()
                        .get_is_selected(pos + offset)
                {
                    continue;
                }
                match self.brush_type {
                    EraseType::Shade => {
                        let ch = editor.get_char_from_cur_layer(pos);

                        let mut attribute = ch.attribute;

                        let mut char_code = gradient[0];
                        let mut found = false;
                        if ch.ch == gradient[gradient.len() - 1] {
                            char_code = gradient[gradient.len() - 1];
                            attribute = TextAttribute::default();
                            found = true;
                        } else {
                            for i in 0..gradient.len() - 1 {
                                if ch.ch == gradient[i] {
                                    char_code = gradient[i + 1];
                                    found = true;
                                    break;
                                }
                            }
                        }

                        if found {
                            editor.set_char(pos, AttributedChar::new(char_code, attribute));
                        }
                    }
                    EraseType::Solid => {
                        editor.set_char(pos, AttributedChar::new(' ', TextAttribute::default()));
                    }
                }
            }
        }
    }
}

impl Tool for EraseTool {
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage {
        &super::icons::ERASER_SVG
    }

    fn use_caret(&self) -> bool {
        false
    }

    fn show_ui(
        &mut self,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        _editor_opt: Option<&AnsiEditor>,
    ) -> Option<Message> {
        ui.horizontal(|ui| {
            ui.label(fl!(crate::LANGUAGE_LOADER, "tool-size-label"));
            ui.add(
                egui::DragValue::new(&mut self.size)
                    .clamp_range(1..=20)
                    .speed(1),
            );
        });
        ui.radio_value(
            &mut self.brush_type,
            EraseType::Solid,
            fl!(crate::LANGUAGE_LOADER, "tool-solid"),
        );
        ui.radio_value(
            &mut self.brush_type,
            EraseType::Shade,
            fl!(crate::LANGUAGE_LOADER, "tool-shade"),
        );
        None
    }

    fn handle_hover(
        &mut self,
        _ui: &egui::Ui,
        response: egui::Response,
        editor: &mut AnsiEditor,
        _cur: Position,
        cur_abs: Position,
    ) -> egui::Response {
        let mid = Position::new(-(self.size / 2), -(self.size / 2));
        for y in 0..self.size {
            for x in 0..self.size {
                let pos = cur_abs + Position::new(x, y) + mid;
                editor
                    .buffer_view
                    .lock()
                    .get_edit_state_mut()
                    .get_tool_overlay_mask_mut()
                    .set_is_selected(pos, true);
            }
        }

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
            let _undo = editor.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-eraser"));

            self.eraser(editor, pos);
        }
        None
    }

    fn handle_drag(
        &mut self,
        _ui: &egui::Ui,
        response: egui::Response,
        editor: &mut AnsiEditor,
        _calc: &TerminalCalc,
    ) -> egui::Response {
        self.eraser(editor, editor.drag_pos.cur);
        response
    }

    fn handle_drag_begin(&mut self, editor: &mut AnsiEditor, _response: &egui::Response) -> Event {
        self.undo_op = Some(editor.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-eraser")));
        self.eraser(editor, editor.drag_pos.cur);
        Event::None
    }

    fn handle_drag_end(&mut self, _editor: &mut AnsiEditor) -> Event {
        self.undo_op = None;
        Event::None
    }
}
