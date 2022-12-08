use std::{cmp::{max}, fs, path::PathBuf, sync::Arc};
use eframe::{epaint::{Vec2, Rect, mutex::Mutex}, egui::{CursorIcon, self, ScrollArea, PointerButton}};
use icy_engine::{Buffer, SaveOptions, AnsiParser, Selection, BufferParser};

pub mod render;
pub use render::*;

pub mod sixel;
pub use sixel::*;

pub mod buffer_view;
pub use buffer_view::*;

pub mod key_maps;
pub use key_maps::*;

use crate::{Document, TerminalResult};


pub struct AnsiEditor {
    is_dirty: bool,
    buffer_view: Arc<Mutex<BufferView>>,
    buffer_parser: Box<dyn BufferParser>
}

impl AnsiEditor {
    pub fn new(gl: &Arc<glow::Context>, buf: Buffer) -> Self {
        let buffer_view = Arc::new(Mutex::new(BufferView::new(gl, buf)));
        let mut buffer_parser = AnsiParser::new();

        Self {
            is_dirty: false,
            buffer_view,
            buffer_parser: Box::new(buffer_parser)
        }
    }

    pub fn output_string(&mut self, str: &str) {
        for ch in str.chars() {
            let translated_char = self.buffer_parser.from_unicode(ch);
            if let Err(err) = self.print_char(translated_char as u8) {
                eprintln!("{}", err);
            }
        }
    }

    pub fn print_char(&mut self, c: u8) -> Result<(), Box<dyn std::error::Error>> {
        let result = self
            .buffer_view
            .lock()
            .print_char(&mut self.buffer_parser, unsafe {
                char::from_u32_unchecked(c as u32)
            })?;
        self.buffer_view.lock().redraw_view();
        Ok(())
    }


}

