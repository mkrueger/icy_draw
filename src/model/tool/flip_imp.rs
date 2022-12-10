
use std::{rc::Rc, cell::RefCell};

use eframe::egui;

use super::{Tool, Editor, Position, Event};
pub struct FlipTool {}

impl Tool for FlipTool
{
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage { &super::icons::FILL_SVG }
    fn use_caret(&self) -> bool { false }
    fn use_selection(&self) -> bool { false }
    fn show_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, buffer_opt: Option<std::sync::Arc<std::sync::Mutex<crate::ui::ansi_editor::BufferView>>>)
    {
    }
    /*
    fn handle_click(&mut self, editor: Rc<RefCell<Editor>>, button: u32, pos: Position) -> Event {
        if button == 1 {
            let mut ch = editor.borrow().get_char(pos).unwrap_or_default();

            if ch.char_code == 222 {
                ch.char_code = 221;
            } else if ch.char_code == 221 {
                ch.char_code = 219;
            } else { 
                ch.char_code = 222;
            }
            
            editor.borrow_mut().set_char(pos, Some(ch));
        }
        Event::None
    }*/
} //   [176, 177, 178, 219, 223, 220, 221, 222, 254, 250 ],