use std::sync::{Arc, Mutex};

use eframe::egui;
use egui_extras::RetainedImage;
use icy_engine::Rectangle;

use super::{Event, Position, Tool, ToolUiResult};

pub struct ClickTool {}

impl Tool for ClickTool {
    fn get_icon_name(&self) -> &'static RetainedImage {
        &super::icons::CURSOR_SVG
    }

    fn show_ui(
        &mut self,
        _ctx: &egui::Context,
        _ui: &mut egui::Ui,
        _buffer_opt: Option<std::sync::Arc<std::sync::Mutex<BufferView>>>,
    ) -> ToolUiResult {
        ToolUiResult::default()
    }

    fn handle_click(
        &mut self,
        buffer_view: Arc<Mutex<BufferView>>,
        button: i32,
        pos: Position,
    ) -> Event {
        if button == 1 {
            let editor = &mut buffer_view.lock().editor;
            editor.set_caret_position(pos);
            editor.cur_selection = None;
        }
        Event::None
    }

    fn handle_drag(
        &mut self,
        buffer_view: Arc<Mutex<BufferView>>,
        start: Position,
        cur: Position,
    ) -> Event {
        let editor = &mut buffer_view.lock().editor;
        let mut cur = cur;
        if start < cur {
            cur = cur + Position::new(1, 1);
        }
        if start == cur {
            editor.cur_selection = None;
        } else {
            editor.cur_selection = Some(Selection {
                rectangle: Rectangle::from_pt(start, cur),
                is_preview: true,
                shape: Shape::Rectangle,
            });
        }
        editor.set_caret_position(cur);
        Event::None
    }

    fn handle_drag_end(
        &mut self,
        buffer_view: Arc<Mutex<BufferView>>,
        start: Position,
        cur: Position,
    ) -> Event {
        let editor = &mut buffer_view.lock().editor;
        let mut cur = cur;
        if start < cur {
            cur = cur + Position::new(1, 1);
        }

        if start == cur {
            editor.cur_selection = None;
        } else {
            editor.cur_selection = Some(Selection {
                rectangle: Rectangle::from_pt(start, cur),
                is_preview: false,
                shape: Shape::Rectangle,
            });
        }
        editor.set_caret_position(cur);

        Event::None
    }
}
