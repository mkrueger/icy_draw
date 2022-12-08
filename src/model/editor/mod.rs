use std::{cmp::{max, min}, path::Path, io::Write, fs::File, ffi::OsStr};

use icy_engine::{Position, Rectangle, Buffer, Caret, convert_to_binary, convert_to_xb, SaveOptions, convert_to_avt, convert_to_pcb, convert_to_asc};

pub enum Event {
    None,
    CursorPositionChange(Position, Position)
}

#[derive(Debug)]
pub enum Shape {
    Rectangle,
    Elipse
}

#[derive(Debug)]
pub struct Selection
{
    pub shape: Shape,
    pub rectangle: Rectangle,
    pub is_preview: bool,
    pub is_active: bool
}

impl Selection {
    pub fn new() -> Self
    {
        Selection {
            shape: Shape::Rectangle,
            rectangle:  Rectangle::from(-1, -1, 0, 0),
            is_preview: true,
            is_active: false
        }
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Editor {
    pub id: usize,
    pub buf: Buffer,
    
    pub cursor: Caret,
    pub cur_selection: Selection
}

impl Default for Editor 
{
    fn default() -> Self
    {
        Editor::new(0, Buffer::new())
    }
}

impl Editor 
{
    pub fn new(id: usize, buf: Buffer) -> Self 
    {
        Editor {
            id,
            buf, 
            cursor: Caret::default(),
            cur_selection: Selection::default()
        }
    }
/* 
    pub fn handle_key(&mut self, key: gtk4::gdk::Key, key_code: u32, modifier: gtk4::gdk::ModifierType) -> Event
    {
        unsafe {
            crate::WORKSPACE.cur_tool().handle_key( self, key, key_code, modifier)
        }
    }

    pub fn handle_click(&mut self, button: u32, x: i32, y: i32) -> Event
    {
        unsafe {
            crate::WORKSPACE.cur_tool().handle_click( self, button, x, y)
        }
    }

    pub fn handle_drag_begin(&mut self, start: Position, cur: Position) -> Event
    {
        unsafe {
            crate::WORKSPACE.cur_tool().handle_drag_begin( self, start, cur)
        }
    }

    pub fn handle_drag(&mut self, start: Position, cur: Position) -> Event
    {
        unsafe {
            crate::WORKSPACE.cur_tool().handle_drag( self, start, cur)
        }
    }

    pub fn handle_drag_end(&mut self, start: Position, cur: Position) -> Event
    {
        unsafe {
            crate::WORKSPACE.cur_tool().handle_drag_end( self, start, cur)
        }
    }
    */
    pub fn set_cursor(&mut self, x: i32, y: i32) -> Event
    {
        let old = self.cursor.get_position();
        self.cursor.set_position_xy(
            min(max(0, x), self.buf.get_buffer_width()),
            min(max(0, y), self.buf.get_real_buffer_height()));
        Event::CursorPositionChange(old, self.cursor.get_position())
    }

    pub fn save_content(&self, file_name: &Path)
    {
        let mut f = File::create(file_name).expect("Can't create file.");

        let content = 
            if let Some(ext) = file_name.extension() {
                let ext = OsStr::to_str(ext).unwrap().to_lowercase();
                self.get_file_content(ext.as_str())
            } else {
                self.get_file_content("")
            };
        
        f.write_all(&content).expect("Can't write file.");
    }

    pub fn get_file_content(&self, extension: &str) -> Vec<u8>
    {
        let options = SaveOptions::new();

        match extension {
            "bin" => convert_to_binary(&self.buf, &options).unwrap(),
            "xb" => convert_to_xb(&self.buf, &options).unwrap(),
            "ans" => convert_to_xb(&self.buf, &options).unwrap(),
            "avt" => convert_to_avt(&self.buf, &options).unwrap(),
            "pcb" => convert_to_pcb(&self.buf, &options).unwrap(),
            _ => convert_to_asc(&self.buf, &options).unwrap()
        }
    }
}