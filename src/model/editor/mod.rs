use std::{
    cmp::{max, min},
    ffi::OsStr,
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

use icy_engine::{
    AttributedChar, Buffer, Caret, Layer, Line, Position, Rectangle, SaveOptions, Size,
    TextAttribute,
};

mod undo_stack;
pub use undo_stack::*;

pub enum Event {
    None,
    CursorPositionChange(Position, Position),
}

#[derive(Clone, Debug)]
pub enum Shape {
    Rectangle,
    Elipse,
}

#[derive(Clone, Debug)]
pub struct Selection {
    pub shape: Shape,
    pub rectangle: Rectangle,
    pub is_preview: bool,
}

impl Selection {
    pub fn new() -> Self {
        Selection {
            shape: Shape::Rectangle,
            rectangle: Rectangle::from(-1, -1, 0, 0),
            is_preview: true,
        }
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Editor {
    pub id: usize,
    pub buf: Buffer,

    pub caret: Caret,
    pub cur_selection: Option<Selection>,

    pub cur_outline: usize,
    pub is_inactive: bool,
    pub cur_font_page: usize,

    pub reference_image: Option<PathBuf>,
    pub cur_layer: i32,
    // pub outline_changed: std::boxed::Box<dyn Fn(&Editor)>,
    //pub request_refresh: Box<dyn Fn ()>,
    atomic_undo_stack: Vec<usize>,

    pub undo_stack: Vec<Box<dyn UndoOperation>>,
    pub redo_stack: Vec<Box<dyn UndoOperation>>,
    //pub pos_changed: std::boxed::Box<dyn Fn(&Editor, Position)>,
    //pub attr_changed: std::boxed::Box<dyn Fn(TextAttribute)>
}

impl std::fmt::Debug for Editor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Editor")
            .field("id", &self.id)
            .field("buf", &self.buf)
            .field("caret", &self.caret)
            .field("cur_selection", &self.cur_selection)
            .field("cur_outline", &self.cur_outline)
            .field("is_inactive", &self.is_inactive)
            .finish()
    }
}

impl Default for Editor {
    fn default() -> Self {
        Editor::new(0, Buffer::new())
    }
}

impl Editor {
    pub fn new(id: usize, buf: Buffer) -> Self {
        Editor {
            id,
            buf,
            caret: Caret::default(),
            cur_selection: None,
            cur_outline: 0,
            is_inactive: false,
            reference_image: None,
            //outline_changed: Box::new(|_| {}),
            // request_refresh: Box::new(|| {}),
            cur_layer: 0,
            cur_font_page: 0,
            atomic_undo_stack: Vec::new(),
            //pos_changed: Box::new(|_, _| {}),
            //attr_changed: Box::new(|_| {}),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn get_caret_position(&self) -> Position {
        self.caret.get_position()
    }

    pub fn set_caret_position(&mut self, pos: Position) {
        let pos = Position::new(
            min(self.buf.get_buffer_width() as i32 - 1, max(0, pos.x)),
            min(self.buf.get_real_buffer_height() as i32 - 1, max(0, pos.y)),
        );
        self.caret.set_position(pos);
        //(self.pos_changed)(self, pos);
    }

    pub fn set_caret_attribute(&mut self, attr: TextAttribute) {
        if attr == self.caret.get_attribute() {
            return;
        }

        self.caret.set_attr(attr);
        // (self.attr_changed)(attr);
    }

    pub fn clear_layer(&mut self, layer_num: i32) -> ClearLayerOperation {
        let layers = std::mem::take(&mut self.buf.layers[layer_num as usize].lines);
        ClearLayerOperation {
            layer_num,
            lines: layers,
        }
    }

    pub fn get_cur_layer(&mut self) -> Option<&Layer> {
        self.buf.layers.get(self.cur_layer as usize)
    }

    pub fn get_cur_layer_mut(&mut self) -> Option<&mut Layer> {
        self.buf.layers.get_mut(self.cur_layer as usize)
    }

    pub fn get_overlay_layer(&mut self) -> &mut Option<Layer> {
        self.buf.get_overlay_layer()
    }

    pub fn join_overlay(&mut self) {
        self.begin_atomic_undo();
        let opt_layer = self.buf.remove_overlay();

        if let Some(layer) = &opt_layer {
            for y in 0..layer.lines.len() {
                let line = &layer.lines[y];
                for x in 0..line.chars.len() {
                    let ch = line.chars[x];
                    if ch.is_some() {
                        self.set_char(Position::new(x as i32, y as i32), ch);
                    }
                }
            }
        }
        self.end_atomic_undo();
    }

    pub fn delete_line(&mut self, line: i32) {
        // TODO: Undo
        let layer = &mut self.buf.layers[self.cur_layer as usize];
        layer.remove_line(line);
    }

    pub fn insert_line(&mut self, line: i32) {
        // TODO: Undo
        let layer = &mut self.buf.layers[self.cur_layer as usize];
        layer.insert_line(line, Line::new());
    }

    pub fn pickup_color(&mut self, pos: Position) {
        let ch = self.buf.get_char(pos);
        if let Some(ch) = ch {
            self.caret.set_attr(ch.attribute);
        }
    }

    pub fn set_caret(&mut self, x: i32, y: i32) -> Event {
        let old = self.caret.get_position();
        self.set_caret_position(Position::new(
            min(max(0, x), self.buf.get_buffer_width() as i32 - 1),
            min(max(0, y), self.buf.get_real_buffer_height() as i32 - 1),
        ));
        Event::CursorPositionChange(old, self.caret.get_position())
    }

    pub fn get_cur_outline(&self) -> usize {
        self.cur_outline
    }

    pub fn set_cur_outline(&mut self, outline: usize) {
        self.cur_outline = outline;
        //(self.outline_changed)(self);
    }

    pub fn save_content(&self, file_name: &Path, options: &SaveOptions) -> io::Result<bool> {
        let mut f = File::create(file_name)?;

        let content = if let Some(ext) = file_name.extension() {
            let ext = OsStr::to_str(ext).unwrap().to_lowercase();
            self.buf.to_bytes(ext.as_str(), options)?
        } else {
            self.buf.to_bytes("mdf", options)?
        };
        f.write_all(&content)?;
        Ok(true)
    }

    pub fn get_outline_char_code(&self, i: usize) -> Result<u16, &str> {
        if self.cur_outline >= DEFAULT_OUTLINE_TABLE.len() {
            return Err("current outline out of range.");
        }
        if !(0..=10).contains(&i) {
            return Err("outline char# out of range.");
        }

        Ok(DEFAULT_OUTLINE_TABLE[self.cur_outline as usize][i as usize] as u16)
    }

    pub fn get_outline_char_code_from(outline: usize, i: usize) -> Result<u16, &'static str> {
        if  outline >= DEFAULT_OUTLINE_TABLE.len() {
            return Err("current outline out of range.");
        }
        if !(0..=10).contains(&i) {
            return Err("outline char# out of range.");
        }
        Ok(DEFAULT_OUTLINE_TABLE[outline as usize][i as usize] as u16)
    }

    pub fn get_char(&self, pos: Position) -> Option<AttributedChar> {
        self.buf.get_char(pos)
    }

    pub fn get_char_from_cur_layer(&self, pos: Position) -> Option<AttributedChar> {
        if self.cur_layer >= self.buf.layers.len() as i32 {
            return None;
        }
        self.buf.layers[self.cur_layer as usize].get_char(pos)
    }

    pub fn set_char(&mut self, pos: Position, dos_char: Option<AttributedChar>) {
        if self.point_is_valid(pos) {
            self.redo_stack.clear();
            let old = self.buf.get_char_from_layer(self.cur_layer as usize, pos);
            self.buf.set_char(self.cur_layer as usize, pos, dos_char);
            self.undo_stack.push(Box::new(UndoSetChar {
                pos,
                layer: self.cur_layer as usize,
                old,
                new: dos_char,
            }));
        }
    }
    pub fn begin_atomic_undo(&mut self) {
        self.atomic_undo_stack.push(self.undo_stack.len());
    }

    pub fn end_atomic_undo(&mut self) {
        let base_count = self.atomic_undo_stack.pop().unwrap();
        let count = self.undo_stack.len();
        if base_count == count {
            return;
        }

        let mut stack = Vec::new();
        while base_count < self.undo_stack.len() {
            let op = self.undo_stack.pop().unwrap();
            stack.push(op);
        }
        self.undo_stack.push(Box::new(super::AtomicUndo { stack }));
    }

    pub fn undo(&mut self) {
        if let Some(op) = self.undo_stack.pop() {
            op.undo(&mut self.buf);
            self.redo_stack.push(op);
        }
    }

    pub fn redo(&mut self) {
        if let Some(op) = self.redo_stack.pop() {
            op.redo(&mut self.buf);
            self.undo_stack.push(op);
        }
    }

    pub fn fill(&mut self, rect: Rectangle, dos_char: Option<AttributedChar>) {
        let mut pos = rect.start;
        self.begin_atomic_undo();
        for _ in 0..rect.size.height {
            for _ in 0..rect.size.width {
                self.set_char(pos, dos_char);
                pos.x += 1;
            }
            pos.y += 1;
            pos.x = rect.start.x;
        }
        self.end_atomic_undo();
    }

    pub fn point_is_valid(&self, pos: Position) -> bool {
        if let Some(selection) = &self.cur_selection {
            return selection.rectangle.is_inside(pos);
        }

        pos.x >= 0
            && pos.y >= 0
            && pos.x < self.buf.get_buffer_width() as i32
            && pos.y < self.buf.get_real_buffer_height() as i32
    }

    pub fn type_key(&mut self, char_code: char) {
        let pos = self.caret.get_position();
        if self.caret.insert_mode {
            for i in (self.buf.get_buffer_width() as i32 - 1)..=pos.x {
                let next = self.get_char_from_cur_layer(Position::new(i - 1, pos.y));
                self.set_char(Position::new(i, pos.y), next);
            }
        }

        self.set_char(
            pos,
            Some(AttributedChar::new(char_code, self.caret.get_attribute())),
        );
        self.set_caret(pos.x + 1, pos.y);
    }

    pub fn delete_selection(&mut self) {
        if let Some(selection) = &self.cur_selection.clone() {
            self.begin_atomic_undo();
            let mut pos = selection.rectangle.start;
            for _ in 0..selection.rectangle.size.height {
                for _ in 0..selection.rectangle.size.width {
                    if self.cur_layer == self.buf.layers.len() as i32 - 1 {
                        self.set_char(pos, Some(AttributedChar::default()));
                    } else {
                        self.set_char(pos, None);
                    }
                    pos.x += 1;
                }
                pos.y += 1;
                pos.x = selection.rectangle.start.x;
            }
            self.end_atomic_undo();
            self.cur_selection = None;
        }
    }

    pub fn clear_cur_layer(&mut self) {
        let b = Box::new(self.clear_layer(self.cur_layer));
        self.undo_stack.push(b);
    }

    fn get_blockaction_rectangle(&self) -> (i32, i32, i32, i32) {
        if let Some(selection) = &self.cur_selection {
            let r = &selection.rectangle;
            (
                r.start.x,
                r.start.y,
                r.start.x + r.size.width as i32 - 1,
                r.start.y + r.size.height as i32 - 1,
            )
        } else {
            (
                0,
                self.caret.get_position().y,
                self.buf.get_buffer_width() as i32 - 1,
                self.caret.get_position().y,
            )
        }
    }

    pub fn justify_left(&mut self) {
        self.begin_atomic_undo();
        let (x1, y1, x2, y2) = self.get_blockaction_rectangle();
        let is_bg_layer = self.cur_layer as usize == self.buf.layers.len() - 1;
        let layer = &mut self.buf.layers[self.cur_layer as usize];
        for y in y1..=y2 {
            let mut removed_chars = 0;
            let len = x2 - x1 + 1;
            while removed_chars < len {
                let ch = layer.get_char(Position::new(x1 + removed_chars, y));
                if let Some(ch) = ch {
                    if !ch.is_transparent() {
                        break;
                    }
                }
                removed_chars += 1;
            }
            if len == removed_chars {
                continue;
            }
            for x in x1..=x2 {
                let ch = if x + removed_chars <= x2 {
                    layer.get_char(Position::new(x + removed_chars, y))
                } else if is_bg_layer {
                    Some(AttributedChar::default())
                } else {
                    None
                };

                let pos = Position::new(x, y);
                let old = layer.get_char(pos);
                layer.set_char(pos, ch);
                self.undo_stack.push(Box::new(UndoSetChar {
                    pos,
                    layer: self.cur_layer as usize,
                    old,
                    new: ch,
                }));
            }
        }
        self.end_atomic_undo();
    }

    pub fn justify_center(&mut self) {
        self.begin_atomic_undo();
        self.justify_left();

        let (x1, y1, x2, y2) = self.get_blockaction_rectangle();
        let is_bg_layer = self.cur_layer as usize == self.buf.layers.len() - 1;
        let layer = &mut self.buf.layers[self.cur_layer as usize];
        for y in y1..=y2 {
            let mut removed_chars = 0;
            let len = x2 - x1 + 1;
            while removed_chars < len {
                let ch = layer.get_char(Position::new(x2 - removed_chars, y));
                if let Some(ch) = ch {
                    if !ch.is_transparent() {
                        break;
                    }
                }
                removed_chars += 1;
            }

            if len == removed_chars {
                continue;
            }
            removed_chars /= 2;
            for x in 0..len {
                let ch = if x2 - x - removed_chars >= x1 {
                    layer.get_char(Position::new(x2 - x - removed_chars, y))
                } else if is_bg_layer {
                    Some(AttributedChar::default())
                } else {
                    None
                };

                let pos = Position::new(x2 - x, y);
                let old = layer.get_char(pos);
                layer.set_char(pos, ch);
                self.undo_stack.push(Box::new(UndoSetChar {
                    pos,
                    layer: self.cur_layer as usize,
                    old,
                    new: ch,
                }));
            }
        }
        self.end_atomic_undo();
    }

    pub fn justify_right(&mut self) {
        self.begin_atomic_undo();
        let (x1, y1, x2, y2) = self.get_blockaction_rectangle();
        let is_bg_layer = self.cur_layer as usize == self.buf.layers.len() - 1;
        let layer = &mut self.buf.layers[self.cur_layer as usize];
        for y in y1..=y2 {
            let mut removed_chars = 0;
            let len = x2 - x1 + 1;
            while removed_chars < len {
                let ch = layer.get_char(Position::new(x2 - removed_chars, y));
                if let Some(ch) = ch {
                    if !ch.is_transparent() {
                        break;
                    }
                }
                removed_chars += 1;
            }

            if len == removed_chars {
                continue;
            }
            for x in 0..len {
                let ch = if x2 - x - removed_chars >= x1 {
                    layer.get_char(Position::new(x2 - x - removed_chars, y))
                } else if is_bg_layer {
                    Some(AttributedChar::default())
                } else {
                    None
                };

                let pos = Position::new(x2 - x, y);
                let old = layer.get_char(pos);
                layer.set_char(pos, ch);
                self.undo_stack.push(Box::new(UndoSetChar {
                    pos,
                    layer: self.cur_layer as usize,
                    old,
                    new: ch,
                }));
            }
        }
        self.end_atomic_undo();
    }

    pub fn flip_x(&mut self) {
        self.begin_atomic_undo();
        let (x1, y1, x2, y2) = self.get_blockaction_rectangle();
        let layer = &mut self.buf.layers[self.cur_layer as usize];
        for y in y1..=y2 {
            for x in 0..=(x2 - x1) / 2 {
                let pos1 = Position::new(x1 + x, y);
                let pos2 = Position::new(x2 - x, y);
                layer.swap_char(pos1, pos2);
                self.undo_stack.push(Box::new(super::UndoSwapChar {
                    layer: self.cur_layer as usize,
                    pos1,
                    pos2,
                }));
            }
        }
        self.end_atomic_undo();
    }

    pub fn flip_y(&mut self) {
        self.begin_atomic_undo();
        let (x1, y1, x2, y2) = self.get_blockaction_rectangle();
        let layer = &mut self.buf.layers[self.cur_layer as usize];
        for x in x1..=x2 {
            for y in 0..=(y2 - y1) / 2 {
                let pos1 = Position::new(x, y1 + y);
                let pos2 = Position::new(x, y2 - y);
                layer.swap_char(pos1, pos2);
                self.undo_stack.push(Box::new(super::UndoSwapChar {
                    layer: self.cur_layer as usize,
                    pos1,
                    pos2,
                }));
            }
        }
        self.end_atomic_undo();
    }
    pub fn crop(&mut self) {
        self.begin_atomic_undo();
        let (x1, y1, x2, y2) = self.get_blockaction_rectangle();

        let new_height = y2 - y1;
        let new_width = x2 - x1;

        if new_height == self.buf.get_real_buffer_height()
            && new_width == self.buf.get_buffer_width()
        {
            return;
        }

        let mut new_layers = Vec::new();
        for l in 0..self.buf.layers.len() {
            let old_layer = &self.buf.layers[l];
            let mut new_layer = Layer {
                title: old_layer.title.clone(),
                is_visible: old_layer.is_visible,
                is_locked: false,
                is_position_locked: false,
                offset: Position::new(0, 0),
                lines: Vec::new(),
                ..Default::default()
            };
            for y in y1..=y2 {
                for x in x1..=x2 {
                    new_layer.set_char(
                        Position::new(x - x1, y - y1),
                        old_layer.get_char(Position::new(x, y)),
                    );
                }
            }

            new_layer.is_locked = old_layer.is_locked;
            new_layer.is_position_locked = old_layer.is_position_locked;
            new_layers.push(new_layer);
        }

        self.undo_stack.push(Box::new(super::UndoReplaceLayers {
            old_layer: self.buf.layers.clone(),
            new_layer: new_layers.clone(),
            old_size: Size::new(
                self.buf.get_buffer_width(),
                self.buf.get_real_buffer_height(),
            ),
            new_size: Size::new(new_width, new_height),
        }));

        self.buf.layers = new_layers;
        self.buf.set_buffer_width(new_width);
        self.buf.set_buffer_height(new_height);
        self.end_atomic_undo();
    }
    pub fn switch_fg_bg_color(&mut self) {
        let mut attr = self.caret.get_attribute();
        let bg = attr.get_background();
        attr.set_background(attr.get_foreground());
        attr.set_foreground(bg);
        self.set_caret_attribute(attr);
    }

    pub fn erase_line(&mut self) {
        self.begin_atomic_undo();
        // TODO
        self.end_atomic_undo();
    }

    pub fn erase_line_to_start(&mut self) {
        self.begin_atomic_undo();
        // TODO
        self.end_atomic_undo();
    }

    pub fn erase_line_to_end(&mut self) {
        self.begin_atomic_undo();
        // TODO
        self.end_atomic_undo();
    }

    pub fn erase_column(&mut self) {
        self.begin_atomic_undo();
        // TODO
        self.end_atomic_undo();
    }

    pub fn erase_column_to_start(&mut self) {
        self.begin_atomic_undo();
        // TODO
        self.end_atomic_undo();
    }

    pub fn erase_column_to_end(&mut self) {
        self.begin_atomic_undo();
        // TODO
        self.end_atomic_undo();
    }

    pub fn delete_row(&mut self) {
        self.begin_atomic_undo();
        // TODO
        self.end_atomic_undo();
    }

    pub fn insert_row(&mut self) {
        self.begin_atomic_undo();
        // TODO
        self.end_atomic_undo();
    }

    pub fn delete_column(&mut self) {
        self.begin_atomic_undo();
        // TODO
        self.end_atomic_undo();
    }

    pub fn insert_column(&mut self) {
        self.begin_atomic_undo();
        // TODO
        self.end_atomic_undo();
    }
}

pub const DEFAULT_OUTLINE_TABLE: [[u8; 10]; 15] = [
    [218, 191, 192, 217, 196, 179, 195, 180, 193, 194],
    [201, 187, 200, 188, 205, 186, 204, 185, 202, 203],
    [213, 184, 212, 190, 205, 179, 198, 181, 207, 209],
    [214, 183, 211, 189, 196, 186, 199, 182, 208, 210],
    [197, 206, 216, 215, 232, 233, 155, 156, 153, 239],
    [176, 177, 178, 219, 223, 220, 221, 222, 254, 250],
    [1, 2, 3, 4, 5, 6, 240, 127, 14, 15],
    [24, 25, 30, 31, 16, 17, 18, 29, 20, 21],
    [174, 175, 242, 243, 169, 170, 253, 246, 171, 172],
    [227, 241, 244, 245, 234, 157, 228, 248, 251, 252],
    [224, 225, 226, 229, 230, 231, 235, 236, 237, 238],
    [128, 135, 165, 164, 152, 159, 247, 249, 173, 168],
    [131, 132, 133, 160, 166, 134, 142, 143, 145, 146],
    [136, 137, 138, 130, 144, 140, 139, 141, 161, 158],
    [147, 148, 149, 162, 167, 150, 129, 151, 163, 154],
];
