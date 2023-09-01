use std::fs;

use eframe::{
    egui::{self, RichText, Sense},
    emath::Align2,
    epaint::{Color32, FontFamily, FontId, Pos2, Rect, Rounding, Vec2},
};
use i18n_embed_fl::fl;
use icy_engine::BitFont;

use crate::{model::Tool, AnsiEditor, Document, DocumentOptions, TerminalResult};

pub struct BitFontEditor {
    font: BitFont,
    selected_char_opt: Option<char>,
    is_dirty: bool,
    id: usize,
}

impl BitFontEditor {
    pub fn new(font: BitFont, id: usize) -> Self {
        Self {
            font,
            selected_char_opt: None,
            is_dirty: false,
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
                        if glyph.data[y] & (128 >> x) != 0 {
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

            let left_ruler = 20.0;
            let top_ruler = 20.0;

            let (id, stroke_rect) = ui.allocate_space(Vec2::new(
                1. + (border + scale) * self.font.size.width as f32 + left_ruler,
                1. + (border + scale) * self.font.size.height as f32 + top_ruler,
            ));
            let mut response = ui.interact(stroke_rect, id, Sense::click_and_drag());

            let painter = ui.painter_at(stroke_rect);
            painter.rect_filled(stroke_rect, Rounding::none(), Color32::DARK_GRAY);

            let s = self.font.size;

            /*   if response.clicked() {
                if let Some(pos) = response.hover_pos() {
                    if let Some(number) = self.selected_char_opt {
                        if let Some(glyph) = self.font.get_glyph_mut(number) {
                            println!("click!");
                            let y = ((pos.y - stroke_rect.top()) / (scale + border)) as usize;
                            let x = ((pos.x - stroke_rect.left()) / (scale + border)) as usize;
                            if glyph.data[y] & (128 >> x) != 0 {
                                println!("unset!");
                                glyph.data[y] &= !(128 >> x);
                            } else {
                                println!("set!");
                                glyph.data[y] |= 128 >> x;
                            }
                            self.is_dirty = true;
                            response.mark_changed();
                        }
                    }
                }
            } else { */

            if response.dragged_by(egui::PointerButton::Primary) {
                if let Some(pos) = response.interact_pointer_pos() {
                    if let Some(number) = self.selected_char_opt {
                        if let Some(glyph) = self.font.get_glyph_mut(number) {
                            let y = ((pos.y - left_ruler - stroke_rect.top()) / (scale + border))
                                as usize;
                            let x = ((pos.x - top_ruler - stroke_rect.left()) / (scale + border))
                                as usize;
                            if y < glyph.data.len() && x < 8 {
                                glyph.data[y] |= 128 >> x;
                                self.is_dirty = true;
                                response.mark_changed();
                            }
                        }
                    }
                }
            }

            if response.dragged_by(egui::PointerButton::Secondary) {
                if let Some(pos) = response.interact_pointer_pos() {
                    if let Some(number) = self.selected_char_opt {
                        if let Some(glyph) = self.font.get_glyph_mut(number) {
                            let y = ((pos.y - left_ruler - stroke_rect.top()) / (scale + border))
                                as usize;
                            let x = ((pos.x - top_ruler - stroke_rect.left()) / (scale + border))
                                as usize;
                            if y < glyph.data.len() && x < 8 {
                                glyph.data[y] &= !(128 >> x);
                                self.is_dirty = true;
                                response.mark_changed();
                            }
                        }
                    }
                }
            }
            if let Some(number) = self.selected_char_opt {
                if let Some(glyph) = self.font.get_glyph_mut(number) {
                    painter.rect_filled(
                        Rect::from_min_size(
                            Pos2::new(stroke_rect.left(), stroke_rect.top()),
                            Vec2::new(
                                2. + left_ruler + s.width as f32 * (border + scale),
                                2. + top_ruler + s.height as f32 * (border + scale),
                            ),
                        ),
                        Rounding::none(),
                        ui.style().visuals.extreme_bg_color,
                    );

                    for x in 0..s.width {
                        let pos = Pos2::new(
                            2. + left_ruler
                                + stroke_rect.left()
                                + (x as f32 + 0.5) * (border + scale),
                            2. + top_ruler / 2. + stroke_rect.top(),
                        );
                        let col = if let Some(pos) = response.hover_pos() {
                            if x as i32
                                == ((pos.x - (2. + left_ruler + stroke_rect.left()))
                                    / (border + scale)) as i32
                            {
                                ui.style().visuals.strong_text_color()
                            } else {
                                ui.style().visuals.text_color()
                            }
                        } else {
                            ui.style().visuals.text_color()
                        };

                        painter.text(
                            pos,
                            Align2::CENTER_CENTER,
                            (x + 1).to_string(),
                            FontId::new(12.0, FontFamily::Proportional),
                            col,
                        );
                    }
                    for y in 0..s.height {
                        let pos = Pos2::new(
                            2. + left_ruler / 2. + stroke_rect.left(),
                            2. + top_ruler
                                + stroke_rect.top()
                                + (y as f32 + 0.5) * (border + scale),
                        );
                        let col = if let Some(pos) = response.hover_pos() {
                            if y as i32
                                == ((pos.y - (2. + top_ruler + stroke_rect.top()))
                                    / (border + scale)) as i32
                            {
                                ui.style().visuals.strong_text_color()
                            } else {
                                ui.style().visuals.text_color()
                            }
                        } else {
                            ui.style().visuals.text_color()
                        };

                        painter.text(
                            pos,
                            Align2::CENTER_CENTER,
                            (y + 1).to_string(),
                            FontId::new(12.0, FontFamily::Proportional),
                            col,
                        );
                    }

                    for y in 0..s.height {
                        for x in 0..s.width {
                            let rect = Rect::from_min_size(
                                Pos2::new(
                                    2. + left_ruler
                                        + stroke_rect.left()
                                        + x as f32 * (border + scale),
                                    2. + top_ruler
                                        + stroke_rect.top()
                                        + y as f32 * (border + scale),
                                ),
                                Vec2::new(scale, scale),
                            );
                            let col = if glyph.data[y] & (128 >> x) != 0 {
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

    fn clear_selected_glyph(&mut self) {
        if let Some(number) = self.selected_char_opt {
            if let Some(glyph) = self.font.get_glyph_mut(number) {
                glyph.data.fill(0);
            }
        }
    }

    fn inverse_selected_glyph(&mut self) {
        if let Some(number) = self.selected_char_opt {
            if let Some(glyph) = self.font.get_glyph_mut(number) {
                for i in 0..glyph.data.len() {
                    glyph.data[i] ^= 0xFF;
                }
            }
        }
    }

    fn left_selected_glyph(&mut self) {
        if let Some(number) = self.selected_char_opt {
            if let Some(glyph) = self.font.get_glyph_mut(number) {
                for i in 0..glyph.data.len() {
                    glyph.data[i] <<= 1;
                }
            }
        }
    }

    fn right_selected_glyph(&mut self) {
        if let Some(number) = self.selected_char_opt {
            if let Some(glyph) = self.font.get_glyph_mut(number) {
                for i in 0..glyph.data.len() {
                    glyph.data[i] >>= 1;
                }
            }
        }
    }

    fn up_selected_glyph(&mut self) {
        if let Some(number) = self.selected_char_opt {
            if let Some(glyph) = self.font.get_glyph_mut(number) {
                glyph.data.remove(0);
                glyph.data.push(0);
            }
        }
    }

    fn down_selected_glyph(&mut self) {
        if let Some(number) = self.selected_char_opt {
            if let Some(glyph) = self.font.get_glyph_mut(number) {
                glyph.data.insert(0, 0);
                glyph.data.pop();
            }
        }
    }

    fn flip_x_selected_glyph(&mut self) {
        if let Some(number) = self.selected_char_opt {
            let w = 8 - self.font.size.width;
            if let Some(glyph) = self.font.get_glyph_mut(number) {
                for i in 0..glyph.data.len() {
                    glyph.data[i] = glyph.data[i].reverse_bits() << w;
                }
            }
        }
    }

    fn flip_y_selected_glyph(&mut self) {
        if let Some(number) = self.selected_char_opt {
            if let Some(glyph) = self.font.get_glyph_mut(number) {
                glyph.data = glyph.data.iter().rev().copied().collect();
            }
        }
    }
}

impl Document for BitFontEditor {
    fn get_title(&self) -> String {
        self.font.name.to_string()
    }
    fn get_id(&self) -> usize {
        self.id
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    fn show_ui(
        &mut self,
        ui: &mut eframe::egui::Ui,
        _cur_tool: &mut Box<dyn Tool>,
        _options: &DocumentOptions,
    ) {
        ui.add_space(16.);

        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.add_space(120.);
                ui.add(self.edit_glyph());

                ui.vertical(|ui| {
                    ui.add_space(20.);
                    ui.horizontal(|ui| {
                        if ui.button("Clear").clicked() {
                            self.clear_selected_glyph();
                        }
                        if ui.button("Inverse").clicked() {
                            self.inverse_selected_glyph();
                        }
                    });
                    ui.add_space(8.);
                    ui.horizontal(|ui| {
                        ui.add_space(14.);

                        if ui.button("⬆").clicked() {
                            self.up_selected_glyph();
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("⬅").clicked() {
                            self.left_selected_glyph();
                        }

                        if ui.button("➡").clicked() {
                            self.right_selected_glyph();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.add_space(14.);

                        if ui.button("⬇").clicked() {
                            self.down_selected_glyph();
                        }
                    });
                    ui.add_space(8.);

                    ui.horizontal(|ui| {
                        if ui.button("Flip X").clicked() {
                            self.flip_x_selected_glyph();
                        }

                        if ui.button("Flip Y").clicked() {
                            self.flip_y_selected_glyph();
                        }
                    });
                });
            });
        });

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

    fn get_ansi_editor_mut(&mut self) -> Option<&mut AnsiEditor> {
        None
    }

    fn get_ansi_editor(&self) -> Option<&AnsiEditor> {
        None
    }

    fn destroy(&self, _gl: &glow::Context) {}
}
