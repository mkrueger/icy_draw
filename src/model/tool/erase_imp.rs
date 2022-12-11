use std::{cell::{RefCell}, rc::Rc};

use eframe::egui;
use i18n_embed_fl::fl;

use super::{ Tool, Editor, Position};

#[derive(PartialEq, Eq)]
pub enum EraseType {
    Shade,
    Solid
}

pub struct EraseTool {
    pub size: i32,
    pub brush_type: EraseType
}

impl EraseTool {
/* 
    fn paint_brush(&self, editor: &Rc<RefCell<Editor>>, pos: Position)
    {
        let mid = Position::from(-(self.size / 2), -(self.size / 2));

        let center = pos + mid;
        let gradient = [ 219, 178, 177, 176, b' '];
        let mut editor = editor.borrow_mut();
        editor.begin_atomic_undo();

        for y in 0..self.size {
            for x in 0..self.size {
                match self.brush_type {
                    EraseType::Shade => {    
                        let ch = editor.get_char_from_cur_layer(center + Position::from(x, y)).unwrap_or_default();
                       
                        let mut attribute= ch.attribute;

                        let mut char_code = gradient[0];
                        let mut found = false;
                        if ch.char_code as u8 == gradient[gradient.len() -1] {
                            char_code = gradient[gradient.len() -1];
                            attribute = TextAttribute::DEFAULT;
                            found = true;
                        } else {
                            for i in 0..gradient.len() - 1 {
                                if ch.char_code as u8 == gradient[i] {
                                    char_code = gradient[i + 1];
                                    found = true;
                                    break;
                                }
                            }
                        }

                        if found {
                            editor.set_char(center + Position::from(x, y), Some(crate::model::DosChar { 
                                char_code: char_code as u16, 
                                attribute
                            }));
                        }
                    },
                    EraseType::Solid => {
                        editor.set_char(center + Position::from(x, y), Some(crate::model::DosChar { char_code: b' ' as u16, attribute: TextAttribute::DEFAULT }));
                    }
                }
            }                
        }
        editor.end_atomic_undo();

    }
    
*/
}

impl Tool for EraseTool
{
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage { &super::icons::ERASER_SVG }
   
    fn use_caret(&self) -> bool { false }

    fn show_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, buffer_opt: Option<std::sync::Arc<std::sync::Mutex<crate::ui::ansi_editor::BufferView>>>)
    {
        ui.horizontal(|ui| {
            ui.label(fl!(crate::LANGUAGE_LOADER, "tool-size-label"));
            ui.add(egui::DragValue::new(&mut self.size).clamp_range(1..=20).speed(1));
        });
        ui.radio_value(&mut self.brush_type, EraseType::Solid, fl!(crate::LANGUAGE_LOADER, "tool-solid"));
        ui.radio_value(&mut self.brush_type, EraseType::Shade, fl!(crate::LANGUAGE_LOADER, "tool-shade"));
    }
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
