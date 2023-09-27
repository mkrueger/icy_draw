use std::sync::{Arc, Mutex};

use eframe::{
    egui::{self, RichText},
    epaint::{Color32, Rect, Stroke, Vec2},
};
use i18n_embed_fl::fl;
use icy_engine::TextPane;
use icy_engine_egui::BufferView;

use crate::{AnsiEditor, Document, Message, ToolWindow};

pub struct MinimapToolWindow {
    buffer_view: Arc<eframe::epaint::mutex::Mutex<BufferView>>,
    undo_size: i32,
    last_id: usize,
    palette_hash: u32,
    next_scroll_pos: Option<Vec2>,
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
            if let Some(editor) = doc.lock().unwrap().get_ansi_editor_mut() {
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
    pub fn show_minimap(&mut self, ui: &mut egui::Ui, editor: &mut AnsiEditor) -> Option<Message> {
        let undo_stack = editor.buffer_view.lock().get_edit_state().undo_stack_len() as i32;
        let cur_palette_hash = editor
            .buffer_view
            .lock()
            .get_buffer_mut()
            .palette
            .get_hash();
        if undo_stack != self.undo_size
            || self.last_id != editor.id
            || self.palette_hash != cur_palette_hash
        {
            self.undo_size = undo_stack;
            self.last_id = editor.id;
            let bv = editor.buffer_view.lock();
            let buffer = bv.get_buffer();
            self.buffer_view
                .lock()
                .get_buffer_mut()
                .set_size(buffer.get_size());
            self.buffer_view.lock().get_buffer_mut().layers = buffer.layers.clone();
            self.buffer_view.lock().get_buffer_mut().palette = buffer.palette.clone();
            self.buffer_view
                .lock()
                .get_buffer_mut()
                .set_font_table(buffer.get_font_table());
            self.palette_hash = cur_palette_hash;
            self.buffer_view.lock().redraw_font();
            self.buffer_view.lock().redraw_view();
        }

        self.buffer_view.lock().use_fg = editor.buffer_view.lock().use_fg;
        self.buffer_view.lock().use_bg = editor.buffer_view.lock().use_bg;
        let w = (ui.available_width()
            / self
                .buffer_view
                .lock()
                .get_buffer()
                .get_font_dimensions()
                .width as f32)
            .floor();

        let scalex = (w / self.buffer_view.lock().get_width() as f32).min(2.0);
        let scaley = if self.buffer_view.lock().get_buffer_mut().use_aspect_ratio() {
            scalex * 1.35
        } else {
            scalex
        };

        let mut opt = icy_engine_egui::TerminalOptions {
            filter: glow::LINEAR as i32,
            stick_to_bottom: false,
            scale: Some(Vec2::new(scalex, scaley)),
            use_terminal_height: false,

            ..Default::default()
        };

        let next_scroll_pos = self.next_scroll_pos.take();

        if let Some(next_scroll_pos) = next_scroll_pos {
            opt.scroll_offset_x = Some(next_scroll_pos.x);
            opt.scroll_offset_y = Some(next_scroll_pos.y);
        }

        let (response, ours) =
            icy_engine_egui::show_terminal_area(ui, self.buffer_view.clone(), opt);

        let theirs = editor.buffer_view.lock().calc.clone();

        let their_total_size = Vec2::new(theirs.char_width, theirs.char_height) * theirs.char_size;
        let their_buffer_size =
            Vec2::new(theirs.buffer_char_width, theirs.buffer_char_height) * theirs.char_size;

        let our_total_size = Vec2::new(ours.char_width, ours.char_height) * ours.char_size;

        let tmax_y: f32 =
            theirs.font_height * (theirs.char_height - theirs.buffer_char_height).max(0.0);
        let tmax_x: f32 =
            theirs.font_width * (theirs.real_width as f32 - theirs.buffer_char_width).max(0.0);

        let size = our_total_size * their_buffer_size / their_total_size;
        let tx = theirs.char_scroll_positon.x / tmax_x.max(1.0);
        let ty = theirs.char_scroll_positon.y / tmax_y.max(1.0);

        let pos = (our_total_size - size - Vec2::new(2.0, 2.0)) * Vec2::new(tx, ty);

        let pos = pos - ours.char_scroll_positon * ours.scale;

        ui.painter().rect_stroke(
            Rect::from_min_size(ours.buffer_rect.min + pos, size),
            0.0,
            Stroke::new(3.0, Color32::from_rgba_premultiplied(155, 155, 155, 90)),
        );
        ui.painter().rect_stroke(
            Rect::from_min_size(ours.buffer_rect.min + pos, size),
            0.0,
            Stroke::new(1.0, Color32::WHITE),
        );

        if pos.x < 0.0 || pos.y < 0.0 {
            self.next_scroll_pos = Some(ours.char_scroll_positon + pos / ours.scale);
            ui.ctx().request_repaint();
        }

        if pos.x + size.x > ours.terminal_rect.size().x
            || pos.y + size.y > ours.terminal_rect.size().y
        {
            let p = pos + size - ours.terminal_rect.size();
            self.next_scroll_pos = Some(ours.char_scroll_positon + p / ours.scale);
            ui.ctx().request_repaint();
        }

        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let pos = (pos - ours.buffer_rect.min) / ours.scale + ours.char_scroll_positon;
                editor.next_scroll_x_position =
                    Some(pos.x - theirs.buffer_char_width * theirs.font_width / 2.0);
                editor.next_scroll_y_position =
                    Some(pos.y - theirs.buffer_char_height * theirs.font_height / 2.0);
                ui.ctx().request_repaint();
            }
        }

        None
    }

    pub(crate) fn new(gl: Arc<glow::Context>) -> Self {
        let mut buffer_view = BufferView::new(&gl);
        buffer_view.interactive = false;
        buffer_view.get_buffer_mut().is_terminal_buffer = true;
        buffer_view.get_caret_mut().is_visible = false;
        Self {
            buffer_view: Arc::new(eframe::epaint::mutex::Mutex::new(buffer_view)),
            last_id: usize::MAX,
            undo_size: -1,
            palette_hash: 0,
            next_scroll_pos: None,
        }
    }
}
