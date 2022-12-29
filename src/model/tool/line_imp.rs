use std::sync::{Arc, Mutex};

use eframe::egui;
use i18n_embed_fl::fl;
use icy_engine::{AttributedChar, Rectangle, TextAttribute};

use crate::{ansi_editor::BufferView, model::ScanLines};

use super::{
    brush_imp::draw_glyph, plot_point, DrawMode, Editor, Event, Plottable, Position, Tool,
};

pub struct LineTool {
    pub draw_mode: DrawMode,

    pub use_fore: bool,
    pub use_back: bool,
    pub attr: TextAttribute,
    pub char_code: char,
    pub font_page: usize,

    pub old_pos: Position,
}

impl Plottable for LineTool {
    fn get_draw_mode(&self) -> DrawMode {
        self.draw_mode
    }

    fn get_use_fore(&self) -> bool {
        self.use_fore
    }
    fn get_use_back(&self) -> bool {
        self.use_back
    }
    fn get_char_code(&self) -> char {
        self.char_code
    }
}

const CORNER_UPPER_LEFT: i32 = 0;
const CORNER_UPPER_RIGHT: i32 = 1;
const CORNER_LOWER_LEFT: i32 = 2;
const CORNER_LOWER_RIGHT: i32 = 3;

const HORIZONTAL_CHAR: i32 = 4;
const VERTICAL_CHAR: i32 = 5;

const VERT_RIGHT_CHAR: i32 = 6;
const VERT_LEFT_CHAR: i32 = 7;

const HORIZ_UP_CHAR: i32 = 8;
const HORIZ_DOWN_CHAR: i32 = 9;

