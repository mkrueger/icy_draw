use std::{rc::Rc, cell::RefCell};

use eframe::egui;

use super::{Tool, Editor, Position, Event};
pub struct PipetteTool {}

impl Tool for PipetteTool
{
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage { &super::icons::DROPPER_SVG }
    fn use_caret(&self) -> bool { false }
    fn use_selection(&self) -> bool { false }
    fn show_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, buffer_opt: Option<std::sync::Arc<std::sync::Mutex<crate::ui::ansi_editor::BufferView>>>)
    {
    }
    /* 
    fn handle_click(&mut self, editor: Rc<RefCell<Editor>>, button: u32, pos: Position) -> Event {
        if button == 1 {
            let ch = editor.borrow().get_char(pos).unwrap_or_default();
            editor.borrow_mut().set_caret_attribute(ch.attribute);
        }
        Event::None
    }*/
}