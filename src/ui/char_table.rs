use crate::ansi_editor::BufferView;
use crate::model::brush_imp::draw_glyph;
use eframe::egui::{self};
use icy_engine::{AsciiParser, BufferParser};
use std::sync::{Arc, Mutex};

pub fn show_char_table(buffer_opt: Option<Arc<Mutex<BufferView>>>) -> impl egui::Widget {
    move |ui: &mut egui::Ui| {
        let Some(buffer) = &buffer_opt else { 
            return ui.label("no selected editor");
        };
        let font_page = buffer.lock().unwrap().editor.cur_font_page;
        let font_length = buffer.lock().unwrap().editor.buf.font_table[font_page].length;

        egui::ScrollArea::vertical()
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for i in 0..font_length {
                        let ch = unsafe { char::from_u32_unchecked(i as u32) };
                        if ui.add(draw_glyph(buffer.clone(), ch, font_page)).clicked() {
                            if let Ok(b) = &mut buffer.lock() {
                                let mut p = AsciiParser::new();
                                let editor = &mut b.editor;
                                let res = BufferParser::print_char(
                                    &mut p,
                                    &mut editor.buf,
                                    &mut editor.caret,
                                    ch,
                                );
                                if let Err(err) = res {
                                    eprintln!("{}", err);
                                }
                                b.redraw_view();
                            }
                        }
                    }
                })
            })
            .inner
            .response
    }
}