impl LineTool {
    pub fn get_new_horiz_char(editor: &mut Editor, new_char: u16, to_left: bool) -> i32 {
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

    pub fn get_new_vert_char(editor: &mut Editor, new_char: u16, to_left: bool) -> i32 {
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

    fn show_ui(
        &mut self,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        buffer_opt: Option<std::sync::Arc<std::sync::Mutex<crate::ui::ansi_editor::BufferView>>>,
    ) {
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
            &mut self.draw_mode,
            DrawMode::Line,
            fl!(crate::LANGUAGE_LOADER, "tool-line"),
        );
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.draw_mode,
                DrawMode::Char,
                fl!(crate::LANGUAGE_LOADER, "tool-character"),
            );

            if let Some(b) = &buffer_opt {
                ui.add(draw_glyph(b.clone(), self.char_code, self.font_page));
            }
        });
        ui.radio_value(
            &mut self.draw_mode,
            DrawMode::Shade,
            fl!(crate::LANGUAGE_LOADER, "tool-shade"),
        );
        ui.radio_value(
            &mut self.draw_mode,
            DrawMode::Colorize,
            fl!(crate::LANGUAGE_LOADER, "tool-colorize"),
        );
    }
    /*
    fn handle_key(
        &mut self,
        editor: Rc<RefCell<Editor>>,
        key: MKey,
        _key_code: MKeyCode,
        modifier: MModifiers,
    ) -> Event {
        let mut e = editor;
        let old_pos = e.get_caret_position();
        match key {
            MKey::Down => {
                e.set_caret(old_pos.x, old_pos.y + 1);
            }
            MKey::Up => {
                e.set_caret(old_pos.x, old_pos.y - 1);
            }
            MKey::Left => {
                e.set_caret(old_pos.x - 1, old_pos.y);
            }
            MKey::Right => {
                e.set_caret(old_pos.x + 1, old_pos.y);
            }

            _ => {
                if modifier.is_shift() || modifier.is_control() {
                    match key {
                        MKey::F1 => {
                            handle_outline_insertion(&mut e, modifier, 0);
                        }
                        MKey::F2 => {
                            handle_outline_insertion(&mut e, modifier, 1);
                        }
                        MKey::F3 => {
                            handle_outline_insertion(&mut e, modifier, 2);
                        }
                        MKey::F4 => {
                            handle_outline_insertion(&mut e, modifier, 3);
                        }
                        MKey::F5 => {
                            handle_outline_insertion(&mut e, modifier, 4);
                        }
                        MKey::F6 => {
                            handle_outline_insertion(&mut e, modifier, 5);
                        }
                        MKey::F7 => {
                            handle_outline_insertion(&mut e, modifier, 6);
                        }
                        MKey::F8 => {
                            handle_outline_insertion(&mut e, modifier, 7);
                        }
                        MKey::F9 => {
                            handle_outline_insertion(&mut e, modifier, 8);
                        }
                        MKey::F10 => {
                            handle_outline_insertion(&mut e, modifier, 9);
                        }
                        _ => {}
                    }
                }
            }
        }

        let new_pos = e.get_caret_position();
        let new_char = e.get_char_from_cur_layer(new_pos).unwrap_or_default();
        let old_char = e.get_char_from_cur_layer(old_pos).unwrap_or_default();

        let b = (new_pos.x - old_pos.x).signum();
        let a = (new_pos.y - old_pos.y).signum();
        if a == 1 || a == -1 {
            let c = LineTool::get_new_vert_char(&e, new_char.char_code, a == -1 );
            let char_code = e.get_outline_char_code(c).unwrap();
            let attribute = e.caret.get_attribute();
            e.set_char(
                new_pos,
                Some(crate::model::DosChar {
                    char_code,
                    attribute,
                }),
            );

            if old_char.is_transparent() {
                let char_code = e.get_outline_char_code(HORIZONTAL_CHAR).unwrap();
                e.set_char(
                    old_pos,
                    Some(crate::model::DosChar {
                        char_code,
                        attribute,
                    }),
                );
            } else if let Some(char_code) = self.get_old_vert_char(&e, old_char.char_code, a == -1) {
                e.set_char(
                    old_pos,
                    Some(crate::model::DosChar {
                        char_code,
                        attribute,
                    }),
                );
            }
        }


        if b == 1 || b == -1 { // horizontal movement
            let c = LineTool::get_new_horiz_char(&e, new_char.char_code, b == -1 );
            let char_code = e.get_outline_char_code(c).unwrap();
            let attribute = e.caret.get_attribute();
            e.set_char(
                new_pos,
                Some(crate::model::DosChar {
                    char_code,
                    attribute,
                }),
            );

            if old_char.is_transparent() {
                let char_code = e.get_outline_char_code(VERTICAL_CHAR).unwrap();
                e.set_char(
                    old_pos,
                    Some(crate::model::DosChar {
                        char_code,
                        attribute,
                    }),
                );
            } else if let Some(char_code) = self.get_old_horiz_char(&e, old_char.char_code, b == -1) {
                e.set_char(
                    old_pos,
                    Some(crate::model::DosChar {
                        char_code,
                        attribute,
                    }),
                );
            }
        }

        self.old_pos = old_pos;
        Event::None
    }*/

    fn handle_click(
        &mut self,
        buffer_view: Arc<Mutex<BufferView>>,
        button: i32,
        pos: Position,
    ) -> Event {
        if button == 1 {
            let editor = &mut buffer_view.lock().unwrap().editor;
            editor.set_caret_position(pos);
        }
        Event::None
    }

    fn handle_drag(
        &mut self,
        buffer_view: Arc<Mutex<BufferView>>,
        start: Position,
        cur: Position,
    ) -> Event {
        if let Some(layer) = buffer_view.lock().unwrap().editor.get_overlay_layer() {
            layer.clear();
        }

        let mut lines = ScanLines::new(1);
        if self.draw_mode == DrawMode::Line {
            lines.add_line(
                Position::new(start.x, start.y * 2),
                Position::new(cur.x, cur.y * 2),
            );
            let col = buffer_view
                .lock()
                .unwrap()
                .editor
                .caret
                .get_attribute()
                .get_foreground();
            let buffer_view = buffer_view.clone();
            let draw = move |rect: Rectangle| {
                let editor = &mut buffer_view.lock().unwrap().editor;
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
        } else {
            lines.add_line(start, cur);
            let buffer_view = buffer_view.clone();
            let draw = move |rect: Rectangle| {
                let editor = &mut buffer_view.lock().unwrap().editor;
                for y in 0..rect.size.height {
                    for x in 0..rect.size.width {
                        plot_point(
                            editor,
                            self,
                            Position::new(rect.start.x + x, rect.start.y + y),
                        );
                    }
                }
            };
            lines.fill(draw);
        }

        Event::None
    }

    fn handle_drag_end(
        &mut self,
        buffer_view: Arc<Mutex<BufferView>>,
        start: Position,
        cur: Position,
    ) -> Event {
        let editor = &mut buffer_view.lock().unwrap().editor;
        if start == cur {
            editor.buf.remove_overlay();
        } else {
            editor.join_overlay();
        }
        Event::None
    }
}

