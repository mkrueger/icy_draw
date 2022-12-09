
use std::{rc::Rc, cell::RefCell};


use super::{Tool, Editor, Position, Event};
pub struct MoveLayer { pub pos: Position }

impl Tool for MoveLayer
{
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage { &super::icons::MOVE_SVG }
    fn use_caret(&self) -> bool { false }
    fn use_selection(&self) -> bool { false }

  /* 
    fn handle_drag_begin(&mut self, editor: Rc<RefCell<Editor>>, _start: Position, _cur: Position) -> Event {
        let mut editor = editor.borrow_mut();
        if let Some(layer) = editor.get_cur_layer() {
            self.pos = layer.get_offset();
        }
        Event::None
    }

    fn handle_drag(&self, editor: Rc<RefCell<Editor>>, start: Position, cur: Position) -> Event
    {
        let mut editor = editor.borrow_mut();
        if let Some(layer) = editor.get_cur_layer_mut() {
            layer.set_offset(self.pos + cur - start);
        }
        Event::None
    }*/

}