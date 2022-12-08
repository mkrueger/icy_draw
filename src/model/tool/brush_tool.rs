use super::{Editor, Event, Position, Tool};

pub struct BrushTool {}

impl Tool for BrushTool
{
    fn get_icon_name(&self) -> &'static str { "edit-select" }
/* 
    fn add_tool_page(&self, window: &ApplicationWindow,parent: &mut gtk4::Box)
    {
        parent.append(&gtk4::Label::builder().label("BrushTool").build());
    }
    
    fn handle_click(&self, editor: &mut Editor, _button: u32, x: i32, y: i32) -> Event
    {
        editor.cursor.pos = Position::from(x, y);
        Event::None
    }

    fn handle_drag(&self, _editor: &mut Editor, _start: Position, _cur: Position) -> Event
    {
        Event::None
    }*/
}