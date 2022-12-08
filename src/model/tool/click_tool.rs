
use std::{rc::Rc, cell::RefCell};

use super::{Editor, Event, Position, Tool};


pub struct ClickTool {}

impl Tool for ClickTool
{
    fn get_icon_name(&self) -> &'static str { "edit-select" }
/* 
    fn add_tool_page(&self, window: &ApplicationWindow,parent: &mut gtk4::Box)
    {
        parent.append(&gtk4::Label::builder().label("Click").build());
    }
*/


    fn handle_click(&self, editor: Rc<RefCell<Editor>>, button: u32, pos: Position) -> Event {

        if button == 1 {
            editor.borrow_mut().cursor.pos = pos;
        }
        Event::None
    }
}
