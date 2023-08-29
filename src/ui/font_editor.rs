use std::fs;

use eframe::{
    egui::{self, RichText, Sense},
    epaint::{Color32, Pos2, Rect, Rounding, Vec2},
};
use i18n_embed_fl::fl;
use icy_engine::BitFont;

use crate::{model::Tool, AnsiEditor, Document, DocumentOptions, TerminalResult};

pub struct FontEditor {
    font: BitFont,
    selected_char_opt: Option<char>,
    is_dirty: bool,
    enabled: bool,
    id: usize,
}

impl FontEditor {
    pub fn new(font: BitFont, id: usize) -> Self {
        Self {
            font,
            selected_char_opt: None,
            is_dirty: false,
            enabled: true,
            id,
        }
    }

    pub fn draw_glyph(&mut self, ch: char) -> impl egui::Widget + '_ {
        move |ui: &mut egui::Ui| {
            let scale = 3.;
            let (id, stroke_rect) = ui.allocate_space(Vec2::new(
                scale * self.font.size.width as f32,
                scale * self.font.size.height as f32,
            ));
            let mut response = ui.interact(stroke_rect, id, Sense::click());
            let is_selected = if let Some(ch2) = self.selected_char_opt {
                ch == ch2
            } else {
                false
            };
            let col = if response.hovered() {
                if is_selected {
                    Color32::LIGHT_YELLOW
                } else {
                    Color32::WHITE
                }
            } else if is_selected {
                Color32::YELLOW
            } else {
                Color32::GRAY
            };

            let painter = ui.painter_at(stroke_rect);
            painter.rect_filled(stroke_rect, Rounding::none(), Color32::BLACK);
            let s = self.font.size;
            if let Some(glyph) = self.font.get_glyph(ch) {
                for y in 0..s.height {
                    for x in 0..s.width {
                        if glyph.data[y as usize] & (128 >> x) != 0 {
                            painter.rect_filled(
                                Rect::from_min_size(
                                    Pos2::new(
                                        stroke_rect.left() + x as f32 * scale,
                                        stroke_rect.top() + y as f32 * scale,
                                    ),
                                    Vec2::new(scale, scale),
                                ),
                                Rounding::none(),
                                col,
                            );
                        }
                    }
                }
            }
            if response.clicked() {
                self.selected_char_opt = Some(ch);
                response.mark_changed();
            }
            response
        }
    }

    pub fn edit_glyph(&mut self) -> impl egui::Widget + '_ {
        move |ui: &mut egui::Ui| {
            let scale = 20.;
            let border = 2.;

            let (id, stroke_rect) = ui.allocate_space(Vec2::new(
                1. + (border + scale) * self.font.size.width as f32,
                1. + (border + scale) * self.font.size.height as f32,
            ));
            let mut response = ui.interact(stroke_rect, id, Sense::click());

            let painter = ui.painter_at(stroke_rect);
            painter.rect_filled(stroke_rect, Rounding::none(), Color32::DARK_GRAY);

            let s = self.font.size;

            if response.clicked() {
                if let Some(pos) = response.hover_pos() {
                    if let Some(number) = self.selected_char_opt {
                        if let Some(glyph) = self.font.get_glyph_mut(number) {
                            let y = ((pos.y - stroke_rect.top()) / (scale + border)) as usize;
                            let x = ((pos.x - stroke_rect.left()) / (scale + border)) as usize;
                            if glyph.data[y] & (128 >> x) != 0 {
                                glyph.data[y] &= !(128 >> x);
                            } else {
                                glyph.data[y] |= 128 >> x;
                            }
                            self.is_dirty = true;
                            response.mark_changed();
                        }
                    }
                }
            }

            if let Some(number) = self.selected_char_opt {
                if let Some(glyph) = self.font.get_glyph_mut(number) {
                    for y in 0..s.height {
                        for x in 0..s.width {
                            let rect = Rect::from_min_size(
                                Pos2::new(
                                    2. + stroke_rect.left() + x as f32 * (border + scale),
                                    2. + stroke_rect.top() + y as f32 * (border + scale),
                                ),
                                Vec2::new(scale, scale),
                            );
                            let col = if glyph.data[y as usize] & (128 >> x) != 0 {
                                if let Some(pos) = response.hover_pos() {
                                    if rect.contains(pos) {
                                        Color32::WHITE
                                    } else {
                                        Color32::GRAY
                                    }
                                } else {
                                    Color32::GRAY
                                }
                            } else if let Some(pos) = response.hover_pos() {
                                if rect.contains(pos) {
                                    Color32::DARK_GRAY
                                } else {
                                    Color32::BLACK
                                }
                            } else {
                                Color32::BLACK
                            };
                            painter.rect_filled(rect, Rounding::none(), col);
                        }
                    }
                }
            }
            response
        }
    }
}

impl Document for FontEditor {
    fn get_title(&self) -> String {
        self.font.name.to_string()
    }
    fn get_id(&self) -> usize {
        self.id
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn show_ui(
        &mut self,
        ui: &mut eframe::egui::Ui,
        _cur_tool: &mut Box<dyn Tool>,
        options: &DocumentOptions,
    ) {
        ui.vertical_centered(|ui| ui.add(self.edit_glyph()));

        ui.label(fl!(
            crate::LANGUAGE_LOADER,
            "font-editor-table",
            length = (self.font.length - 1).to_string()
        ));
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for i in 0..self.font.length {
                    ui.add(self.draw_glyph(unsafe { char::from_u32_unchecked(i as u32) }))
                        .on_hover_ui(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Char").small());
                                ui.label(
                                    RichText::new(format!("{0}/0x{0:02X}", i))
                                        .small()
                                        .color(Color32::WHITE),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("ASCII").small());
                                ui.label(
                                    RichText::new(format!("'{0}'", unsafe {
                                        char::from_u32_unchecked(i as u32)
                                    }))
                                    .small()
                                    .color(Color32::WHITE),
                                );
                            });
                        });
                }
            })
        });
    }

    fn save(&mut self, file_name: &str) -> TerminalResult<()> {
        fs::write(file_name, self.font.to_psf2_bytes()?)?;
        self.is_dirty = false;
        Ok(())
    }

    fn get_buffer_view(&mut self) -> Option<&mut AnsiEditor> {
        None
    }

    fn destroy(&self, _gl: &glow::Context) {}
}
