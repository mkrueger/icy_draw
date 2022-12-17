use std::{cmp::{max, min}, fs, path::PathBuf, sync::{Arc, Mutex}};
use eframe::{epaint::{Vec2, Rect, Pos2}, egui::{CursorIcon, self, ScrollArea, PointerButton}};
use icy_engine::{Buffer, SaveOptions, AnsiParser, BufferParser, Position};

pub mod render;
pub use render::*;

pub mod sixel;
pub use sixel::*;

pub mod buffer_view;
pub use buffer_view::*;

pub mod key_maps;
pub use key_maps::*;

use crate::{Document, TerminalResult, model::Tool};

pub struct AnsiEditor {
    is_dirty: bool,

    pressed_button: i32,
    drag_start: Position,
    drag_pos: Position,

    buffer_view: Arc<Mutex<BufferView>>,
    buffer_parser: Box<dyn BufferParser>
}

impl AnsiEditor {
    pub fn new(gl: &Arc<glow::Context>, buf: Buffer) -> Self {
        let buffer_view = Arc::new(Mutex::new(BufferView::new(gl, buf)));
        let buffer_parser = AnsiParser::new();

        Self {
            is_dirty: false,
            pressed_button: -1,
            drag_start: Position::default(),
            drag_pos: Position::default(),
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
        self
            .buffer_view
            .lock().unwrap()
            .print_char(&mut self.buffer_parser, unsafe {
                char::from_u32_unchecked(c as u32)
            })?;
        self.buffer_view.lock().unwrap().redraw_view();
        Ok(())
    }


}

impl Document for AnsiEditor {
    fn get_title(&self) -> String {
        if let Some(file_name) = &self.buffer_view.lock().unwrap().editor.buf.file_name {
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
        let bytes = self.buffer_view.lock().unwrap().editor.buf.to_bytes(file.extension().unwrap().to_str().unwrap(), &options)?;
        fs::write(file_name, bytes)?;
        self.is_dirty = false;
        Ok(())
    }

    fn show_ui(&mut self, ui: &mut eframe::egui::Ui, cur_tool: &mut Box<dyn Tool>) {
        let size = ui.max_rect().size();


        let _output = ScrollArea::both()
            .auto_shrink([false; 2])
            .show_viewport(ui, |ui, viewport| {
                let (id, draw_area) = ui.allocate_space(size);
                let mut response = ui.interact(draw_area, id, egui::Sense::click());
                let font_dimensions = self.buffer_view.lock().unwrap().editor.buf.get_font_dimensions();
                let scale = self.buffer_view.lock().unwrap().scale;
                let real_height = self.buffer_view.lock().unwrap().editor.buf.get_real_buffer_height();

                self.buffer_view.lock().unwrap().editor.buf.terminal_state.height = min(real_height, (draw_area.height() / (font_dimensions.height as f32 * scale)).ceil() as i32);

                let buf_w = self.buffer_view.lock().unwrap().editor.buf.get_buffer_width();
                let buf_h = self.buffer_view.lock().unwrap().editor.buf.get_buffer_height();
        
                let char_size = Vec2::new(
                    font_dimensions.width as f32 * scale,
                    font_dimensions.height as f32 * scale,
                );
        
                let rect_w = buf_w as f32 * char_size.x;
                let rect_h = buf_h as f32 * char_size.y;
                let top_margin_height = ui.min_rect().top();

                let rect_h = min(rect_h as i32, draw_area.height() as i32) as f32;

                let relative_rect = Rect::from_min_size(
                    Pos2::new(
                        if rect_w < draw_area.width() { (draw_area.width() - rect_w) / 2. } else { 0. },
                        if rect_h < draw_area.height() { (draw_area.height() - rect_h) / 2. } else { 0. },
                        )
                        .ceil(),
                    Vec2::new(rect_w, rect_h),
                );

                let max_lines = max(0, real_height - buf_h);
                ui.set_height(scale * max_lines as f32 * font_dimensions.height as f32);
                ui.set_width(rect_w);
                let first_line = (viewport.top() / char_size.y) as i32;

                if first_line != self.buffer_view.lock().unwrap().scroll_first_line {
                    self.buffer_view.lock().unwrap().scroll_first_line = first_line;
                    self.buffer_view.lock().unwrap().redraw_view();
                }
                
                let buffer_view  = self.buffer_view.clone();
                let callback = egui::PaintCallback {
                    rect: draw_area,
                    callback: std::sync::Arc::new(egui_glow::CallbackFn::new(
                        move |info, painter| {
                            buffer_view.lock().unwrap().update_buffer(painter.gl());
                            buffer_view.lock().unwrap().paint(painter.gl(), info, draw_area, relative_rect);
                        },
                    )),
                };

                let rect = Rect::from_min_size(
                    draw_area.left_top() + 
                    Vec2::new(
                        if rect_w < draw_area.width() { (draw_area.width() - rect_w) / 2. } else { 0. },
                        if rect_h < draw_area.height() { (draw_area.height() - rect_h) / 2. } else { 0. } 
                        -draw_area.left_top().y,
                        )
                        .ceil(),
                    Vec2::new(rect_w, rect_h),
                );

                ui.painter().add(callback);
                response = response.context_menu(terminal_context_menu);

                let events = ui.input().events.clone();
                for e in &events {
                    match e {
                        egui::Event::Copy => {
                            let buffer_view = self.buffer_view.clone();
                            let mut l = buffer_view.lock().unwrap();
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
                            button,
                            pressed: true,
                            ..
                        } => {
                            if rect.contains(*pos) {
                                let buffer_view = self.buffer_view.clone();
                                let click_pos =
                                    (*pos - rect.min - Vec2::new(0., top_margin_height))
                                        / char_size
                                        + Vec2::new(0.0, first_line as f32);
                                let b = match button {
                                    PointerButton::Primary => 1,
                                    PointerButton::Secondary => 2,
                                    PointerButton::Middle => 3,
                                    PointerButton::Extra1 => 4,
                                    PointerButton::Extra2 => 5,
                                };
                                self.pressed_button = b;
                                self.drag_start = Position::new(click_pos.x as i32, click_pos.y as i32);
                                self.drag_pos = self.drag_start;
                                cur_tool.handle_click( buffer_view, b, self.drag_start);
                            }
                        }

                        egui::Event::PointerButton {
                            pos,
                            pressed: false,
                            ..
                        } => {
                            self.pressed_button = -1;
                            let buffer_view = self.buffer_view.clone();
                            let click_pos =
                                (*pos - rect.min - Vec2::new(0., top_margin_height))
                                    / char_size
                                    + Vec2::new(0.0, first_line as f32);
                            cur_tool.handle_drag_end(buffer_view, self.drag_start, Position::new(click_pos.x as i32, click_pos.y as i32));
                        }

                        egui::Event::PointerMoved(pos) => {
                            if self.pressed_button >= 0 {
                                let buffer_view = self.buffer_view.clone();
                                let click_pos =
                                    (*pos - rect.min - Vec2::new(0., top_margin_height))
                                        / char_size
                                        + Vec2::new(0.0, first_line as f32);
                                let cur  =Position::new(click_pos.x as i32, click_pos.y as i32);
                                if self.drag_pos != cur {
                                    self.drag_pos = cur;
                                    buffer_view.lock().unwrap().redraw_view();
                                    cur_tool.handle_drag(buffer_view, self.drag_start, cur);
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

    fn get_buffer_view(&self) -> Option<Arc<Mutex<buffer_view::BufferView>>> {
        Some(self.buffer_view.clone())
    }

    fn destroy(&self, gl: &glow::Context) {
        self.buffer_view.lock().unwrap().destroy(gl);
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
