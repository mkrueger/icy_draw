use std::{cell::{RefCell}, rc::Rc};
use egui_extras::RetainedImage;

use super::{ Tool, Editor, Position};

pub enum BrushType {
    Shade,
    Solid,
    Color
}

pub struct BrushTool {
    pub use_fore: bool,
    pub use_back: bool,
    pub size: i32,
    pub char_code: u16,

    pub brush_type: BrushType
}

impl BrushTool {
/* 
    fn paint_brush(&self, editor: &Rc<RefCell<Editor>>, pos: Position)
    {
        let mid = Position::from(-(self.size / 2), -(self.size / 2));

        let center = pos + mid;
        let gradient = [176, 177, 178, 219];
        let mut editor = editor.borrow_mut();
        editor.begin_atomic_undo();

        for y in 0..self.size {
            for x in 0..self.size {
                match self.brush_type {
                    BrushType::Shade => {    
                        let ch = editor.get_char_from_cur_layer(center + Position::from(x, y)).unwrap_or_default();
                       
                        let attribute= editor.caret.get_attribute();

                        let mut char_code = gradient[0];
                        if ch.char_code == gradient[gradient.len() -1] {
                            char_code = gradient[gradient.len() -1];
                        } else {
                            for i in 0..gradient.len() - 1 {
                                if ch.char_code == gradient[i] {
                                    char_code = gradient[i + 1];
                                    break;
                                }
                            }
                        }
                        editor.set_char(center + Position::from(x, y), Some(crate::model::DosChar { 
                            char_code, 
                            attribute
                        }));

                    },
                    BrushType::Solid => {
                        let attribute= editor.caret.get_attribute();
                        editor.set_char(center + Position::from(x, y), Some(crate::model::DosChar { char_code: self.char_code, attribute }));
                    },
                    BrushType::Color => {
                        let ch = editor.get_char_from_cur_layer(center + Position::from(x, y)).unwrap_or_default();
                        let mut attribute = ch.attribute;

                        if self.use_fore {
                            attribute.set_foreground(editor.caret.get_attribute().get_foreground());
                        }
                        if self.use_back {
                            attribute.set_background(editor.caret.get_attribute().get_background());
                        }

                        editor.set_char(center + Position::from(x, y), Some(crate::model::DosChar { 
                            char_code:ch.char_code, 
                            attribute
                        }));
                    },
                }
            }                
        }
        editor.end_atomic_undo();

    }
    
*/
}

impl Tool for BrushTool
{
    fn get_icon_name(&self) -> &'static RetainedImage { &super::icons::BRUSH_SVG }
    fn use_caret(&self) -> bool { false }
/* 
    fn handle_click(&mut self, editor: Rc<RefCell<Editor>>, button: u32, pos: Position) -> super::Event {
        if button == 1 {
            self.paint_brush(&editor, pos);
        }
        super::Event::None
    }

    fn handle_drag(&self, editor: Rc<RefCell<Editor>>, _start: Position, cur: Position) -> super::Event
    {
        self.paint_brush(&editor, cur);
        super::Event::None
    }*/
}