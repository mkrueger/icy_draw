use icy_engine::TextAttribute;

use super::{plot_point, DrawMode, Editor, Event, Plottable, Position, Tool, ScanLines };
use std::{
    cell::RefCell,
    rc::Rc,
};

pub struct DrawEllipseFilledTool {
    pub draw_mode: DrawMode,

    pub use_fore: bool,
    pub use_back: bool,
    pub attr: TextAttribute,
    pub char_code: u16,
}

impl Plottable for DrawEllipseFilledTool {
    fn get_draw_mode(&self) -> DrawMode {
        self.draw_mode
    }
    fn get_use_fore(&self) -> bool {
        self.use_fore
    }
    fn get_use_back(&self) -> bool {
        self.use_back
    }
    fn get_char_code(&self) -> u16 {
        self.char_code
    }
}

impl Tool for DrawEllipseFilledTool {
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage { &super::icons::ELLIPSE_FILLED_SVG }

    fn use_caret(&self) -> bool {
        false
    }
    fn use_selection(&self) -> bool {
        false
    }
/*
    fn handle_drag(&self, editor: Rc<RefCell<Editor>>, start: Position, cur: Position) -> Event {
        if let Some(layer) = editor.borrow_mut().get_overlay_layer() {
            layer.clear();
        }

        let mut lines = ScanLines::new(1);

        if start < cur {
            lines.add_ellipse(Rectangle::from_pt( start, cur));
        } else {
            lines.add_ellipse(Rectangle::from_pt(cur, start));
        }

        let draw = move |rect: Rectangle| {
            for y in 0..rect.size.height {
                for x in 0..rect.size.width {
                    plot_point(&editor, self, Position::from(rect.start.x + x, rect.start.y + y));
                }
            }
        };
        lines.fill(draw);
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
    } */
}
