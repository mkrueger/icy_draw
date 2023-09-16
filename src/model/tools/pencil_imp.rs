use eframe::egui::{self};
use egui_extras::RetainedImage;
use i18n_embed_fl::fl;
use icy_engine::{editor::AtomicUndoGuard, AttributedChar, Rectangle};
use icy_engine_egui::TerminalCalc;

use crate::{model::ScanLines, AnsiEditor, Event, Message};

use super::{brush_imp::draw_glyph, line_imp::set_half_block, Position, Tool};

#[derive(PartialEq, Eq)]
pub enum PencilType {
    HalfBlock,
    Shade,
    Solid,
    Color,
}

pub struct PencilTool {
    pub use_fore: bool,
    pub use_back: bool,
    pub char_code: std::rc::Rc<std::cell::RefCell<char>>,
    pub undo_op: Option<AtomicUndoGuard>,

    pub last_pos: Position,
    pub brush_type: PencilType,
}

impl PencilTool {
    fn paint_brush(&self, editor: &mut AnsiEditor, pos: Position) {
        let center = pos;
        let gradient = ['\u{00B0}', '\u{00B1}', '\u{00B2}', '\u{00DB}'];
        let offset = if let Some(layer) = editor.buffer_view.lock().get_edit_state().get_cur_layer()
        {
            layer.get_offset()
        } else {
            Position::default()
        };

        let use_selection = editor
            .buffer_view
            .lock()
            .get_edit_state()
            .is_something_selected();

        if use_selection
            && !editor
                .buffer_view
                .lock()
                .get_edit_state()
                .get_is_selected(pos + offset)
        {
            return;
        }

        match self.brush_type {
            PencilType::HalfBlock => {
                let mut lines = ScanLines::new(1);
                lines.add_line(
                    Position::new(self.last_pos.x, self.last_pos.y * 2),
                    Position::new(pos.x, pos.y * 2),
                );
                let draw = move |rect: Rectangle| {
                    let col = editor
                        .buffer_view
                        .lock()
                        .get_caret()
                        .get_attribute()
                        .get_foreground();
                    for y in 0..rect.size.height {
                        for x in 0..rect.size.width {
                            set_half_block(
                                editor,
                                Position::new(rect.start.x + x, rect.start.y + y),
                                col,
                            );
                        }
                    }
                };
                lines.fill(draw);
            }
            PencilType::Shade => {
                let ch = editor.get_char_from_cur_layer(center);
                let attribute = editor.buffer_view.lock().get_caret().get_attribute();

                let mut char_code = gradient[0];
                if ch.ch == gradient[gradient.len() - 1] {
                    char_code = gradient[gradient.len() - 1];
                } else {
                    for i in 0..gradient.len() - 1 {
                        if ch.ch == gradient[i] {
                            char_code = gradient[i + 1];
                            break;
                        }
                    }
                }
                editor.set_char(center, AttributedChar::new(char_code, attribute));
            }
            PencilType::Solid => {
                let attribute = editor.buffer_view.lock().get_caret().get_attribute();
                editor.set_char(
                    center,
                    AttributedChar::new(*self.char_code.borrow(), attribute),
                );
            }
            PencilType::Color => {
                let ch = editor.get_char_from_cur_layer(center);
                let mut attribute = ch.attribute;
                if self.use_fore {
                    attribute.set_foreground(
                        editor
                            .buffer_view
                            .lock()
                            .get_caret()
                            .get_attribute()
                            .get_foreground(),
                    );
                }
                if self.use_back {
                    attribute.set_background(
                        editor
                            .buffer_view
                            .lock()
                            .get_caret()
                            .get_attribute()
                            .get_background(),
                    );
                }
                editor.set_char(center, AttributedChar::new(ch.ch, attribute));
            }
        }
    }
}

impl Tool for PencilTool {
    fn get_icon_name(&self) -> &'static RetainedImage {
        &super::icons::PENCIL_SVG
    }

    fn use_caret(&self) -> bool {
        false
    }

    fn show_ui(
        &mut self,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        editor_opt: Option<&AnsiEditor>,
    ) -> Option<Message> {
        let mut result = None;
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.use_fore, fl!(crate::LANGUAGE_LOADER, "tool-fg"))
                    .clicked()
                {
                    self.use_fore = !self.use_fore;
                }
                if ui
                    .selectable_label(self.use_back, fl!(crate::LANGUAGE_LOADER, "tool-bg"))
                    .clicked()
                {
                    self.use_back = !self.use_back;
                }
            });
        });
        ui.radio_value(
            &mut self.brush_type,
            PencilType::HalfBlock,
            fl!(crate::LANGUAGE_LOADER, "tool-half-block"),
        );
        ui.radio_value(
            &mut self.brush_type,
            PencilType::Shade,
            fl!(crate::LANGUAGE_LOADER, "tool-shade"),
        );
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.brush_type,
                PencilType::Solid,
                fl!(crate::LANGUAGE_LOADER, "tool-character"),
            );
            if let Some(editor) = editor_opt {
                result = draw_glyph(ui, editor, &self.char_code);
            }
        });
        ui.radio_value(
            &mut self.brush_type,
            PencilType::Color,
            fl!(crate::LANGUAGE_LOADER, "tool-colorize"),
        );
        result
    }

    fn handle_click(
        &mut self,
        editor: &mut AnsiEditor,
        button: i32,
        pos: Position,
        _pos_abs: Position,
        _response: &egui::Response,
    ) -> super::Event {
        if button == 1 {
            self.last_pos = pos;
            let _op: AtomicUndoGuard =
                editor.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-pencil"));
            editor.clear_overlay_layer();
            self.paint_brush(editor, pos);
            editor.join_overlay(fl!(crate::LANGUAGE_LOADER, "undo-pencil"));
        }
        super::Event::None
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

    fn handle_drag(
        &mut self,
        _ui: &egui::Ui,
        response: egui::Response,
        editor: &mut AnsiEditor,
        _calc: &TerminalCalc,
    ) -> egui::Response {
        self.paint_brush(editor, editor.drag_pos.cur);
        self.last_pos = editor.drag_pos.cur;

        response
    }

    fn handle_drag_begin(&mut self, editor: &mut AnsiEditor, _response: &egui::Response) -> Event {
        self.undo_op = Some(editor.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-pencil")));
        self.last_pos = editor.drag_pos.cur;
        editor.clear_overlay_layer();

        Event::None
    }

    fn handle_drag_end(&mut self, editor: &mut AnsiEditor) -> Event {
        editor.join_overlay(fl!(crate::LANGUAGE_LOADER, "undo-pencil"));
        self.undo_op = None;
        Event::None
    }
}
