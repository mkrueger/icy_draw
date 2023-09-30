use eframe::egui;
use i18n_embed_fl::fl;
use icy_engine_egui::TerminalCalc;

use crate::{
    paint::{draw_line, BrushMode, ColorMode},
    AnsiEditor, Message,
};

use super::{Position, Tool};

pub struct LineTool {
    draw_mode: BrushMode,
    color_mode: ColorMode,

    pub char_code: std::rc::Rc<std::cell::RefCell<char>>,

    pub old_pos: Position,
}

impl Default for LineTool {
    fn default() -> Self {
        Self {
            draw_mode: BrushMode::HalfBlock,
            color_mode: crate::paint::ColorMode::Both,
            char_code: std::rc::Rc::new(std::cell::RefCell::new('\u{00B0}')),
            old_pos: Position::default(),
        }
    }
}

/*

impl LineTool {
    pub fn get_new_horiz_char(editor: &mut Editor, new_char: u16, to_left: bool) -> usize {
        if new_char == editor.get_outline_char_code(VERTICAL_CHAR).unwrap() {
            if to_left {
                VERT_RIGHT_CHAR
            } else {
                VERT_LEFT_CHAR
            }
        } else if new_char == editor.get_outline_char_code(CORNER_LOWER_RIGHT).unwrap()
            || new_char == editor.get_outline_char_code(CORNER_LOWER_LEFT).unwrap()
        {
            HORIZ_UP_CHAR
        } else if new_char == editor.get_outline_char_code(CORNER_UPPER_RIGHT).unwrap()
            || new_char == editor.get_outline_char_code(CORNER_UPPER_LEFT).unwrap()
        {
            HORIZ_DOWN_CHAR
        } else {
            HORIZONTAL_CHAR
        }
    }

    pub fn get_old_horiz_char(
        &self,
        editor: &mut Editor,
        old_char: u16,
        to_left: bool,
    ) -> Option<u16> {
        let pos = editor.get_caret_position();
        if old_char == editor.get_outline_char_code(VERTICAL_CHAR).unwrap() {
            match self.old_pos.y.cmp(&pos.y) {
                std::cmp::Ordering::Greater => Some(
                    editor
                        .get_outline_char_code(if to_left {
                            CORNER_UPPER_RIGHT
                        } else {
                            CORNER_UPPER_LEFT
                        })
                        .unwrap(),
                ),
                std::cmp::Ordering::Less => Some(
                    editor
                        .get_outline_char_code(if to_left {
                            CORNER_LOWER_RIGHT
                        } else {
                            CORNER_LOWER_LEFT
                        })
                        .unwrap(),
                ),
                std::cmp::Ordering::Equal => None,
            }
        } else if old_char == editor.get_outline_char_code(VERT_LEFT_CHAR).unwrap()
            || old_char == editor.get_outline_char_code(VERT_RIGHT_CHAR).unwrap()
        {
            let cur = editor.get_cur_outline();
            if cur < 4 {
                let ck = Editor::get_outline_char_code_from(4, cur);
                Some(ck.unwrap())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_new_vert_char(editor: &mut Editor, new_char: u16, to_left: bool) -> usize {
        if new_char == editor.get_outline_char_code(HORIZONTAL_CHAR).unwrap() {
            if to_left {
                HORIZ_DOWN_CHAR
            } else {
                HORIZ_UP_CHAR
            }
        } else if new_char == editor.get_outline_char_code(CORNER_LOWER_RIGHT).unwrap()
            || new_char == editor.get_outline_char_code(CORNER_LOWER_LEFT).unwrap()
        {
            HORIZ_UP_CHAR
        } else if new_char == editor.get_outline_char_code(CORNER_UPPER_RIGHT).unwrap()
            || new_char == editor.get_outline_char_code(CORNER_UPPER_LEFT).unwrap()
        {
            VERT_RIGHT_CHAR
        } else {
            VERTICAL_CHAR
        }
    }

    pub fn get_old_vert_char(
        &self,
        editor: &mut Editor,
        old_char: u16,
        to_left: bool,
    ) -> Option<u16> {
        let pos = editor.get_caret_position();
        if old_char == editor.get_outline_char_code(HORIZONTAL_CHAR).unwrap() {
            match self.old_pos.x.cmp(&pos.x) {
                std::cmp::Ordering::Greater => Some(
                    editor
                        .get_outline_char_code(if to_left {
                            CORNER_LOWER_RIGHT
                        } else {
                            CORNER_UPPER_RIGHT
                        })
                        .unwrap(),
                ),
                std::cmp::Ordering::Less => Some(
                    editor
                        .get_outline_char_code(if to_left {
                            CORNER_LOWER_LEFT
                        } else {
                            CORNER_UPPER_LEFT
                        })
                        .unwrap(),
                ),
                std::cmp::Ordering::Equal => None,
            }
        } else if old_char == editor.get_outline_char_code(HORIZ_UP_CHAR).unwrap()
            || old_char == editor.get_outline_char_code(HORIZ_DOWN_CHAR).unwrap()
        {
            if editor.get_cur_outline() < 4 {
                Some(Editor::get_outline_char_code_from(4, editor.get_cur_outline()).unwrap())
            } else {
                None
            }
        } else {
            None
        }
    }
}
*/

// block tools:
// copy/moxe
// fill, delete
impl Tool for LineTool {
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage {
        &super::icons::LINE_SVG
    }
    fn use_selection(&self) -> bool {
        false
    }

    fn show_ui(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, editor_opt: Option<&AnsiEditor>) -> Option<Message> {
        self.color_mode.show_ui(ui);
        self.draw_mode.show_ui(ui, editor_opt, self.char_code.clone(), false)
    }

    fn handle_click(&mut self, editor: &mut AnsiEditor, button: i32, pos: Position, _pos_abs: Position, _response: &egui::Response) -> Option<Message> {
        if button == 1 {
            editor.set_caret_position(pos);
        }
        None
    }

    fn handle_hover(&mut self, _ui: &egui::Ui, response: egui::Response, _editor: &mut AnsiEditor, _cur: Position, _cur_abs: Position) -> egui::Response {
        response.on_hover_cursor(egui::CursorIcon::Crosshair)
    }

    fn handle_drag(&mut self, _ui: &egui::Ui, response: egui::Response, editor: &mut AnsiEditor, _calc: &TerminalCalc) -> egui::Response {
        editor.clear_overlay_layer();
        draw_line(
            &mut editor.buffer_view.lock(),
            editor.drag_pos.start_half_block,
            editor.half_block_click_pos,
            self.draw_mode.clone(),
            self.color_mode,
        );
        response
    }

    fn handle_drag_end(&mut self, editor: &mut AnsiEditor) -> Option<Message> {
        if editor.drag_pos.start == editor.drag_pos.cur {
            editor.buffer_view.lock().get_buffer_mut().remove_overlay();
        } else {
            editor.join_overlay(fl!(crate::LANGUAGE_LOADER, "undo-line"));
        }
        None
    }
}