fn get_half_block(
    editor: &mut Editor,
    pos: Position,
) -> (
    Position,
    i32,
    bool,
    bool,
    u32,
    u32,
    u32,
    u32,
    bool,
    u32,
    u32,
) {
    let text_y = pos.y / 2;
    let is_top = pos.y % 2 == 0;
    let block = editor
        .get_char(Position::new(pos.x, text_y))
        .unwrap_or_default();

    let mut upper_block_color = 0;
    let mut lower_block_color = 0;
    let mut left_block_color = 0;
    let mut right_block_color = 0;
    let mut is_blocky = false;
    let mut is_vertically_blocky = false;
    match block.ch as u8 {
        0 | 32 | 255 => {
            upper_block_color = block.attribute.get_background();
            lower_block_color = block.attribute.get_background();
            is_blocky = true;
        }
        220 => {
            upper_block_color = block.attribute.get_background();
            lower_block_color = block.attribute.get_foreground();
            is_blocky = true;
        }
        223 => {
            upper_block_color = block.attribute.get_foreground();
            lower_block_color = block.attribute.get_background();
            is_blocky = true;
        }
        219 => {
            upper_block_color = block.attribute.get_foreground();
            lower_block_color = block.attribute.get_foreground();
            is_blocky = true;
        }
        221 => {
            left_block_color = block.attribute.get_foreground();
            right_block_color = block.attribute.get_background();
            is_vertically_blocky = true;
        }
        222 => {
            left_block_color = block.attribute.get_background();
            right_block_color = block.attribute.get_foreground();
            is_vertically_blocky = true;
        }
        _ => {
            if block.attribute.get_foreground() == block.attribute.get_background() {
                is_blocky = true;
                upper_block_color = block.attribute.get_foreground();
                lower_block_color = block.attribute.get_foreground();
            } else {
                is_blocky = false;
            }
        }
    }
    (
        pos,
        text_y,
        is_blocky,
        is_vertically_blocky,
        upper_block_color,
        lower_block_color,
        left_block_color,
        right_block_color,
        is_top,
        block.attribute.get_foreground(),
        block.attribute.get_background(),
    )
}

pub fn set_half_block(editor: &mut Editor, pos: Position, col: u32) {
    let w = editor.buf.get_buffer_width() as i32;
    let h = editor.buf.get_real_buffer_height() as i32;

    if pos.x < 0 || pos.x >= w || pos.y < 0 || pos.y >= h * 2 {
        return;
    }

    let (
        _,
        text_y,
        is_blocky,
        _,
        upper_block_color,
        lower_block_color,
        _,
        _,
        is_top,
        _,
        block_back,
    ) = get_half_block(editor, pos);

    let pos = Position::new(pos.x, text_y);
    if is_blocky {
        if (is_top && lower_block_color == col) || (!is_top && upper_block_color == col) {
            if let Some(layer) = editor.get_overlay_layer() {
                layer.set_char(
                    pos,
                    Some(AttributedChar::new('\u{00DB}', TextAttribute::new(col, 0))),
                );
            }
        } else if is_top {
            if let Some(layer) = editor.get_overlay_layer() {
                layer.set_char(
                    pos,
                    Some(AttributedChar::new(
                        '\u{00DF}',
                        TextAttribute::new(col, lower_block_color),
                    )),
                );
            }
        } else if let Some(layer) = editor.get_overlay_layer() {
            layer.set_char(
                pos,
                Some(AttributedChar::new(
                    '\u{00DC}',
                    TextAttribute::new(col, upper_block_color),
                )),
            );
        }
    } else if is_top {
        if let Some(layer) = editor.get_overlay_layer() {
            layer.set_char(
                pos,
                Some(AttributedChar::new(
                    '\u{00DF}',
                    TextAttribute::new(col, block_back),
                )),
            );
        }
    } else if let Some(layer) = editor.get_overlay_layer() {
        layer.set_char(
            pos,
            Some(AttributedChar::new(
                '\u{00DC}',
                TextAttribute::new(col, block_back),
            )),
        );
    }
    optimize_block(editor, Position::new(pos.x, text_y));
}

