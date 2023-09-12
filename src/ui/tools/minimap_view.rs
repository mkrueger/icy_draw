use std::sync::{Arc, Mutex};

use eframe::{
    egui::{self, RichText},
    epaint::Vec2,
};
use i18n_embed_fl::fl;
use icy_engine::TextPane;
use icy_engine_egui::BufferView;

use crate::{AnsiEditor, Document, Message, ToolWindow};

pub struct MinimapToolWindow {
    buffer_view: Arc<eframe::epaint::mutex::Mutex<BufferView>>,
    undo_size: i32,
    last_id: usize,
}

impl ToolWindow for MinimapToolWindow {
    fn get_title(&self) -> String {
        fl!(crate::LANGUAGE_LOADER, "minimap_tool_title")
    }

    fn show_ui(
        &mut self,
        ui: &mut egui::Ui,
        active_document: Option<Arc<Mutex<Box<dyn Document>>>>,
    ) -> Option<Message> {
        if let Some(doc) = active_document {
            if let Some(editor) = doc.lock().unwrap().get_ansi_editor() {
                return self.show_minimap(ui, editor);
            }
        }
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            ui.label(RichText::new(fl!(crate::LANGUAGE_LOADER, "no_document_selected")).small());
        });
        None
    }
}

impl MinimapToolWindow {
    pub fn show_minimap(&mut self, ui: &mut egui::Ui, editor: &AnsiEditor) -> Option<Message> {
        let w = (ui.available_width() / 8.0).floor();

        let undo_stack = editor.buffer_view.lock().get_edit_state().undo_stack_len() as i32;
        if undo_stack != self.undo_size || self.last_id != editor.id {
            self.undo_size = undo_stack;
            self.last_id = editor.id;
            let bv = editor.buffer_view.lock();
            let buffer = bv.get_buffer();
            self.buffer_view
                .lock()
                .get_buffer_mut()
                .set_size(buffer.get_size());
            self.buffer_view.lock().get_buffer_mut().layers = buffer.layers.clone();
            self.buffer_view.lock().redraw_view();
        }

        self.buffer_view.lock().use_fg = editor.buffer_view.lock().use_fg;
        self.buffer_view.lock().use_bg = editor.buffer_view.lock().use_bg;

        let scalex = (w / self.buffer_view.lock().get_width() as f32).min(2.0);
        let scaley = if self.buffer_view.lock().get_buffer_mut().use_aspect_ratio() {
            scalex * 1.35
        } else {
            scalex
        };

        let opt = icy_engine_egui::TerminalOptions {
            focus_lock: false,
            stick_to_bottom: false,
            scale: Some(Vec2::new(scalex, scaley)),
            use_terminal_height: false,
            ..Default::default()
        };
        icy_engine_egui::show_terminal_area(ui, self.buffer_view.clone(), opt);
        None
    }

    pub(crate) fn new(gl: Arc<glow::Context>) -> Self {
        let mut buffer_view = BufferView::new(&gl, glow::NEAREST as i32);
        buffer_view.get_buffer_mut().is_terminal_buffer = true;
        buffer_view.get_caret_mut().is_visible = false;
        Self {
            buffer_view: Arc::new(eframe::epaint::mutex::Mutex::new(buffer_view)),
            last_id: usize::MAX,
            undo_size: -1,
        }
    }
}
