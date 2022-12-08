use crate::model::TheDrawFont;

use super::{Tool, Editor, Event, Position};
use walkdir::{DirEntry, WalkDir};
pub struct FontTool {
    pub selected_font: i32,
    pub fonts: Vec<TheDrawFont>
}

impl FontTool 
{
    fn is_hidden(entry: &DirEntry) -> bool {
        entry.file_name()
             .to_str()
             .map(|s| s.starts_with("."))
             .unwrap_or(false)
    }
        
    pub fn load_fonts(&mut self)
    {
        let walker = WalkDir::new("/home/mkrueger/Dokumente/THEDRAWFONTS").into_iter();
        for entry in walker.filter_entry(|e| !FontTool::is_hidden(e)) {
             let entry = entry.unwrap();
             let path = entry.path();

             if path.is_dir() {
                continue;
             }
             let extension = path.extension();
             if extension.is_none() { continue; }
             let extension = extension.unwrap().to_str();
             if extension.is_none() { continue; }
             let extension = extension.unwrap().to_lowercase();

             if extension == "tdf" {
                if let Some(font) = TheDrawFont::load(path) {
                    self.fonts.push(font);
                }
            }
        }

        println!("{} fonts read.", self.fonts.len());
    }
}

impl Tool for FontTool
{
    fn get_icon_name(&self) -> &'static str { "edit-select" }
/*
    fn add_tool_page(&self, window: &ApplicationWindow,parent: &mut gtk4::Box)
    {
        crate::ui::add_font_tool_page(window, parent);
    }
    
    fn handle_click(&self, editor: &mut Editor, _button: u32, x: i32, y: i32) -> Event
    {
        editor.cursor.pos = Position::from(x, y);
        Event::None
    }

    fn handle_key(&self, editor: &mut Editor, key: Key, _key_code: u32, _modifier: ModifierType) -> Event
    {
        if self.selected_font < 0 || self.selected_font >= self.fonts.len() as i32 {
            return Event::None;
        }
        let font = &self.fonts[self.selected_font as usize];

        match key {
            Key::Down => {
                editor.set_cursor(editor.cursor.pos.x, editor.cursor.pos.y + 1);
            }
            Key::Up => {
                editor.set_cursor(editor.cursor.pos.x, editor.cursor.pos.y - 1);
            }
            Key::Left => {
                editor.set_cursor(editor.cursor.pos.x - 1, editor.cursor.pos.y);
            }
            Key::Right => {
                editor.set_cursor(editor.cursor.pos.x + 1, editor.cursor.pos.y);
            }
            
            Key::Page_Down |
            Key::Page_Up => {
                // TODO
            }
            
            Key::Home | Key::KP_Home => {
                editor.set_cursor(0, editor.cursor.pos.y);
            }
            
            Key::End | Key::KP_End => {
                editor.set_cursor(editor.buf.width as i32 - 1, editor.cursor.pos.y);
            }

            Key::Return | Key::KP_Enter => {
                editor.set_cursor(0,editor.cursor.pos.y + font.get_font_height() as i32);
            }

            _ => { 
                if let Some(key) = key.to_unicode() {
                    if key.len_utf8() == 1 {
                        let mut dst = [0];
                        key.encode_utf8(&mut dst);

                        let width = font.render(&mut editor.buf, editor.cursor.pos, editor.cursor.attr, dst[0]);
                        if width > 0 {
                            editor.set_cursor(editor.cursor.pos.x + width + font.spaces, editor.cursor.pos.y);
                        } else {
                            editor.buf.set_char(editor.cursor.pos, crate::model::DosChar {
                                char_code: dst[0],
                                attribute: editor.cursor.attr,
                            });
                            editor.set_cursor(editor.cursor.pos.x + 1, editor.cursor.pos.y);
                        }
                    }
                }
            }
        }
        Event::None
    }*/

}