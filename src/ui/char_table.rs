/* TODO

use icy_engine::{ascii, BufferParser};
use eframe::egui::{self};

use crate::AnsiEditor;


pub fn show_char_table(editor: Option<&AnsiEditor>) -> impl egui::Widget {
    let Some(editor) = &editor else {
        return move |ui: &mut egui::Ui| { ui.label("no selected editor") }.inner.response;
    };

    let buffer_view = editor.buffer_view.clone();

    move |ui: &mut egui::Ui| {
        let font_page = editor.buffer_view.lock().caret.get_font_page();
        let font_length = editor
            .buffer_view
            .lock()
            .buf
            .get_font(font_page)
            .unwrap()
            .length;

        egui::ScrollArea::vertical()
            .id_source("char_table_scroll_area")
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for i in 0..font_length {
                        let ch = unsafe { char::from_u32_unchecked(i as u32) };
                        if ui
                            .add(crate::model::pencil_imp::draw_glyph_plain(
                                editor.clone(),
                                ch,
                                font_page,
                            ))
                            .clicked()
                        {
                            let mut p = ascii::Parser::default();
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
                })
            })
            .inner
            .response
    }

}
    */
