
use std::{sync::{Arc, Mutex}};

use eframe::egui;
use egui_extras::RetainedImage;

use crate::{ansi_editor::BufferView};

use super::{Event, Position, Tool};

pub struct ClickTool {}

impl Tool for ClickTool
{
    fn get_icon_name(&self) -> &'static RetainedImage { &super::icons::CURSOR_SVG }

    fn show_ui(&mut self, _ctx: &egui::Context, _ui: &mut egui::Ui, _buffer_opt: Option<std::sync::Arc<std::sync::Mutex<crate::ui::ansi_editor::BufferView>>>)
    {
    }

    fn handle_click(&mut self, buffer_view: Arc<Mutex<BufferView>>, button: i32, pos: Position) -> Event {
        if button == 1 {
            let editor = &mut buffer_view.lock().unwrap().editor;
            editor.set_caret_position(pos);
            editor.cur_selection = None;
        }
        Event::None
    }
/* 
    fn handle_drag(&self, buffer_view: Arc<Mutex<BufferView>>, start: Position, cur: Position) -> Event
    {
        let mut editor = editor.borrow_mut();
        let mut cur = cur;
        if start < cur {
            cur = cur + Position::from(1, 1);
        }
        if start == cur { 
            editor.cur_selection = None;
        } else {
            editor.cur_selection = Some(Selection { 
                rectangle: crate::model::Rectangle::from_pt(start, cur),
                is_preview: true,
                shape: crate::model::Shape::Rectangle
            });
        }
        editor.set_caret_position(cur);
        Event::None
    }

    fn handle_drag_end(&self, buffer_view: Arc<Mutex<BufferView>>, start: Position, cur: Position) -> Event {
        let mut editor = editor.borrow_mut();
        let mut cur = cur;
        if start < cur {
            cur = cur + Position::from(1, 1);
        }

        if start == cur { 
            editor.cur_selection = None;
        } else {
            editor.cur_selection = Some(Selection { 
                rectangle: crate::model::Rectangle::from_pt(start, cur),
                is_preview: false,
                shape: crate::model::Shape::Rectangle
            });
        }
        editor.set_caret_position(cur);

        Event::None
    } */
}
