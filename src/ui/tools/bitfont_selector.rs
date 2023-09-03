use std::sync::{Arc, Mutex};

use eframe::egui::{self, RichText};
use i18n_embed_fl::fl;

use crate::{AnsiEditor, Document, Message, ToolWindow};

#[derive(Default)]
pub struct BitFontSelector {}

impl ToolWindow for BitFontSelector {
    fn get_title(&self) -> String {
        fl!(crate::LANGUAGE_LOADER, "bitfont_tool_title")
    }

    fn show_ui(
        &mut self,
        ui: &mut egui::Ui,
        active_document: Option<Arc<Mutex<Box<dyn Document>>>>,
    ) -> Option<Message> {
        if let Some(doc) = active_document {
            if let Some(editor) = doc.lock().unwrap().get_ansi_editor() {
                return show_font_list(ui, editor);
            }
        }
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            ui.label(RichText::new(fl!(crate::LANGUAGE_LOADER, "no_document_selected")).small());
        });
        None
    }
}

fn show_font_list(ui: &mut egui::Ui, editor: &AnsiEditor) -> Option<Message> {
    let mut result = None;

    /*
    for (id, font) in editor.buffer_view.lock().get_buffer().font_iter() {

        if id >= 100 {
            // TODO
        }
    }*/
    let row_height = 23.0;

    let cur_font_page = editor.buffer_view.lock().get_caret().get_font_page();

    egui::ScrollArea::vertical()
        .id_source("bitfont_scroll_area")
        .max_height(300.)
        .show_rows(
            ui,
            row_height,
            icy_engine::parsers::ansi::constants::ANSI_FONT_NAMES.len(),
            |ui, range| {
                for r in range {
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(
                                cur_font_page == r,
                                format!(
                                    "{r}. {}",
                                    icy_engine::parsers::ansi::constants::ANSI_FONT_NAMES[r]
                                ),
                            )
                            .clicked()
                        {
                            result = Some(Message::SetFontPage(r));
                        }
                    });
                }
            },
        );
    result
}