fn optimize_block(editor: &mut Editor, pos: Position) {
    let block = if let Some(layer) = editor.get_overlay_layer() {
        layer.get_char(pos).unwrap_or_default()
    } else {
        AttributedChar::default()
    };

    if block.attribute.get_foreground() == 0 {
        if block.attribute.get_background() == 0 || block.ch == '\u{00DB}' {
            if let Some(layer) = editor.get_overlay_layer() {
                layer.set_char(pos, Some(AttributedChar::default()));
            }
        } else {
            match block.ch as u8 {
                220 => {
                    if let Some(layer) = editor.get_overlay_layer() {
                        layer.set_char(
                            pos,
                            Some(AttributedChar::new(
                                '\u{00DF}',
                                TextAttribute::new(
                                    block.attribute.get_background(),
                                    block.attribute.get_foreground(),
                                ),
                            )),
                        );
                    }
                }
                223 => {
                    if let Some(layer) = editor.get_overlay_layer() {
                        layer.set_char(
                            pos,
                            Some(AttributedChar::new(
                                '\u{00DC}',
                                TextAttribute::new(
                                    block.attribute.get_background(),
                                    block.attribute.get_foreground(),
                                ),
                            )),
                        );
                    }
                }
                _ => {}
            }
        }
    } else if block.attribute.get_foreground() < 8 && block.attribute.get_background() >= 8 {
        let (pos, _, is_blocky, is_vertically_blocky, _, _, _, _, _, _, _) =
            get_half_block(editor, pos);

        if is_blocky {
            match block.ch as u8 {
                220 => {
                    if let Some(layer) = editor.get_overlay_layer() {
                        layer.set_char(
                            pos,
                            Some(AttributedChar::new(
                                '\u{00DF}',
                                TextAttribute::new(
                                    block.attribute.get_background(),
                                    block.attribute.get_foreground(),
                                ),
                            )),
                        );
                    }
                }
                223 => {
                    if let Some(layer) = editor.get_overlay_layer() {
                        layer.set_char(
                            pos,
                            Some(AttributedChar::new(
                                '\u{00DC}',
                                TextAttribute::new(
                                    block.attribute.get_background(),
                                    block.attribute.get_foreground(),
                                ),
                            )),
                        );
                    }
                }
                _ => {}
            }
        } else if is_vertically_blocky {
            match block.ch as u8 {
                221 => {
                    if let Some(layer) = editor.get_overlay_layer() {
                        layer.set_char(
                            pos,
                            Some(AttributedChar::new(
                                '\u{00DE}',
                                TextAttribute::new(
                                    block.attribute.get_background(),
                                    block.attribute.get_foreground(),
                                ),
                            )),
                        );
                    }
                }
                222 => {
                    if let Some(layer) = editor.get_overlay_layer() {
                        layer.set_char(
                            pos,
                            Some(AttributedChar::new(
                                '\u{00DD}',
                                TextAttribute::new(
                                    block.attribute.get_background(),
                                    block.attribute.get_foreground(),
                                ),
                            )),
                        );
                    }
                }
                _ => {}
            }
        }
    }
}
