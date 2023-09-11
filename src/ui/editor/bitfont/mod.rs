mod undo;

use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use eframe::{
    egui::{self, RichText, Sense},
    emath::Align2,
    epaint::{Color32, FontFamily, FontId, Pos2, Rect, Rounding, Vec2},
};
use i18n_embed_fl::fl;
use icy_engine::{
    editor::UndoState,
    util::{pop_data, push_data, BITFONT_GLYPH},
    BitFont, EngineResult, Glyph,
};

use crate::{
    model::Tool, to_message, AnsiEditor, ClipboardHandler, Document, DocumentOptions, Message,
    TerminalResult,
};

use self::undo::UndoOperation;

pub struct BitFontEditor {
    font: BitFont,
    selected_char_opt: Option<char>,
    undo_stack: Arc<Mutex<Vec<Box<dyn UndoOperation>>>>,
    redo_stack: Vec<Box<dyn UndoOperation>>,
    old_data: Option<Vec<u8>>,
}

pub enum DrawGlyphStyle {
    Normal,
    Selected,
    GrayOut,
}

impl BitFontEditor {
    pub fn new(font: BitFont) -> Self {
        Self {
            font,
            selected_char_opt: None,
            undo_stack: Arc::new(Mutex::new(Vec::new())),
            redo_stack: Vec::new(),
            old_data: None,
        }
    }

