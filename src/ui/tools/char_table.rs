use std::sync::{Arc, Mutex};

use eframe::{
    egui::{self, RichText, Sense, TextureOptions},
    epaint::{Color32, Pos2, Rect, Vec2},
};
use egui_extras::RetainedImage;
use i18n_embed_fl::fl;
use icy_engine::{AttributedChar, BitFont, Buffer, TextAttribute};

use crate::{create_retained_image, AnsiEditor, Document, Message, ToolWindow};

pub struct CharTableToolWindow {
    font: BitFont,
    hover_char: Option<char>,
    hover_char_image: RetainedImage,
    char_table: RetainedImage,
}

const BUFFER_WIDTH: usize = 16;

impl CharTableToolWindow {
    pub fn new() -> Self {
        let font = BitFont::default();
        let char_table = create_font_image(&font);
        let hover_char_image = create_hover_image(&font, ' ', 14);
        Self {
            font,
            char_table,
            hover_char: None,
            hover_char_image,
        }
    }
    pub fn show_char_table(&mut self, ui: &mut egui::Ui, editor: &AnsiEditor) -> Option<Message> {
        let mut result = None;

        let font_page = editor.buffer_view.lock().get_caret().get_font_page();
        if let Some(cur_font) = editor.buffer_view.lock().get_buffer().get_font(font_page) {
            if cur_font.name != self.font.name {
                self.font = cur_font.clone();
                self.char_table = create_font_image(&self.font);
                self.hover_char = None;
            }
        }

        let mut hover_char = None;

        egui::ScrollArea::vertical()
            .id_source("char_table_scroll_area")
            .show(ui, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let scale = 2.0;

                    let width = self.char_table.width() as f32 * scale;

                    let height = self.char_table.height() as f32 * scale;
                    ui.add_space((ui.available_width() - width) / 2.0);

                    let (id, rect) = ui.allocate_space([width, height].into());
                    let response = ui.interact(rect, id, Sense::click());
                    ui.painter().image(
                        self.char_table.texture_id(ui.ctx()),
                        Rect::from_min_size(
                            Pos2::new(rect.left(), rect.top()),
                            Vec2::new(width, height),
                        ),
                        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                    let fw = scale * self.font.size.width as f32;
                    let fh = scale * self.font.size.height as f32;

                   
                    if response.hovered() {
                        if let Some(pos) = response.hover_pos() {
                            let pos = pos - response.rect.min;
                            let ch = (pos.x / fw) as usize + BUFFER_WIDTH * (pos.y / fh) as usize;
                            let ch = unsafe { char::from_u32_unchecked(ch as u32) };
                            hover_char = Some(ch);
                            if self.hover_char != hover_char {
                                self.hover_char = hover_char;
                                self.hover_char_image = create_hover_image(&self.font, ch, 14);
                            }

                            let x = (ch as usize) % BUFFER_WIDTH;
                            let y = (ch as usize) / BUFFER_WIDTH;

                            let rect = Rect::from_min_size(
                                rect.min + Vec2::new(x as f32 * fw, y as f32 * fh),
                                Vec2::new(fw, fh),
                            );

                            ui.painter().image(
                                self.hover_char_image.texture_id(ui.ctx()),
                                rect.expand(2.0),
                                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        }
                    }/* TODO: Font should swap on pipette tool as well.
                     else {
                        unsafe {
                            if let Some(pipete_char) = crate::model::pipette_imp::CUR_CHAR {
                                let ch = pipete_char.ch;
                                hover_char = Some(ch);
                                if self.hover_char != hover_char {
                                    self.hover_char = hover_char;
                                    self.hover_char_image = create_hover_image(&self.font, ch, 12);
                                }
    
                                let x = (ch as usize) % BUFFER_WIDTH;
                                let y = (ch as usize) / BUFFER_WIDTH;
    
                                let rect = Rect::from_min_size(
                                    rect.min + Vec2::new(x as f32 * fw, y as f32 * fh),
                                    Vec2::new(fw, fh),
                                );
    
                                ui.painter().image(
                                    self.hover_char_image.texture_id(ui.ctx()),
                                    rect.expand(2.0),
                                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                                    Color32::WHITE,
                                );
                            }
                        }
                    }*/

                    if response.clicked() {
                        if let Some(ch) = hover_char {
                            result = Some(Message::CharTable(ch));
                        }
                    }
                });
            });
        if let Some(ch) = hover_char {
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(fl!(crate::LANGUAGE_LOADER, "font-view-char_label")).small(),
                );
                ui.label(
                    RichText::new(format!("{0}/0x{0:02X}", ch as u32))
                        .small()
                        .color(Color32::WHITE),
                );
            });
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(fl!(crate::LANGUAGE_LOADER, "font-view-font_label")).small(),
                );
                ui.label(
                    RichText::new(self.font.name.to_string())
                        .small()
                        .color(Color32::WHITE),
                );
            });
        }
        result
    }
}

impl Default for CharTableToolWindow {
    fn default() -> Self {
        Self::new()
    }
}

fn create_font_image(font: &BitFont) -> RetainedImage {
    let mut buffer = Buffer::new((BUFFER_WIDTH, 256 / BUFFER_WIDTH));
    buffer.set_font(0, font.clone());
    for ch in 0..256 {
        buffer.layers[0].set_char(
            (ch % BUFFER_WIDTH, ch / BUFFER_WIDTH),
            AttributedChar::new(
                unsafe { char::from_u32_unchecked(ch as u32) },
                TextAttribute::default(),
            ),
        );
    }
    create_retained_image(&buffer).with_options(TextureOptions::NEAREST)
}

fn create_hover_image(font: &BitFont, ch: char, color: u32) -> RetainedImage {
    let mut buffer = Buffer::new((1, 1));
    buffer.set_font(0, font.clone());
    let mut attr = TextAttribute::default();
    attr.set_foreground(color);

    buffer.layers[0].set_char(
        (0, 0),
        AttributedChar::new(unsafe { char::from_u32_unchecked(ch as u32) }, attr),
    );
    create_retained_image(&buffer).with_options(TextureOptions::NEAREST)
}

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
                return self.show_char_table(ui, editor);
            }
        }
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            ui.label(RichText::new(fl!(crate::LANGUAGE_LOADER, "no_document_selected")).small());
        });
        None
    }
}
