use eframe::egui;
use egui_extras::RetainedImage;
use icy_engine::Selection;

use crate::AnsiEditor;

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
        _buffer_opt: &mut AnsiEditor,
    ) -> ToolUiResult {
        ToolUiResult::default()
    }

    fn handle_click(&mut self, editor: &mut AnsiEditor, button: i32, pos: Position) -> Event {
        if button == 1 {
            editor.set_caret_position(pos);
            editor.cur_selection = None;
        }
        Event::None
    }

    fn handle_drag(&mut self, editor: &mut AnsiEditor, start: Position, cur: Position) -> Event {
        let mut cur = cur;
        if start < cur {
            cur = cur + Position::new(1, 1);
        }
        if start == cur {
            editor.buffer_view.lock().clear_selection();
        } else {
            editor
                .buffer_view
                .lock()
                .set_selection(Selection::from_rectangle(
                    start.x as f32,
                    start.y as f32,
                    cur.x as f32,
                    cur.y as f32,
                ));
        }
        editor.set_caret_position(cur);
        Event::None
    }

    fn handle_drag_end(
        &mut self,
        editor: &mut AnsiEditor,
        start: Position,
        cur: Position,
    ) -> Event {
        let mut cur = cur;
        if start < cur {
            cur = cur + Position::new(1, 1);
        }

        if start == cur {
            editor.cur_selection = None;
        } else {
            editor
                .buffer_view
                .lock()
                .set_selection(Selection::from_rectangle(
                    start.x as f32,
                    start.y as f32,
                    cur.x as f32,
                    cur.y as f32,
                ));
        }
        editor.set_caret_position(cur);

        Event::None
    }
}
