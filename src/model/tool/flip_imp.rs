use std::sync::{Arc, Mutex};

use eframe::egui;

use crate::ansi_editor::BufferView;

use super::{Event, Position, Tool, ToolUiResult};
pub struct FlipTool {}

impl Tool for FlipTool {
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage {
        &super::icons::FILL_SVG
    }
    fn use_caret(&self) -> bool {
        false
    }
    fn use_selection(&self) -> bool {
        false
    }
    fn show_ui(
        &mut self,
        _ctx: &egui::Context,
        _ui: &mut egui::Ui,
        _buffer_opt: Option<std::sync::Arc<std::sync::Mutex<crate::ui::ansi_editor::BufferView>>>,
    ) -> ToolUiResult {
        ToolUiResult::new()
    }

    fn handle_click(
        &mut self,
        buffer_view: Arc<Mutex<BufferView>>,
        button: i32,
        pos: Position,
    ) -> Event {
        if button == 1 {
            let editor = &mut buffer_view.lock().unwrap().editor;
            let mut ch = editor.get_char(pos).unwrap_or_default();

            if ch.ch as u8 == 222 {
                ch.ch = '\u{00DD}';
            } else if ch.ch as u8 == 221 {
                ch.ch = '\u{00DB}';
            } else {
                ch.ch = '\u{00DE}';
            }

            editor.set_char(pos, Some(ch));
        }
        Event::None
    }
} //   [176, 177, 178, 219, 223, 220, 221, 222, 254, 250 ],
