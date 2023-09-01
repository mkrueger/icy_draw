use eframe::{
    egui::{self, RichText},
    epaint::{Color32, Vec2},
};
use egui_extras::{Column, TableBuilder};
use i18n_embed_fl::fl;
use icy_engine::BitFont;

use crate::{AnsiEditor, Document, Message};

#[derive(Default)]
pub struct BitFontSelector {}

impl BitFontSelector {
    pub fn show_ui(
        &self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        editor: &AnsiEditor,
    ) -> Option<Message> {
        let mut result = None;

        /*
        for (id, font) in editor.buffer_view.lock().buf.font_iter() {

            if id >= 100 {
                // TODO
            }
        }*/
        let row_height = 23.0;

        let cur_font_page = editor.buffer_view.lock().caret.get_font_page();

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
}
