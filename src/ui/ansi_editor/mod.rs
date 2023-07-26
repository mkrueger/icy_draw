use eframe::{
    egui::{self, CursorIcon, PointerButton, ScrollArea},
    epaint::{Pos2, Rect, Vec2},
};
use i18n_embed_fl::fl;
use icy_engine::{AnsiParser, Buffer, BufferParser, Position, SaveOptions};
use std::{
    borrow::BorrowMut,
    cmp::{max, min},
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub mod render;
pub use render::*;

pub mod sixel;
pub use sixel::*;

pub mod buffer_view;
pub use buffer_view::*;

pub mod key_maps;
pub use key_maps::*;

use crate::{
    model::{brush_imp::draw_glyph, MKey, MModifiers, Tool},
    Document, TerminalResult,
};

pub struct AnsiEditor {
    is_dirty: bool,
    enabled: bool,
    pressed_button: i32,
    drag_start: Position,
    drag_pos: Position,

    buffer_view: Arc<Mutex<BufferView>>,
    buffer_parser: Box<dyn BufferParser>,
}

impl AnsiEditor {
    pub fn new(gl: &Arc<glow::Context>, buf: Buffer) -> Self {
        let buffer_view = Arc::new(Mutex::new(BufferView::new(gl, buf)));
        let buffer_parser = AnsiParser::new();

        Self {
            is_dirty: false,
            enabled: true,
            pressed_button: -1,
            drag_start: Position::default(),
            drag_pos: Position::default(),
            buffer_view,
            buffer_parser: Box::new(buffer_parser),
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
        self.buffer_view
            .lock()
            .unwrap()
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

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn save(&mut self, file_name: &str) -> TerminalResult<()> {
        let file = PathBuf::from(file_name);
        let options = SaveOptions::new();
        let bytes = self
            .buffer_view
            .lock()
            .unwrap()
            .editor
            .buf
            .to_bytes(file.extension().unwrap().to_str().unwrap(), &options)?;
        fs::write(file_name, bytes)?;
        self.is_dirty = false;
        Ok(())
    }

    fn show_ui(&mut self, ui: &mut egui_dock::egui::Ui, cur_tool: &mut Box<dyn Tool>) {
        ui.vertical(|ui| {
            let size = ui.max_rect().size();
            let size = Vec2::new(size.x, size.y - 28.0);

            ScrollArea::both()
                .auto_shrink([false; 2])
                .max_height(size.y)
                .drag_to_scroll(false)
                .show_viewport(ui, |ui, viewport| {
                    let (id, draw_area) = ui.allocate_space(size);
                    let mut response = ui.interact(draw_area, id, egui::Sense::click());
                    let font_dimensions = self
                        .buffer_view
                        .lock()
                        .unwrap()
                        .editor
                        .buf
                        .get_font_dimensions();
                    let scale = self.buffer_view.lock().unwrap().scale;
                    let real_height = self
                        .buffer_view
                        .lock()
                        .unwrap()
                        .editor
                        .buf
                        .get_real_buffer_height();

                    self.buffer_view
                        .lock()
                        .unwrap()
                        .editor
                        .buf
                        .terminal_state
                        .height = min(
                        real_height,
                        (draw_area.height() / (font_dimensions.height as f32 * scale)).ceil()
                            as i32,
                    );

                    let buf_w = self
                        .buffer_view
                        .lock()
                        .unwrap()
                        .editor
                        .buf
                        .get_buffer_width();
                    let buf_h = self
                        .buffer_view
                        .lock()
                        .unwrap()
                        .editor
                        .buf
                        .get_buffer_height();

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
                            if rect_w < draw_area.width() {
                                (draw_area.width() - rect_w) / 2.
                            } else {
                                0.
                            },
                            if rect_h < draw_area.height() {
                                (draw_area.height() - rect_h) / 2.
                            } else {
                                0.
                            },
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

                    let buffer_view = self.buffer_view.clone();
                    let callback = egui::PaintCallback {
                        rect: draw_area,
                        callback: std::sync::Arc::new(egui_glow::CallbackFn::new(
                            move |info, painter| {
                                buffer_view.lock().unwrap().update_buffer(painter.gl());
                                buffer_view.lock().unwrap().paint(
                                    painter.gl(),
                                    info,
                                    draw_area,
                                    relative_rect,
                                );
                            },
                        )),
                    };

                    let rect = Rect::from_min_size(
                        draw_area.left_top()
                            + Vec2::new(
                                if rect_w < draw_area.width() {
                                    (draw_area.width() - rect_w) / 2.
                                } else {
                                    0.
                                },
                                if rect_h < draw_area.height() {
                                    (draw_area.height() - rect_h) / 2.
                                } else {
                                    0.
                                } - draw_area.left_top().y,
                            )
                            .ceil(),
                        Vec2::new(rect_w, rect_h),
                    );

                    ui.painter().add(callback);
                    response = response.context_menu(|ui| {
                        terminal_context_menu(&mut self.buffer_view.clone(), ui)
                    });
                    if self.enabled {
                        let events = ui.input(|i| i.events.clone());
                        for e in &events {
                            match e {
                                egui::Event::Copy => {
                                    let buffer_view = self.buffer_view.clone();
                                    let mut l = buffer_view.lock().unwrap();
                                    if let Some(txt) = l.get_copy_text(&self.buffer_parser) {
                                        ui.output_mut(|o| o.copied_text = txt);
                                    }
                                }
                                egui::Event::Cut => {}
                                egui::Event::Paste(text) => {
                                    self.output_string(&text);
                                    self.buffer_view.lock().unwrap().redraw_view();
                                }

                                egui::Event::CompositionEnd(text) | egui::Event::Text(text) => {
                                    for c in text.chars() {
                                        let buffer_view = self.buffer_view.clone();
                                        cur_tool.handle_key(
                                            buffer_view,
                                            MKey::Character(c as u16),
                                            MModifiers::None,
                                        );
                                    }
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
                                        let click_pos = calc_click_pos(
                                            &pos,
                                            rect,
                                            top_margin_height,
                                            char_size,
                                            first_line,
                                        );
                                        let b = match button {
                                            PointerButton::Primary => 1,
                                            PointerButton::Secondary => 2,
                                            PointerButton::Middle => 3,
                                            PointerButton::Extra1 => 4,
                                            PointerButton::Extra2 => 5,
                                        };
                                        self.pressed_button = b;
                                        self.drag_start =
                                            Position::new(click_pos.x as i32, click_pos.y as i32);
                                        self.drag_pos = self.drag_start;
                                        cur_tool.handle_click(buffer_view, b, self.drag_start);
                                    }
                                }

                                egui::Event::PointerButton {
                                    pos,
                                    pressed: false,
                                    ..
                                } => {
                                    self.pressed_button = -1;
                                    let buffer_view = self.buffer_view.clone();
                                    let click_pos = calc_click_pos(
                                        pos,
                                        rect,
                                        top_margin_height,
                                        char_size,
                                        first_line,
                                    );
                                    cur_tool.handle_drag_end(
                                        buffer_view,
                                        self.drag_start,
                                        Position::new(click_pos.x as i32, click_pos.y as i32),
                                    );
                                }

                                egui::Event::PointerMoved(pos) => {
                                    if self.pressed_button >= 0 {
                                        let buffer_view = self.buffer_view.clone();
                                        let click_pos = calc_click_pos(
                                            &pos,
                                            rect,
                                            top_margin_height,
                                            char_size,
                                            first_line,
                                        );
                                        let cur =
                                            Position::new(click_pos.x as i32, click_pos.y as i32);
                                        if self.drag_pos != cur {
                                            self.drag_pos = cur;
                                            buffer_view.lock().unwrap().redraw_view();
                                            cur_tool.handle_drag(buffer_view, self.drag_start, cur);
                                        }
                                    }
                                }

                                /*egui::Event::KeyRepeat { key, modifiers }
                                | */
                                egui::Event::Key {
                                    key,
                                    pressed: true,
                                    modifiers,
                                    ..
                                } => {
                                    let mut key_code = *key as u32;
                                    if modifiers.ctrl || modifiers.command {
                                        key_code |= CTRL_MOD;
                                    }
                                    if modifiers.shift {
                                        key_code |= SHIFT_MOD;
                                    }

                                    let mut modifier: MModifiers = MModifiers::None;
                                    if modifiers.ctrl || modifiers.command {
                                        modifier = MModifiers::Control;
                                    }

                                    if modifiers.shift {
                                        modifier = MModifiers::Shift;
                                    }
                                    for (k, m) in ANSI_KEY_MAP {
                                        if *k == key_code {
                                            let buffer_view = self.buffer_view.clone();
                                            cur_tool.handle_key(buffer_view, *m, modifier);
                                            self.buffer_view.lock().unwrap().redraw_view();
                                            response.mark_changed();
                                            ui.input_mut(|i| i.consume_key(*modifiers, *key));
                                            break;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    if response.hovered() {
                        let hover_pos_opt = ui.input(|i| i.pointer.hover_pos());
                        if let Some(hover_pos) = hover_pos_opt {
                            if rect.contains(hover_pos) {
                                ui.output_mut(|o| o.cursor_icon = CursorIcon::Text);
                            }
                        }
                    }

                    response.dragged = false;
                    response.drag_released = true;
                    response.is_pointer_button_down_on = false;
                    response.interact_pointer_pos = None;
                    response
                });

            ui.horizontal(|ui| {
                let pos = self.buffer_view.lock().unwrap().editor.caret.get_position();
                let width = self
                    .buffer_view
                    .lock()
                    .unwrap()
                    .editor
                    .buf
                    .get_buffer_width();
                let height = self
                    .buffer_view
                    .lock()
                    .unwrap()
                    .editor
                    .buf
                    .get_buffer_height();

                ui.label(fl!(
                    crate::LANGUAGE_LOADER,
                    "toolbar-position",
                    line = pos.y,
                    column = pos.x
                ));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    ui.horizontal(|ui| {
                        for i in (0..10).into_iter().rev() {
                            let ch = self
                                .buffer_view
                                .lock()
                                .unwrap()
                                .editor
                                .get_outline_char_code(i as i32)
                                .unwrap();
                            let cur_page = self.buffer_view.lock().unwrap().editor.cur_font_page;
                            ui.add(draw_glyph(
                                self.buffer_view.clone(),
                                unsafe { char::from_u32_unchecked(ch as u32) },
                                cur_page,
                            ));
                            ui.label(format!("F{}", i + 1));
                        }

                        ui.label(fl!(
                            crate::LANGUAGE_LOADER,
                            "toolbar-size",
                            colums = width,
                            rows = height
                        ));
                    });
                });
            });
        });
    }

    fn get_buffer_view(&self) -> Option<Arc<Mutex<buffer_view::BufferView>>> {
        Some(self.buffer_view.clone())
    }

    fn destroy(&self, gl: &glow::Context) {
        self.buffer_view.lock().unwrap().destroy(gl);
    }
}

fn calc_click_pos(
    pos: &Pos2,
    rect: Rect,
    top_margin_height: f32,
    char_size: Vec2,
    _first_line: i32,
) -> Vec2 {
    (*pos - rect.min - Vec2::new(0., top_margin_height)) / char_size
}

pub fn terminal_context_menu(buffer_view: &mut Arc<Mutex<BufferView>>, ui: &mut egui::Ui) {
    ui.input_mut(|i| i.events.clear());

    if ui
        .button(fl!(crate::LANGUAGE_LOADER, "menu-copy"))
        .clicked()
    {
        ui.input_mut(|i| i.events.push(egui::Event::Copy));
        ui.close_menu();
    }

    if ui
        .button(fl!(crate::LANGUAGE_LOADER, "menu-paste"))
        .clicked()
    {
        /* let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        if let Ok(text) = ctx.get_contents() {
            ui.input_mut().events.push(egui::Event::Paste(text));
        }
        ui.close_menu();*/
    }
    let mut view = buffer_view.borrow_mut().lock().unwrap();

    if let Some(_sel) = &view.editor.cur_selection {
        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-erase"))
            .clicked()
        {
            view.editor.delete_selection();
            ui.close_menu();
        }
        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-flipx"))
            .clicked()
        {
            view.editor.flip_x();
            ui.close_menu();
        }
        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-flipy"))
            .clicked()
        {
            view.editor.flip_y();
            ui.close_menu();
        }

        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-justifyleft"))
            .clicked()
        {
            view.editor.justify_left();
            ui.close_menu();
        }
        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-justifyright"))
            .clicked()
        {
            view.editor.justify_right();
            ui.close_menu();
        }
        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-justifycenter"))
            .clicked()
        {
            view.editor.justify_center();
            ui.close_menu();
        }
    }
}
