use std::{cell::{RefCell, RefMut}, rc::Rc, collections::{HashSet}};

use icy_engine::TextAttribute;

use super::{ Tool, Editor, Position, Event};

pub struct FillTool {
    pub use_char : bool,
    pub use_fore : bool,
    pub use_back : bool,

    pub attr: TextAttribute,
    pub char_code: u16
}

// Fill with 
// Attribute, Fore/Back
// Character 
// Both

impl Tool for FillTool
{
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage { &super::icons::FILL_SVG }
    fn use_caret(&self) -> bool { false }
   /*
    fn handle_click(&mut self, editor: Rc<RefCell<Editor>>, button: u32, pos: Position) -> Event {
        if button == 1 {
            let mut editor = editor.borrow_mut();
            if editor.cur_layer >= editor.buf.layers.len() as i32 { return Event::None; }
            let attr = editor.caret.get_attribute();
            let ch = editor.buf.get_char(pos);
            if self.use_back || self.use_fore || self.use_char {
                editor.begin_atomic_undo();
                let mut visited = HashSet::new();
                fill(&mut editor, &mut visited, attr, pos, ch,  DosChar{ char_code: self.char_code, attribute: attr });
                editor.end_atomic_undo();

            }
        }
        Event::None
    }*/
}
/*
fn fill(editor: &mut RefMut<Editor>, visited: &mut HashSet<Position>, attribute: TextAttribute, pos: Position, opt_old_ch: Option<DosChar>, new_ch: DosChar) {
    if !editor.point_is_valid(pos) || !visited.insert(pos) {
        return;
    }

    let cur_char = editor.buf.get_char(pos).unwrap_or_default();
    unsafe {
        if let Some(old_ch) = opt_old_ch {
            if FILL_TOOL.use_char && FILL_TOOL.use_fore && FILL_TOOL.use_back {
                if cur_char != old_ch || cur_char == new_ch {
                    return;
                }
            } else if FILL_TOOL.use_fore && FILL_TOOL.use_back {
                if cur_char.attribute != old_ch.attribute || cur_char.attribute == new_ch.attribute {
                    return;
                }
            } else if FILL_TOOL.use_char && FILL_TOOL.use_fore  {
                if cur_char.char_code != old_ch.char_code && cur_char.attribute.get_foreground() != old_ch.attribute.get_foreground() || 
                cur_char.char_code == new_ch.char_code && cur_char.attribute.get_foreground() == new_ch.attribute.get_foreground() {
                    return;
                }
            } else if FILL_TOOL.use_char && FILL_TOOL.use_back  {
                if cur_char.char_code != old_ch.char_code && cur_char.attribute.get_background() != old_ch.attribute.get_background() || 
                cur_char.char_code == new_ch.char_code && cur_char.attribute.get_background() == new_ch.attribute.get_background() {
                    return;
                }
            } else if FILL_TOOL.use_char {
                if cur_char.char_code != old_ch.char_code || cur_char.char_code == new_ch.char_code {
                    return;
                }
            } else if FILL_TOOL.use_fore  {
                if cur_char.attribute.get_foreground() != old_ch.attribute.get_foreground() || cur_char.attribute.get_foreground() == new_ch.attribute.get_foreground() {
                    return;
                }
            } else if FILL_TOOL.use_back {
                if cur_char.attribute.get_background() != old_ch.attribute.get_background()  || cur_char.attribute.get_background() == new_ch.attribute.get_background() {
                    return;
                }
            } else {
                panic!("should never happen!");
            }
           
        }
        let mut repl_ch = cur_char;
        if FILL_TOOL.use_char { repl_ch.char_code = new_ch.char_code; }
        if FILL_TOOL.use_fore { repl_ch.attribute.set_foreground(new_ch.attribute.get_foreground()) }
        if FILL_TOOL.use_back { repl_ch.attribute.set_background(new_ch.attribute.get_background()) }

        editor.set_char(pos, Some(repl_ch));
    }
    fill(editor, visited, attribute, pos + Position::from(-1, 0), opt_old_ch, new_ch);
    fill(editor, visited, attribute, pos + Position::from(1, 0), opt_old_ch, new_ch);
    fill(editor, visited, attribute, pos + Position::from(    0, -1), opt_old_ch, new_ch);
    fill(editor, visited, attribute, pos + Position::from(0, 1), opt_old_ch, new_ch);
} */