    pub fn draw_glyph(
        ui: &mut egui::Ui,
        font: &BitFont,
        style: DrawGlyphStyle,
        ch: char,
    ) -> egui::Response {
        let scale = 3.;
        let (id, stroke_rect) = ui.allocate_space(Vec2::new(
            scale * font.size.width as f32,
            scale * font.size.height as f32,
        ));
        let response = ui.interact(stroke_rect, id, Sense::click());
        let col = if response.hovered() {
            match style {
                DrawGlyphStyle::Normal => Color32::LIGHT_GRAY,
                DrawGlyphStyle::Selected => Color32::WHITE,
                DrawGlyphStyle::GrayOut => Color32::GRAY,
            }
        } else {
            match style {
                DrawGlyphStyle::Normal => Color32::GRAY,
                DrawGlyphStyle::Selected => Color32::YELLOW,
                DrawGlyphStyle::GrayOut => Color32::DARK_GRAY,
            }
        };

        let painter = ui.painter_at(stroke_rect);
        painter.rect_filled(stroke_rect, Rounding::none(), Color32::BLACK);
        let s = font.size;
        if let Some(glyph) = font.get_glyph(ch) {
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
        response
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
            if response.drag_started_by(egui::PointerButton::Primary)
                || response.drag_started_by(egui::PointerButton::Secondary)
            {
                self.start_edit();
            }

            if response.drag_released_by(egui::PointerButton::Primary)
                || response.drag_released_by(egui::PointerButton::Secondary)
            {
                self.end_edit();
            }

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
                            if x == ((pos.x - (2. + left_ruler + stroke_rect.left()))
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
                            if y == ((pos.y - (2. + top_ruler + stroke_rect.top()))
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

    fn push_undo(&mut self, mut op: Box<dyn UndoOperation>) -> EngineResult<()> {
        op.redo(self)?;
        self.undo_stack.lock().unwrap().push(op);
        self.redo_stack.clear();
        Ok(())
    }

    fn clear_selected_glyph(&mut self) -> EngineResult<()> {
        if let Some(number) = self.selected_char_opt {
            let op = undo::ClearGlyph::new(number);
            self.push_undo(Box::new(op))?;
        }
        Ok(())
    }

    fn inverse_selected_glyph(&mut self) -> EngineResult<()> {
        if let Some(number) = self.selected_char_opt {
            let op = undo::InverseGlyph::new(number);
            self.push_undo(Box::new(op))?;
        }
        Ok(())
    }

    fn left_selected_glyph(&mut self) -> EngineResult<()> {
        if let Some(number) = self.selected_char_opt {
            let op = undo::LeftGlyph::new(number);
            self.push_undo(Box::new(op))?;
        }
        Ok(())
    }

    fn right_selected_glyph(&mut self) -> EngineResult<()> {
        if let Some(number) = self.selected_char_opt {
            let op = undo::RightGlyph::new(number);
            self.push_undo(Box::new(op))?;
        }
        Ok(())
    }

    fn up_selected_glyph(&mut self) -> EngineResult<()> {
        if let Some(number) = self.selected_char_opt {
            let op = undo::UpGlyph::new(number);
            self.push_undo(Box::new(op))?;
        }
        Ok(())
    }

    fn down_selected_glyph(&mut self) -> EngineResult<()> {
        if let Some(number) = self.selected_char_opt {
            let op = undo::DownGlyph::new(number);
            self.push_undo(Box::new(op))?;
        }
        Ok(())
    }

    fn flip_x_selected_glyph(&mut self) -> EngineResult<()> {
        if let Some(number) = self.selected_char_opt {
            let op = undo::FlipX::new(number);
            self.push_undo(Box::new(op))?;
        }
        Ok(())
    }

    fn flip_y_selected_glyph(&mut self) -> EngineResult<()> {
        if let Some(number) = self.selected_char_opt {
            let op = undo::FlipY::new(number);
            self.push_undo(Box::new(op))?;
        }
        Ok(())
    }

    fn start_edit(&mut self) {
        if let Some(number) = self.selected_char_opt {
            if let Some(glyph) = self.font.get_glyph_mut(number) {
                self.old_data = Some(glyph.data.clone());
            }
        }
    }

    fn end_edit(&mut self) {
        if self.old_data.is_none() {
            return;
        }
        if let Some(number) = self.selected_char_opt {
            if let Some(glyph) = self.font.get_glyph(number) {
                let op = undo::Edit::new(number, glyph.data.clone(), self.old_data.take().unwrap());
                self.undo_stack.lock().unwrap().push(Box::new(op));
                self.redo_stack.clear();
            }
        }
    }
}

impl ClipboardHandler for BitFontEditor {
    fn can_copy(&self) -> bool {
        self.selected_char_opt.is_some()
    }

    fn copy(&mut self) -> EngineResult<()> {
        if let Some(ch) = self.selected_char_opt {
            if let Some(data) = self.font.get_clipboard_data(ch) {
                push_data(BITFONT_GLYPH, &data)?;
            }
        }
        Ok(())
    }

    fn can_paste(&self) -> bool {
        if self.selected_char_opt.is_none() {
            return false;
        }

        pop_data(BITFONT_GLYPH).is_some()
    }

    fn paste(&mut self) -> EngineResult<()> {
        if let Some(data) = pop_data(BITFONT_GLYPH) {
            let (_, g) = Glyph::from_clipbard_data(&data);
            if let Some(ch) = self.selected_char_opt {
                let op = undo::Paste::new(ch, g);
                self.push_undo(Box::new(op))?;
            }
        }
        Ok(())
    }
}

impl UndoState for BitFontEditor {
    fn undo_description(&self) -> Option<String> {
        self.undo_stack
            .lock()
            .unwrap()
            .last()
            .map(|op| op.get_description())
    }

    fn can_undo(&self) -> bool {
        !self.undo_stack.lock().unwrap().is_empty()
    }

    fn undo(&mut self) -> EngineResult<()> {
        let Some(mut op) = self.undo_stack.lock().unwrap().pop() else {
            return Ok(());
        };

        let res = op.undo(self);
        self.redo_stack.push(op);

        res
    }

    fn redo_description(&self) -> Option<String> {
        self.redo_stack.last().map(|op| op.get_description())
    }

    fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    fn redo(&mut self) -> EngineResult<()> {
        if let Some(mut op) = self.redo_stack.pop() {
            let res = op.redo(self);
            self.undo_stack.lock().unwrap().push(op);
            return res;
        }
        Ok(())
    }
}

impl Document for BitFontEditor {
    fn default_extenision(&self) -> &'static str {
        "psf"
    }

    fn undo_stack_len(&self) -> usize {
        self.undo_stack.lock().unwrap().len()
    }

    fn show_ui(
        &mut self,
        ui: &mut eframe::egui::Ui,
        _cur_tool: &mut Box<dyn Tool>,
        _selected_tool: usize,
        _options: &DocumentOptions,
    ) -> Option<Message> {
        let mut message = None;
        ui.add_space(16.);
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.add_space(120.);
                ui.add(self.edit_glyph());

                ui.vertical(|ui| {
                    ui.add_space(20.);
                    ui.horizontal(|ui| {
                        if ui.button("Clear").clicked() {
                            message = to_message(self.clear_selected_glyph());
                        }
                        if ui.button("Inverse").clicked() {
                            message = to_message(self.inverse_selected_glyph());
                        }
                    });
                    ui.add_space(8.);
                    ui.horizontal(|ui| {
                        ui.add_space(14.);

                        if ui.button("⬆").clicked() {
                            message = to_message(self.up_selected_glyph());
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("⬅").clicked() {
                            message = to_message(self.left_selected_glyph());
                        }

                        if ui.button("➡").clicked() {
                            message = to_message(self.right_selected_glyph());
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.add_space(14.);

                        if ui.button("⬇").clicked() {
                            message = to_message(self.down_selected_glyph());
                        }
                    });
                    ui.add_space(8.);

                    ui.horizontal(|ui| {
                        if ui.button("Flip X").clicked() {
                            message = to_message(self.flip_x_selected_glyph());
                        }

                        if ui.button("Flip Y").clicked() {
                            message = to_message(self.flip_y_selected_glyph());
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
                    let ch = unsafe { char::from_u32_unchecked(i as u32) };
                    let mut style = DrawGlyphStyle::Normal;
                    if let Some(ch2) = self.selected_char_opt {
                        if ch == ch2 {
                            style = DrawGlyphStyle::Selected
                        }
                    }
                    let response = BitFontEditor::draw_glyph(ui, &self.font, style, ch);
                    if response.clicked() {
                        self.selected_char_opt = Some(ch);
                    }

                    response.on_hover_ui(|ui| {
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
        message
    }

    fn get_bytes(&mut self, _path: &Path) -> TerminalResult<Vec<u8>> {
        self.font.to_psf2_bytes()
    }

    fn get_ansi_editor_mut(&mut self) -> Option<&mut AnsiEditor> {
        None
    }

    fn get_ansi_editor(&self) -> Option<&AnsiEditor> {
        None
    }

    fn destroy(&self, _gl: &glow::Context) {}
}
