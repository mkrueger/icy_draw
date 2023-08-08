use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use eframe::egui;
use i18n_embed_fl::fl;
use icy_engine::{AttributedChar, TextAttribute};

use crate::ansi_editor::BufferView;

use super::{brush_imp::draw_glyph, Editor, Event, Position, Tool, ToolUiResult};

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
    pub font_page: usize,
    pub fill_type: FillType,
}

impl FillTool {
    fn fill(
        &self,
        editor: &mut Editor,
        visited: &mut HashSet<Position>,
        pos: Position,
        opt_old_ch: Option<AttributedChar>,
        new_ch: AttributedChar,
    ) {
        if !editor.point_is_valid(pos) || !visited.insert(pos) {
            return;
        }

        let cur_char = editor.buf.get_char(pos).unwrap_or_default();
        if let Some(old_ch) = opt_old_ch {
            if matches!(self.fill_type, FillType::Character) && self.use_fore && self.use_back {
                if cur_char != old_ch || cur_char == new_ch {
                    return;
                }
            } else if self.use_fore && self.use_back {
                if cur_char.attribute != old_ch.attribute || cur_char.attribute == new_ch.attribute
                {
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

        editor.set_char(pos, Some(repl_ch));

        self.fill(
            editor,
            visited,
            pos + Position::new(-1, 0),
            opt_old_ch,
            new_ch,
        );
        self.fill(
            editor,
            visited,
            pos + Position::new(1, 0),
            opt_old_ch,
            new_ch,
        );
        self.fill(
            editor,
            visited,
            pos + Position::new(0, -1),
            opt_old_ch,
            new_ch,
        );
        self.fill(
            editor,
            visited,
            pos + Position::new(0, 1),
            opt_old_ch,
            new_ch,
        );
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
        buffer_opt: Option<std::sync::Arc<std::sync::Mutex<crate::ui::ansi_editor::BufferView>>>,
    ) -> ToolUiResult {
        let mut result = ToolUiResult::default();
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

            if let Some(b) = &buffer_opt {
                draw_glyph(
                    ui,
                    b,
                    &mut result,
                    &self.char_code,
                    self.font_page,
                );
            }
        });
        ui.radio_value(
            &mut self.fill_type,
            FillType::Colorize,
            fl!(crate::LANGUAGE_LOADER, "tool-colorize"),
        );
        result
    }

    fn handle_click(
        &mut self,
        buffer_view: Arc<Mutex<BufferView>>,
        button: i32,
        pos: Position,
    ) -> Event {
        if button == 1 {
            let editor = &mut buffer_view.lock().unwrap().editor;
            if editor.cur_layer >= editor.buf.layers.len() as i32 {
                return Event::None;
            }
            let attr = editor.caret.get_attribute();
            let ch = editor.buf.get_char(pos);
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
