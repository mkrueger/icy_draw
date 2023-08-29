use std::{
    cmp::{max, min},
    ffi::OsStr,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use eframe::{
    egui::{self, CursorIcon, Id, Key, Response, RichText},
    epaint::{mutex::Mutex, FontId, Vec2},
};
use i18n_embed_fl::fl;
use icy_engine::{
    ansi, AttributedChar, Buffer, BufferParser, Layer, Line, Position, Rectangle, SaveOptions,
    Size, TextAttribute,
};

mod undo_stack;
use icy_engine_egui::{
    show_terminal_area, BackgroundEffect, BufferView, MonitorSettings, TerminalCalc,
};
pub use undo_stack::*;

use crate::{
    model::{MKey, MModifiers, Tool},
    Document, DocumentOptions, TerminalResult,
};

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

pub struct AnsiEditor {
    pub id: usize,

    is_dirty: bool,
    enabled: bool,
    drag_start: Option<Vec2>,
    last_pos: Position,

    pub cur_selection: Option<Selection>,
    pub buffer_view: Arc<eframe::epaint::mutex::Mutex<BufferView>>,
    buffer_parser: Box<dyn BufferParser>,

    pub cur_outline: usize,
    pub is_inactive: bool,

    pub reference_image: Option<PathBuf>,
    pub cur_layer: usize,
    pub cur_font_page: i32,
    // pub outline_changed: std::boxed::Box<dyn Fn(&Editor)>,
    //pub request_refresh: Box<dyn Fn ()>,
    atomic_undo_stack: Vec<usize>,

    pub undo_stack: Vec<Box<dyn UndoOperation>>,
    pub redo_stack: Vec<Box<dyn UndoOperation>>,

    pub egui_id: Id,
    //pub pos_changed: std::boxed::Box<dyn Fn(&Editor, Position)>,
    //pub attr_changed: std::boxed::Box<dyn Fn(TextAttribute)>
}

impl Document for AnsiEditor {
    fn get_title(&self) -> String {
        if let Some(file_name) = &self.buffer_view.lock().buf.file_name {
            file_name.file_name().unwrap().to_str().unwrap().to_string()
        } else {
            "Untitled".to_string()
        }
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn save(&mut self, file_name: &str) -> TerminalResult<()> {
        let file = PathBuf::from(file_name);
        let options = SaveOptions::new();
        let bytes = self
            .buffer_view
            .lock()
            .buf
            .to_bytes(file.extension().unwrap().to_str().unwrap(), &options)?;
        fs::write(file_name, bytes)?;
        self.is_dirty = false;
        Ok(())
    }

    fn show_ui(
        &mut self,
        ui: &mut egui_dock::egui::Ui,
        cur_tool: &mut Box<dyn Tool>,
        options: &DocumentOptions,
    ) {
        ui.allocate_ui(
            Vec2::new(ui.available_width(), ui.available_height() - 35.0),
            |ui| {
                let opt = icy_engine_egui::TerminalOptions {
                    focus_lock: false,
                    stick_to_bottom: false,
                    scale: Some(options.scale),
                    settings: MonitorSettings {
                        background_effect: BackgroundEffect::Checkers,
                        ..Default::default()
                    },
                    id: Some(Id::new(self.id + 10000)),
                    ..Default::default()
                };

                let (response, calc) = show_terminal_area(ui, self.buffer_view.clone(), opt);
                self.handle_response(ui, response, calc, cur_tool);
            },
        );
        self.show_toolbar(ui);

        // TODO: Context menu
        //let response = response.context_menu(|ui| terminal_context_menu(&self, ui));
    }

    fn get_buffer_view(&mut self) -> Option<&mut AnsiEditor> {
        Some(self)
    }

    fn destroy(&self, gl: &glow::Context) {
        self.buffer_view.lock().destroy(gl);
    }
}

impl AnsiEditor {
    pub fn new(gl: &Arc<glow::Context>, id: usize, buf: Buffer) -> Self {
        let buffer_view = Arc::new(Mutex::new(BufferView::from_buffer(
            gl,
            buf,
            glow::NEAREST as i32,
            icy_engine_egui::FontExtension::Off,
        )));
        // let buffer_parser = ansi::Parser::default();
        let buffer_parser = ansi::Parser::default();

        AnsiEditor {
            id,
            buffer_view,
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

            is_dirty: false,
            enabled: true,
            drag_start: None,
            last_pos: Position::default(),
            buffer_parser: Box::new(buffer_parser),
            egui_id: Id::new(id),
        }
    }

    pub fn output_string(&mut self, str: &str) {
        for ch in str.chars() {
            let translated_char = self.buffer_parser.convert_from_unicode(ch, 0);
            if let Err(err) = self.print_char(translated_char as u8) {
                eprintln!("{}", err);
            }
        }
    }

    pub fn print_char(&mut self, c: u8) -> Result<(), Box<dyn std::error::Error>> {
        self.buffer_view
            .lock()
            .print_char(&mut self.buffer_parser, unsafe {
                char::from_u32_unchecked(c as u32)
            })?;
        self.buffer_view.lock().redraw_view();
        Ok(())
    }
    pub fn get_caret_position(&self) -> Position {
        self.buffer_view.lock().caret.get_position()
    }

    pub fn set_caret_position(&mut self, pos: Position) {
        let buffer_view = &mut self.buffer_view.lock();
        let pos = Position::new(
            min(buffer_view.buf.get_width() - 1, max(0, pos.x)),
            min(buffer_view.buf.get_line_count() - 1, max(0, pos.y)),
        );
        buffer_view.caret.set_position(pos);
        //(self.pos_changed)(self, pos);
    }

    pub fn set_caret_attribute(&mut self, attr: TextAttribute) {
        if attr == self.buffer_view.lock().caret.get_attribute() {
            return;
        }

        self.buffer_view.lock().caret.set_attr(attr);
        // (self.attr_changed)(attr);
    }

    pub fn clear_layer(&mut self, layer_num: usize) -> ClearLayerOperation {
        let layers = std::mem::take(&mut self.buffer_view.lock().buf.layers[layer_num].lines);
        ClearLayerOperation {
            layer_num,
            lines: layers,
        }
    }

    /*   pub fn get_overlay_layer(&mut self) -> &mut Option<Layer> {
        self.buffer_view.lock().buf.get_overlay_layer()
    }*/

    pub fn join_overlay(&mut self) {
        self.begin_atomic_undo();
        let opt_layer = self.buffer_view.lock().buf.remove_overlay();

        if let Some(layer) = &opt_layer {
            for y in 0..layer.lines.len() {
                let line = &layer.lines[y];
                for x in 0..line.chars.len() {
                    let ch = line.chars[x];
                    if ch.is_visible() {
                        self.set_char(Position::new(x as i32, y as i32), ch);
                    }
                }
            }
        }
        self.end_atomic_undo();
    }

    pub fn delete_line(&mut self, line: i32) {
        // TODO: Undo
        let layer = &mut self.buffer_view.lock().buf.layers[self.cur_layer];
        layer.remove_line(line);
    }

    pub fn insert_line(&mut self, line: i32) {
        // TODO: Undo
        let layer = &mut self.buffer_view.lock().buf.layers[self.cur_layer];
        layer.insert_line(line, Line::new());
    }

    pub fn pickup_color(&mut self, pos: Position) {
        let ch = self.buffer_view.lock().buf.get_char(pos);
        if ch.is_visible() {
            self.buffer_view.lock().caret.set_attr(ch.attribute);
        }
    }

    pub fn set_caret(&mut self, x: i32, y: i32) -> Event {
        let old = self.buffer_view.lock().caret.get_position();
        let w = self.buffer_view.lock().buf.get_width() - 1;
        let h = self.buffer_view.lock().buf.get_line_count() - 1;
        self.set_caret_position(Position::new(min(max(0, x), w), min(max(0, y), h)));
        Event::CursorPositionChange(old, self.buffer_view.lock().caret.get_position())
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
            self.buffer_view
                .lock()
                .buf
                .to_bytes(ext.as_str(), options)?
        } else {
            self.buffer_view.lock().buf.to_bytes("icd", options)?
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

        Ok(DEFAULT_OUTLINE_TABLE[self.cur_outline][i] as u16)
    }

    pub fn get_outline_char_code_from(outline: usize, i: usize) -> Result<u16, &'static str> {
        if outline >= DEFAULT_OUTLINE_TABLE.len() {
            return Err("current outline out of range.");
        }
        if !(0..=10).contains(&i) {
            return Err("outline char# out of range.");
        }
        Ok(DEFAULT_OUTLINE_TABLE[outline][i] as u16)
    }

    pub fn get_char(&self, pos: Position) -> AttributedChar {
        self.buffer_view.lock().buf.get_char(pos)
    }

    pub fn get_char_from_cur_layer(&self, pos: Position) -> AttributedChar {
        if self.cur_layer >= self.buffer_view.lock().buf.layers.len() {
            return AttributedChar::invisible();
        }
        self.buffer_view.lock().buf.layers[self.cur_layer].get_char(pos)
    }

    pub fn set_char(&mut self, pos: Position, dos_char: AttributedChar) {
        if self.point_is_valid(pos) {
            self.redo_stack.clear();
            let old = self.buffer_view.lock().buf.layers[self.cur_layer].get_char(pos);
            self.buffer_view.lock().buf.layers[self.cur_layer].set_char(pos, dos_char);
            self.undo_stack.push(Box::new(UndoSetChar {
                pos,
                layer: self.cur_layer,
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
            op.undo(&mut self.buffer_view.lock().buf);
            self.redo_stack.push(op);
        }
    }

    pub fn redo(&mut self) {
        if let Some(op) = self.redo_stack.pop() {
            op.redo(&mut self.buffer_view.lock().buf);
            self.undo_stack.push(op);
        }
    }

    pub fn fill(&mut self, rect: Rectangle, dos_char: AttributedChar) {
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
            return selection.rectangle.contains_pt(pos);
        }

        pos.x >= 0
            && pos.y >= 0
            && pos.x < self.buffer_view.lock().buf.get_width()
            && pos.y < self.buffer_view.lock().buf.get_line_count()
    }

    pub fn type_key(&mut self, char_code: char) {
        let pos = self.buffer_view.lock().caret.get_position();
        if self.buffer_view.lock().caret.insert_mode {
            let start = self.buffer_view.lock().buf.get_width() - 1;
            for i in start..=pos.x {
                let next = self.get_char_from_cur_layer(Position::new(i - 1, pos.y));
                self.set_char(Position::new(i, pos.y), next);
            }
        }
        let attr = self.buffer_view.lock().caret.get_attribute();
        self.set_char(pos, AttributedChar::new(char_code, attr));
        self.set_caret(pos.x + 1, pos.y);
    }

    pub fn delete_selection(&mut self) {
        if let Some(selection) = &self.cur_selection.clone() {
            self.begin_atomic_undo();
            let mut pos = selection.rectangle.start;
            for _ in 0..selection.rectangle.size.height {
                for _ in 0..selection.rectangle.size.width {
                    if self.cur_layer == self.buffer_view.lock().buf.layers.len() - 1 {
                        self.set_char(pos, AttributedChar::default());
                    } else {
                        self.set_char(pos, AttributedChar::invisible());
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
                r.start.x + r.size.width - 1,
                r.start.y + r.size.height - 1,
            )
        } else {
            (
                0,
                self.buffer_view.lock().caret.get_position().y,
                self.buffer_view.lock().buf.get_width() - 1,
                self.buffer_view.lock().caret.get_position().y,
            )
        }
    }

    pub fn justify_left(&mut self) {
        self.begin_atomic_undo();
        let (x1, y1, x2, y2) = self.get_blockaction_rectangle();
        let is_bg_layer = self.cur_layer == self.buffer_view.lock().buf.layers.len() - 1;
        {
            let layer = &mut self.buffer_view.lock().buf.layers[self.cur_layer];
            for y in y1..=y2 {
                let mut removed_chars = 0;
                let len = x2 - x1 + 1;
                while removed_chars < len {
                    let ch = layer.get_char(Position::new(x1 + removed_chars, y));
                    if ch.is_visible() && !ch.is_transparent() {
                        break;
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
                        AttributedChar::default()
                    } else {
                        AttributedChar::invisible()
                    };

                    let pos = Position::new(x, y);
                    let old = layer.get_char(pos);
                    layer.set_char(pos, ch);
                    self.undo_stack.push(Box::new(UndoSetChar {
                        pos,
                        layer: self.cur_layer,
                        old,
                        new: ch,
                    }));
                }
            }
        }
        self.end_atomic_undo();
    }

    pub fn justify_center(&mut self) {
        self.begin_atomic_undo();
        self.justify_left();

        let (x1, y1, x2, y2) = self.get_blockaction_rectangle();
        let is_bg_layer = self.cur_layer == self.buffer_view.lock().buf.layers.len() - 1;
        {
            let layer = &mut self.buffer_view.lock().buf.layers[self.cur_layer];
            for y in y1..=y2 {
                let mut removed_chars = 0;
                let len = x2 - x1 + 1;
                while removed_chars < len {
                    let ch = layer.get_char(Position::new(x2 - removed_chars, y));
                    if ch.is_visible() && !ch.is_transparent() {
                        break;
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
                        AttributedChar::default()
                    } else {
                        AttributedChar::invisible()
                    };

                    let pos = Position::new(x2 - x, y);
                    let old = layer.get_char(pos);
                    layer.set_char(pos, ch);
                    self.undo_stack.push(Box::new(UndoSetChar {
                        pos,
                        layer: self.cur_layer,
                        old,
                        new: ch,
                    }));
                }
            }
        }
        self.end_atomic_undo();
    }

    pub fn justify_right(&mut self) {
        self.begin_atomic_undo();
        let (x1, y1, x2, y2) = self.get_blockaction_rectangle();
        let is_bg_layer = self.cur_layer == self.buffer_view.lock().buf.layers.len() - 1;
        {
            let layer = &mut self.buffer_view.lock().buf.layers[self.cur_layer];
            for y in y1..=y2 {
                let mut removed_chars = 0;
                let len = x2 - x1 + 1;
                while removed_chars < len {
                    let ch = layer.get_char(Position::new(x2 - removed_chars, y));
                    if ch.is_visible() && !ch.is_transparent() {
                        break;
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
                        AttributedChar::default()
                    } else {
                        AttributedChar::invisible()
                    };

                    let pos = Position::new(x2 - x, y);
                    let old = layer.get_char(pos);
                    layer.set_char(pos, ch);
                    self.undo_stack.push(Box::new(UndoSetChar {
                        pos,
                        layer: self.cur_layer,
                        old,
                        new: ch,
                    }));
                }
            }
        }
        self.end_atomic_undo();
    }

    pub fn flip_x(&mut self) {
        self.begin_atomic_undo();
        let (x1, y1, x2, y2) = self.get_blockaction_rectangle();
        {
            let layer = &mut self.buffer_view.lock().buf.layers[self.cur_layer];
            for y in y1..=y2 {
                for x in 0..=(x2 - x1) / 2 {
                    let pos1 = Position::new(x1 + x, y);
                    let pos2 = Position::new(x2 - x, y);
                    layer.swap_char(pos1, pos2);
                    self.undo_stack.push(Box::new(super::UndoSwapChar {
                        layer: self.cur_layer,
                        pos1,
                        pos2,
                    }));
                }
            }
        }
        self.end_atomic_undo();
    }

    pub fn redraw_view(&self) {
        self.buffer_view.lock().redraw_view();
    }

    pub fn flip_y(&mut self) {
        self.begin_atomic_undo();
        let (x1, y1, x2, y2) = self.get_blockaction_rectangle();
        {
            let layer = &mut self.buffer_view.lock().buf.layers[self.cur_layer];
            for x in x1..=x2 {
                for y in 0..=(y2 - y1) / 2 {
                    let pos1 = Position::new(x, y1 + y);
                    let pos2 = Position::new(x, y2 - y);
                    layer.swap_char(pos1, pos2);
                    self.undo_stack.push(Box::new(super::UndoSwapChar {
                        layer: self.cur_layer,
                        pos1,
                        pos2,
                    }));
                }
            }
        }
        self.end_atomic_undo();
    }
    pub fn crop(&mut self) {
        self.begin_atomic_undo();
        let (x1, y1, x2, y2) = self.get_blockaction_rectangle();

        let new_height = y2 - y1;
        let new_width = x2 - x1;

        if new_height == self.buffer_view.lock().buf.get_line_count()
            && new_width == self.buffer_view.lock().buf.get_width()
        {
            return;
        }

        let mut new_layers = Vec::new();
        for l in 0..self.buffer_view.lock().buf.layers.len() {
            let old_layer = &self.buffer_view.lock().buf.layers[l];
            let mut new_layer = Layer::default();
            new_layer.title = old_layer.title.clone();
            new_layer.is_visible = old_layer.is_visible;
            new_layer.offset = Position::new(0, 0);
            new_layer.lines = Vec::new();
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

        /* TODO
        self.undo_stack.push(Box::new(super::UndoReplaceLayers {

            old_layer: self.buffer_view.lock().buf.layers.clone(),
            new_layer: new_layers.clone(),
            old_size: Size::new(
                self.buffer_view.lock().buf.get_width(),
                self.buffer_view.lock().buf.get_line_count(),
            ),
            new_size: Size::new(new_width, new_height),
        })); */

        self.buffer_view.lock().buf.layers = new_layers;
        self.buffer_view.lock().buf.set_buffer_width(new_width);
        self.buffer_view.lock().buf.set_buffer_height(new_height);
        self.end_atomic_undo();
    }
    pub fn switch_fg_bg_color(&mut self) {
        let mut attr = self.buffer_view.lock().caret.get_attribute();
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

    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let pos = self.buffer_view.lock().caret.get_position();

            let label_font_size = 20.0;

            ui.vertical(|ui| {
                ui.add_space(4.);
                ui.label(
                    RichText::new(fl!(
                        crate::LANGUAGE_LOADER,
                        "toolbar-position",
                        line = pos.y,
                        column = pos.x
                    ))
                    .font(FontId::proportional(label_font_size)),
                );
            });

            let r = ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let cur_outline = self.cur_outline;
                let cur_font_page = self.buffer_view.lock().caret.get_font_page();

                let button_font_size = 16.0;
                if ui
                    .selectable_label(
                        false,
                        RichText::new("▶").font(FontId::proportional(button_font_size)),
                    )
                    .clicked()
                {
                    self.cur_outline = (cur_outline + 1) % DEFAULT_OUTLINE_TABLE.len();
                }
                ui.label(
                    RichText::new((cur_outline + 1).to_string())
                        .font(FontId::proportional(label_font_size)),
                );

                if ui
                    .selectable_label(
                        false,
                        RichText::new("◀").font(FontId::proportional(button_font_size)),
                    )
                    .clicked()
                {
                    self.cur_outline = (cur_outline + DEFAULT_OUTLINE_TABLE.len() - 1)
                        % DEFAULT_OUTLINE_TABLE.len();
                }

                for i in (0..10).rev() {
                    let ch = self.get_outline_char_code(i).unwrap();
                    ui.add(crate::model::pencil_imp::draw_glyph_plain(
                        self,
                        unsafe { char::from_u32_unchecked(ch as u32) },
                        cur_font_page,
                    ));

                    ui.label(
                        RichText::new(format!("F{}", i + 1))
                            .font(FontId::proportional(label_font_size)),
                    );
                }
            });
            r.response
        });
    }

    fn handle_response(
        &mut self,
        ui: &egui::Ui,
        response: Response,
        calc: TerminalCalc,
        cur_tool: &mut Box<dyn Tool>,
    ) {
        if self.enabled {
            let events = ui.input(|i| i.events.clone());
            for e in &events {
                match e {
                    egui::Event::Copy => {
                        let buffer_view = self.buffer_view.clone();
                        let mut l = buffer_view.lock();
                        if let Some(txt) = l.get_copy_text(&*self.buffer_parser) {
                            ui.output_mut(|o| o.copied_text = txt);
                        }
                    }
                    egui::Event::Cut => {}
                    egui::Event::Paste(text) => {
                        self.output_string(text);
                        self.buffer_view.lock().redraw_view();
                        self.is_dirty = true;
                    }

                    egui::Event::CompositionEnd(text) | egui::Event::Text(text) => {
                        for c in text.chars() {
                            cur_tool.handle_key(self, MKey::Character(c as u16), MModifiers::None);
                        }
                        self.is_dirty = true;
                        self.redraw_view();
                    }

                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => {
                        let mut key_code = *key as u32;
                        if modifiers.ctrl || modifiers.command {
                            key_code |= CTRL_MOD;
                        }
                        if modifiers.shift {
                            key_code |= SHIFT_MOD;
                        }

                        let mut modifier: MModifiers = MModifiers::None;
                        if modifiers.ctrl || modifiers.command {
                            modifier = MModifiers::Control;
                        }

                        if modifiers.shift {
                            modifier = MModifiers::Shift;
                        }
                        for (k, m) in ANSI_KEY_MAP {
                            if *k == key_code {
                                cur_tool.handle_key(self, *m, modifier);
                                self.buffer_view.lock().redraw_view();
                                ui.input_mut(|i| i.consume_key(*modifiers, *key));
                                self.is_dirty = true;
                                self.redraw_view();
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if calc.buffer_rect.contains(mouse_pos) {
                    let click_pos = calc.calc_click_pos(mouse_pos);
                    println!("click !");
                    /*
                    let b: i32 = match responsee.b {
                                     PointerButton::Primary => 1,
                                     PointerButton::Secondary => 2,
                                     PointerButton::Middle => 3,
                                     PointerButton::Extra1 => 4,
                                     PointerButton::Extra2 => 5,
                                 }; */
                    cur_tool.handle_click(
                        self,
                        1,
                        Position::new(click_pos.x as i32, click_pos.y as i32),
                    );
                    self.is_dirty = true;
                    self.redraw_view();
                }
            }
        }

        if response.drag_started() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if calc.buffer_rect.contains(mouse_pos) {
                    let click_pos = calc.calc_click_pos(mouse_pos);
                    self.last_pos = Position::new(click_pos.x as i32, click_pos.y as i32);
                    self.drag_start = Some(click_pos);
                }
            }
            self.last_pos = Position::new(-1, -1);
            self.redraw_view();
        }

        if response.dragged() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                let click_pos = calc.calc_click_pos(mouse_pos);
                if let Some(ds) = self.drag_start {
                    let cur = Position::new(click_pos.x as i32, click_pos.y as i32);

                    if cur != self.last_pos {
                        self.last_pos = cur;
                        cur_tool.handle_drag(self, Position::new(ds.x as i32, ds.y as i32), cur);
                    }
                }
            }
            self.redraw_view();
        }

        if response.drag_released() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                let click_pos = calc.calc_click_pos(mouse_pos);
                if let Some(ds) = self.drag_start {
                    let cur = Position::new(click_pos.x as i32, click_pos.y as i32);
                    self.is_dirty = true;

                    cur_tool.handle_drag_end(self, Position::new(ds.x as i32, ds.y as i32), cur);
                }
            }
            self.last_pos = Position::new(-1, -1);

            self.drag_start = None;
            self.redraw_view();
        }

        if response.hovered() {
            let hover_pos_opt = ui.input(|i| i.pointer.hover_pos());
            if let Some(hover_pos) = hover_pos_opt {
                if calc.terminal_rect.contains(hover_pos) {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::Text);
                }
            }
        }
    }

    pub(crate) fn set_file_name(&self, file_name: impl Into<PathBuf>) {
        self.buffer_view.lock().buf.file_name = Some(file_name.into());
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

pub fn terminal_context_menu(editor: &mut AnsiEditor, ui: &mut egui::Ui) {
    ui.input_mut(|i| i.events.clear());

    if ui
        .button(fl!(crate::LANGUAGE_LOADER, "menu-copy"))
        .clicked()
    {
        ui.input_mut(|i| i.events.push(egui::Event::Copy));
        ui.close_menu();
    }

    if ui
        .button(fl!(crate::LANGUAGE_LOADER, "menu-paste"))
        .clicked()
    {
        /* let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        if let Ok(text) = ctx.get_contents() {
            ui.input_mut().events.push(egui::Event::Paste(text));
        }
        ui.close_menu();*/
    }

    let sel = editor.buffer_view.lock().get_selection().clone();

    if let Some(_sel) = sel {
        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-erase"))
            .clicked()
        {
            editor.delete_selection();
            ui.close_menu();
        }
        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-flipx"))
            .clicked()
        {
            editor.flip_x();
            ui.close_menu();
        }
        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-flipy"))
            .clicked()
        {
            editor.flip_y();
            ui.close_menu();
        }

        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-justifyleft"))
            .clicked()
        {
            editor.justify_left();
            ui.close_menu();
        }
        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-justifyright"))
            .clicked()
        {
            editor.justify_right();
            ui.close_menu();
        }
        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-justifycenter"))
            .clicked()
        {
            editor.justify_center();
            ui.close_menu();
        }
    }
}

pub const CTRL_MOD: u32 = 0b1000_0000_0000_0000_0000;
pub const SHIFT_MOD: u32 = 0b0100_0000_0000_0000_0000;

pub static ANSI_KEY_MAP: &[(u32, MKey)] = &[
    (Key::Escape as u32, MKey::Escape),
    (Key::Home as u32, MKey::Home),
    (Key::Insert as u32, MKey::Insert),
    (Key::Backspace as u32, MKey::Backspace),
    (Key::Enter as u32, MKey::Return),
    (Key::Tab as u32, MKey::Tab),
    (Key::Delete as u32, MKey::Delete),
    (Key::End as u32, MKey::End),
    (Key::PageUp as u32, MKey::PageUp),
    (Key::PageDown as u32, MKey::PageDown),
    (Key::F1 as u32, MKey::F1),
    (Key::F2 as u32, MKey::F2),
    (Key::F3 as u32, MKey::F3),
    (Key::F4 as u32, MKey::F4),
    (Key::F5 as u32, MKey::F5),
    (Key::F6 as u32, MKey::F6),
    (Key::F7 as u32, MKey::F7),
    (Key::F8 as u32, MKey::F8),
    (Key::F9 as u32, MKey::F9),
    (Key::F10 as u32, MKey::F10),
    (Key::F11 as u32, MKey::F11),
    (Key::F12 as u32, MKey::F12),
    (Key::ArrowUp as u32, MKey::Up),
    (Key::ArrowDown as u32, MKey::Down),
    (Key::ArrowRight as u32, MKey::Right),
    (Key::ArrowLeft as u32, MKey::Left),
    (Key::A as u32 | CTRL_MOD, MKey::Character(1)),
    (Key::B as u32 | CTRL_MOD, MKey::Character(2)),
    (Key::C as u32 | CTRL_MOD, MKey::Character(3)),
    (Key::D as u32 | CTRL_MOD, MKey::Character(4)),
    (Key::E as u32 | CTRL_MOD, MKey::Character(5)),
    (Key::F as u32 | CTRL_MOD, MKey::Character(6)),
    (Key::G as u32 | CTRL_MOD, MKey::Character(7)),
    (Key::H as u32 | CTRL_MOD, MKey::Character(8)),
    (Key::I as u32 | CTRL_MOD, MKey::Character(9)),
    (Key::J as u32 | CTRL_MOD, MKey::Character(10)),
    (Key::K as u32 | CTRL_MOD, MKey::Character(11)),
    (Key::L as u32 | CTRL_MOD, MKey::Character(12)),
    (Key::M as u32 | CTRL_MOD, MKey::Character(13)),
    (Key::N as u32 | CTRL_MOD, MKey::Character(14)),
    (Key::O as u32 | CTRL_MOD, MKey::Character(15)),
    (Key::P as u32 | CTRL_MOD, MKey::Character(16)),
    (Key::Q as u32 | CTRL_MOD, MKey::Character(17)),
    (Key::R as u32 | CTRL_MOD, MKey::Character(18)),
    (Key::S as u32 | CTRL_MOD, MKey::Character(19)),
    (Key::T as u32 | CTRL_MOD, MKey::Character(20)),
    (Key::U as u32 | CTRL_MOD, MKey::Character(21)),
    (Key::V as u32 | CTRL_MOD, MKey::Character(22)),
    (Key::W as u32 | CTRL_MOD, MKey::Character(23)),
    (Key::X as u32 | CTRL_MOD, MKey::Character(24)),
    (Key::Y as u32 | CTRL_MOD, MKey::Character(25)),
    (Key::Z as u32 | CTRL_MOD, MKey::Character(26)),
    (Key::Num2 as u32 | CTRL_MOD, MKey::Character(0)),
    (Key::Num3 as u32 | CTRL_MOD, MKey::Character(0x1B)),
    (Key::Num4 as u32 | CTRL_MOD, MKey::Character(0x1C)),
    (Key::Num5 as u32 | CTRL_MOD, MKey::Character(0x1D)),
    (Key::Num6 as u32 | CTRL_MOD, MKey::Character(0x1E)),
    (Key::Num7 as u32 | CTRL_MOD, MKey::Character(0x1F)),
];