impl Document for AnsiEditor {
    fn get_title(&self) -> String {
        if let Some(file_name) = &self.buffer_view.lock().buf.file_name {
            file_name.file_name().unwrap().to_str().unwrap().to_string()
        } else {
            "Untitled".to_string()
        }
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn save(&mut self, file_name: &str) -> TerminalResult<()> {
        let file =  PathBuf::from(file_name);
        let options = SaveOptions::new();
        let bytes = self.buffer_view.lock().buf.to_bytes(file.extension().unwrap().to_str().unwrap(), &options)?;
        fs::write(file_name, bytes)?;
        self.is_dirty = false;
        Ok(())
    }

    fn show_ui(&mut self, ui: &mut eframe::egui::Ui) {
        let size = ui.max_rect().size();
        let buf_w = self.buffer_view.lock().buf.get_buffer_width();
        let buf_h = self.buffer_view.lock().buf.get_buffer_height();
        // let h = max(buf_h, buffer_view.lock().buf.get_real_buffer_height());
        let font_dimensions = self.buffer_view.lock().buf.get_font_dimensions();

        let mut scale_x = (size.x - 4.0) / font_dimensions.width as f32 / buf_w as f32;
        let mut scale_y = size.y / font_dimensions.height as f32 / buf_h as f32;

        if scale_x < scale_y {
            scale_y = scale_x;
        } else {
            scale_x = scale_y;
        }

        let char_size = Vec2::new(
            font_dimensions.width as f32 * scale_x,
            font_dimensions.height as f32 * scale_y,
        );

        let rect_w = buf_w as f32 * char_size.x;
        let rect_h = buf_h as f32 * char_size.y;
        let top_margin_height = ui.min_rect().top();

        let output = ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show_viewport(ui, |ui, viewport| {
                let (draw_area, mut response) = ui.allocate_at_least(size, egui::Sense::click());

                let rect = Rect::from_min_size(
                    draw_area.left_top()
                        + Vec2::new(
                            (-4.0 + draw_area.width() - rect_w) / 2.,
                            (-top_margin_height
                                + viewport.top()
                                + (draw_area.height() - rect_h) / 2.)
                                .floor(),
                        )
                        .ceil(),
                    Vec2::new(rect_w, rect_h),
                );
                let real_height = self.buffer_view.lock().buf.get_real_buffer_height();
                let max_lines = max(0, real_height - buf_h);
                ui.set_height(scale_y * max_lines as f32 * font_dimensions.height as f32);

                let first_line = (viewport.top() / char_size.y) as i32;
                let scroll_back_line = max(0, max_lines - first_line);

                if scroll_back_line != self.buffer_view.lock().scroll_back_line {
                    self.buffer_view.lock().scroll_back_line = scroll_back_line;
                    self.buffer_view.lock().redraw_view();
                }
                
                let buffer_view  = self.buffer_view.clone();
                let callback = egui::PaintCallback {
                    rect,
                    callback: std::sync::Arc::new(egui_glow::CallbackFn::new(
                        move |_info, painter| {
                            buffer_view.lock().update_buffer(painter.gl());
                            buffer_view.lock().paint(painter.gl(), rect);
                        },
                    )),
                };

                ui.painter().add(callback);
                response = response.context_menu(terminal_context_menu);

                let events = ui.input().events.clone();
                for e in &events {
                    match e {
                        egui::Event::PointerButton {
                            button: PointerButton::Middle,
                            pressed: true,
                            ..
                        }
                        | egui::Event::Copy => {
                            let buffer_view = self.buffer_view.clone();
                            let mut l = buffer_view.lock();
                            if let Some(txt) = l.get_copy_text(&self.buffer_parser) {
                                ui.output().copied_text = txt;
                            }
                        }
                        egui::Event::Cut => {}
                        egui::Event::Paste(text) => {
                            self.output_string(text);
                        }
                        egui::Event::CompositionEnd(text) | egui::Event::Text(text) => {
                            self.output_string(text);
                            response.mark_changed();
                        }

                        egui::Event::PointerButton {
                            pos,
                            button: PointerButton::Primary,
                            pressed: true,
                            modifiers,
                        } => {
                            if rect.contains(*pos) {
                                let buffer_view = self.buffer_view.clone();
                                let click_pos =
                                    (*pos - rect.min - Vec2::new(0., top_margin_height))
                                        / char_size
                                        + Vec2::new(0.0, first_line as f32);
                                buffer_view.lock().selection_opt =
                                    Some(Selection::new((click_pos.x, click_pos.y)));
                                buffer_view
                                    .lock()
                                    .selection_opt
                                    .as_mut()
                                    .unwrap()
                                    .block_selection = modifiers.alt;
                            }
                        }

                        egui::Event::PointerButton {
                            button: PointerButton::Primary,
                            pressed: false,
                            ..
                        } => {
                            let buffer_view = self.buffer_view.clone();
                            let mut l = buffer_view.lock();
                            if let Some(sel) = &mut l.selection_opt {
                                sel.locked = true;
                            }
                        }

                        egui::Event::PointerMoved(pos) => {
                            let buffer_view = self.buffer_view.clone();
                            let mut l = buffer_view.lock();
                            if let Some(sel) = &mut l.selection_opt {
                                if !sel.locked {
                                    let click_pos =
                                        (*pos - rect.min - Vec2::new(0., top_margin_height))
                                            / char_size
                                            + Vec2::new(0.0, first_line as f32);
                                    sel.set_lead((click_pos.x, click_pos.y));
                                    sel.block_selection = ui.input().modifiers.alt;
                                    l.redraw_view();
                                }
                            }
                        }
                        /*egui::Event::KeyRepeat { key, modifiers }
                        | */egui::Event::Key {
                            key,
                            pressed: true,
                            modifiers,
                        } => {
                            let mut key_code = *key as u32;
                            if modifiers.ctrl || modifiers.command {
                                key_code |= CTRL_MOD;
                            }
                            if modifiers.shift {
                                key_code |= SHIFT_MOD;
                            }
                            for (k, m) in ANSI_KEY_MAP {
                                if *k == key_code {
                                    //self.handled_char = true;
                                    for c in *m {
                                        if let Err(err) = self.print_char(*c) {
                                            eprintln!("{}", err);
                                        }
                                    }
                                    response.mark_changed();
                                    ui.input_mut().consume_key(*modifiers, *key);
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                } 
                if response.hovered() {
                    let hover_pos_opt = ui.input().pointer.hover_pos();
                    if let Some(hover_pos) = hover_pos_opt {
                        if rect.contains(hover_pos) {
                            ui.output().cursor_icon = CursorIcon::Text;
                        }
                    }
                }
                response.dragged = false;
                response.drag_released = true;
                response.is_pointer_button_down_on = false;
                response.interact_pointer_pos = None;
                response
            });

    }
}

fn terminal_context_menu(ui: &mut egui::Ui) {
    ui.input_mut().events.clear();

    if ui.button("Copy").clicked() {
        ui.input_mut().events.push(egui::Event::Copy);
        ui.close_menu();
    }

    if ui.button("Paste").clicked() {
       /* let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        if let Ok(text) = ctx.get_contents() {
            ui.input_mut().events.push(egui::Event::Paste(text));
        }
        ui.close_menu();*/
    }
}
