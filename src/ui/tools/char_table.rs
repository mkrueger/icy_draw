use std::sync::{Arc, Mutex};

use eframe::egui::{self, RichText};
use i18n_embed_fl::fl;

use crate::{AnsiEditor, Document, Message, ToolWindow};

#[derive(Default)]
pub struct CharTableToolWindow {}

impl ToolWindow for CharTableToolWindow {
    fn get_title(&self) -> String {
        fl!(crate::LANGUAGE_LOADER, "char_table_tool_title")
    }

    fn show_ui(
        &mut self,
        ui: &mut egui::Ui,
        active_document: Option<Arc<Mutex<Box<dyn Document>>>>,
    ) -> Option<Message> {
        if let Some(doc) = active_document {
            if let Some(editor) = doc.lock().unwrap().get_ansi_editor() {
                return show_char_table(ui, editor);
            }
        }
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            ui.label(RichText::new(fl!(crate::LANGUAGE_LOADER, "no_document_selected")).small());
        });
        None
    }
}

pub fn show_char_table(ui: &mut egui::Ui, editor: &AnsiEditor) -> Option<Message> {
    let mut result = None;

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
                            editor, ch, font_page,
                        ))
                        .clicked()
                    {
                        result = Some(Message::CharTable(ch));

                        /*
                        let editor = &mut b.editor;

                        b.redraw_view();*/
                    }
                }
            })
        });
    result
}
