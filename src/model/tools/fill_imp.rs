use std::collections::HashSet;

use eframe::egui;
use i18n_embed_fl::fl;
use icy_engine::{AttributedChar, TextAttribute};

use crate::{AnsiEditor, Message};

use super::{brush_imp::draw_glyph, Event, Position, Tool};

#[derive(PartialEq, Eq)]
pub enum FillType {
    Character,
    Colorize,
}

pub struct FillTool {
    pub use_fore: bool,
    pub use_back: bool,

    pub attr: TextAttribute,
    pub char_code: std::rc::Rc<std::cell::RefCell<char>>,
    pub fill_type: FillType,
}

impl FillTool {
    fn fill(
        &self,
        editor: &mut AnsiEditor,
        visited: &mut HashSet<Position>,
        pos: Position,
        old_ch: AttributedChar,
        new_ch: AttributedChar,
    ) {
        if pos.x >= editor.buffer_view.lock().get_buffer().get_width() as i32
            || pos.y >= editor.buffer_view.lock().get_buffer().get_height() as i32
            || !visited.insert(pos)
        {
            return;
        }

        let cur_char = editor.buffer_view.lock().get_buffer().get_char(pos);
        if matches!(self.fill_type, FillType::Character) && self.use_fore && self.use_back {
            if cur_char != old_ch || cur_char == new_ch {
                return;
            }
        } else if self.use_fore && self.use_back {
            if cur_char.attribute != old_ch.attribute || cur_char.attribute == new_ch.attribute {
                return;
            }
        } else if matches!(self.fill_type, FillType::Character) && self.use_fore {
            if cur_char.ch != old_ch.ch
                && cur_char.attribute.get_foreground() != old_ch.attribute.get_foreground()
                || cur_char.ch == new_ch.ch
                    && cur_char.attribute.get_foreground() == new_ch.attribute.get_foreground()
            {
                return;
            }
        } else if matches!(self.fill_type, FillType::Character) && self.use_back {
            if cur_char.ch != old_ch.ch
                && cur_char.attribute.get_background() != old_ch.attribute.get_background()
                || cur_char.ch == new_ch.ch
                    && cur_char.attribute.get_background() == new_ch.attribute.get_background()
            {
                return;
            }
        } else if matches!(self.fill_type, FillType::Character) {
            if cur_char.ch != old_ch.ch || cur_char.ch == new_ch.ch {
                return;
            }
        } else if self.use_fore {
            if cur_char.attribute.get_foreground() != old_ch.attribute.get_foreground()
                || cur_char.attribute.get_foreground() == new_ch.attribute.get_foreground()
            {
                return;
            }
        } else if self.use_back {
            if cur_char.attribute.get_background() != old_ch.attribute.get_background()
                || cur_char.attribute.get_background() == new_ch.attribute.get_background()
            {
                return;
            }
        } else {
            panic!("should never happen!");
        }
        let mut repl_ch = cur_char;
        if matches!(self.fill_type, FillType::Character) {
            repl_ch.ch = new_ch.ch;
        }
        if self.use_fore {
            repl_ch
                .attribute
                .set_foreground(new_ch.attribute.get_foreground());
        }
        if self.use_back {
            repl_ch
                .attribute
                .set_background(new_ch.attribute.get_background());
        }

        repl_ch.set_font_page(
            editor
                .buffer_view
                .lock()
                .get_caret()
                .get_attribute()
                .get_font_page(),
        );
        editor.set_char(pos, repl_ch);

        if pos.x != 0 {
            self.fill(editor, visited, pos + Position::new(-1, 0), old_ch, new_ch);
        }
        self.fill(editor, visited, pos + Position::new(1, 0), old_ch, new_ch);

        if pos.y != 0 {
            self.fill(editor, visited, pos + Position::new(0, -1), old_ch, new_ch);
        }
        self.fill(editor, visited, pos + Position::new(0, 1), old_ch, new_ch);
    }
}

// Fill with
// Attribute, Fore/Back
// Character
// Both
impl Tool for FillTool {
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage {
        &super::icons::FILL_SVG
    }
    fn use_caret(&self) -> bool {
        false
    }
    fn show_ui(
        &mut self,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        editor: &AnsiEditor,
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

        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.fill_type,
                FillType::Character,
                fl!(crate::LANGUAGE_LOADER, "tool-character"),
            );

            result = draw_glyph(ui, editor, &self.char_code);
        });
        ui.radio_value(
            &mut self.fill_type,
            FillType::Colorize,
            fl!(crate::LANGUAGE_LOADER, "tool-colorize"),
        );
        result
    }

    fn handle_click(&mut self, editor: &mut AnsiEditor, button: i32, pos: Position) -> Event {
        if button == 1 {
            if editor.get_cur_layer() >= editor.buffer_view.lock().get_buffer().layers.len() {
                return Event::None;
            }
            let attr = editor.buffer_view.lock().get_caret().get_attribute();
            let ch = editor.buffer_view.lock().get_buffer().get_char(pos);
            if self.use_back || self.use_fore || matches!(self.fill_type, FillType::Character) {
                editor.begin_atomic_undo();
                let mut visited = HashSet::new();
                self.fill(
                    editor,
                    &mut visited,
                    pos,
                    ch,
                    AttributedChar::new(*self.char_code.borrow(), attr),
                );
                editor.end_atomic_undo();
            }
        }
        Event::None
    }
}
