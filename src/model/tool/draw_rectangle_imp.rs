
use icy_engine::TextAttribute;

use super::{Editor, Event, Position, Tool, DrawMode, Plottable, ScanLines};
use std::{
    cell::{RefCell},
    rc::Rc,
};

pub struct DrawRectangleTool {
    pub draw_mode: DrawMode,

    pub use_fore: bool,
    pub use_back: bool,
    pub attr: TextAttribute,
    pub char_code: u16
}

impl Plottable for DrawRectangleTool {
    fn get_draw_mode(&self) -> DrawMode { self.draw_mode }

    fn get_use_fore(&self) -> bool { self.use_fore }
    fn get_use_back(&self) -> bool { self.use_back }
    fn get_char_code(&self) -> u16 { self.char_code }
}

impl Tool for DrawRectangleTool {
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage { &super::icons::RECTANGLE_OUTLINE_SVG }
    fn use_caret(&self) -> bool { false }
    fn use_selection(&self) -> bool { false }
/* 
    fn handle_drag(&self, editor: Rc<RefCell<Editor>>,  mut start: Position, mut cur: Position) -> Event {
        if let Some(layer) = editor.borrow_mut().get_overlay_layer() {
            layer.clear();
        }

        if self.draw_mode == DrawMode::Line {
            start.y *= 2;
            cur.y *= 2;
        }

        let mut lines = ScanLines::new(1);
        lines.add_rectangle(Rectangle::from_pt(start, cur));

        let col = editor.borrow().caret.get_attribute().get_foreground();
        let draw = move |rect: Rectangle| {
            for y in 0..rect.size.height {
                for x in 0..rect.size.width {
                    set_half_block(&editor, Position::from(rect.start.x + x, rect.start.y + y ), col);
                }
            }
        };
        lines.outline(draw);

        Event::None
    }

    fn handle_drag_end(
        &self,
        editor: Rc<RefCell<Editor>>,
        start: Position,
        cur: Position,
    ) -> Event {
        let mut editor = editor.borrow_mut();
        if start == cur {
            editor.buf.remove_overlay();
        } else {
            editor.join_overlay();
        }
        Event::None
    }

    */
}
