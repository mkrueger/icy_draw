use std::{cell::RefCell, collections::HashSet, rc::Rc};

use eframe::egui;
use i18n_embed_fl::fl;
use icy_engine::{AttributedChar, Size, TextPane};

use crate::{AnsiEditor, Message};

use super::{brush_imp::draw_glyph, Position, Tool};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FillType {
    Character,
    Colorize,
}

pub struct FillTool {
    use_fore: bool,
    use_back: bool,

    char_code: std::rc::Rc<std::cell::RefCell<char>>,
    fill_type: FillType,
    use_exact_matching: bool,
}

impl FillTool {
    pub fn new() -> Self {
        Self {
            use_fore: true,
            use_back: true,
            char_code: Rc::new(RefCell::new('\u{00B0}')),
            fill_type: crate::model::fill_imp::FillType::Character,
            use_exact_matching: false,
        }
    }
}
#[allow(clippy::struct_excessive_bools)]
struct FillOperation {
    fill_type: FillType,
    use_fore: bool,
    use_back: bool,
    use_exact_matching: bool,

    size: Size,
    pub offset: Position,
    use_selection: bool,
    base_char: AttributedChar,
    new_char: AttributedChar,
    visited: HashSet<Position>,
}

impl FillOperation {
    pub fn new(
        fill_tool: &FillTool,
        editor: &AnsiEditor,
        base_char: AttributedChar,
        new_ch: AttributedChar,
    ) -> Self {
        let size = editor
            .buffer_view
            .lock()
            .get_edit_state()
            .get_cur_layer()
            .unwrap()
            .get_size();
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

        Self {
            size,
            fill_type: fill_tool.fill_type,
            use_fore: fill_tool.use_fore,
            use_back: fill_tool.use_back,
            use_selection,
            base_char,
            offset,
            new_char: new_ch,
            use_exact_matching: fill_tool.use_exact_matching,
            visited: HashSet::new(),
        }
    }

    pub fn fill(&mut self, editor: &mut AnsiEditor, pos: Position) {
        if pos.x < 0
            || pos.y < 0
            || pos.x >= self.size.width
            || pos.y >= self.size.height
            || !self.visited.insert(pos)
        {
            return;
        }

        if !self.use_selection
            || editor
                .buffer_view
                .lock()
                .get_edit_state()
                .get_is_selected(pos + self.offset)
        {
            let cur_char = editor
                .buffer_view
                .lock()
                .get_edit_state()
                .get_cur_layer()
                .unwrap()
                .get_char(pos);

            let mut repl_ch = cur_char;

            match self.fill_type {
                FillType::Character => {
                    if self.use_exact_matching && cur_char != self.base_char
                        || !self.use_exact_matching && cur_char.ch != self.base_char.ch
                    {
                        return;
                    }
                    repl_ch.ch = self.new_char.ch;
                    repl_ch.set_font_page(self.new_char.get_font_page());
                }
                FillType::Colorize => {
                    if self.use_exact_matching && cur_char != self.base_char
                        || !self.use_exact_matching
                            && cur_char.attribute != self.base_char.attribute
                    {
                        return;
                    }
                }
            }
            if self.use_fore {
                repl_ch
                    .attribute
                    .set_foreground(self.new_char.attribute.get_foreground());
                repl_ch
                    .attribute
                    .set_is_bold(self.new_char.attribute.is_bold());
            }

            if self.use_back {
                repl_ch
                    .attribute
                    .set_background(self.new_char.attribute.get_background());
            }

            repl_ch.set_font_page(
                editor
                    .buffer_view
                    .lock()
                    .get_caret()
                    .get_attribute()
                    .get_font_page(),
            );
            repl_ch.attribute.attr &= !icy_engine::attribute::INVISIBLE;
            editor.set_char(pos, repl_ch);
        }

        self.fill(editor, pos + Position::new(-1, 0));
        self.fill(editor, pos + Position::new(1, 0));
        self.fill(editor, pos + Position::new(0, -1));
        self.fill(editor, pos + Position::new(0, 1));
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

        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.fill_type,
                FillType::Character,
                fl!(crate::LANGUAGE_LOADER, "tool-character"),
            );
            if let Some(editor) = editor_opt {
                result = draw_glyph(ui, editor, &self.char_code);
            }
        });
        ui.radio_value(
            &mut self.fill_type,
            FillType::Colorize,
            fl!(crate::LANGUAGE_LOADER, "tool-colorize"),
        );

        ui.checkbox(
            &mut self.use_exact_matching,
            fl!(crate::LANGUAGE_LOADER, "tool-fill-exact_match_label"),
        );

        result
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
            if editor.get_cur_layer_index() >= editor.buffer_view.lock().get_buffer().layers.len() {
                return None;
            }
            let attr = editor.buffer_view.lock().get_caret().get_attribute();
            let ch = editor
                .buffer_view
                .lock()
                .get_edit_state()
                .get_cur_layer()
                .unwrap()
                .get_char(pos);
            if self.use_back || self.use_fore || matches!(self.fill_type, FillType::Character) {
                let _undo =
                    editor.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-bucket-fill"));
                let mut op = FillOperation::new(
                    self,
                    editor,
                    ch,
                    AttributedChar::new(*self.char_code.borrow(), attr),
                );
                op.fill(editor, pos);
            }
        }
        None
    }
}
