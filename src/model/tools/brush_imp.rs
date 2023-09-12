use eframe::{
    egui::{self, Response, RichText, Sense},
    epaint::{Color32, Pos2, Rect, Rounding, Vec2},
};
use egui_extras::RetainedImage;
use i18n_embed_fl::fl;
use icy_engine::{editor::AtomicUndoGuard, AttributedChar, Layer, TextPane};
use icy_engine_egui::TerminalCalc;
use std::{cell::RefCell, rc::Rc};

use crate::{create_retained_image, AnsiEditor, Event, Message};

use super::{Position, Tool};

#[derive(PartialEq, Eq)]
pub enum BrushType {
    Shade,
    Solid,
    Color,
    Custom,
}

pub static mut CUSTOM_BRUSH: Option<Layer> = None;

pub struct BrushTool {
    pub use_fore: bool,
    pub use_back: bool,
    pub size: i32,
    pub char_code: Rc<RefCell<char>>,

    pub undo_op: Option<AtomicUndoGuard>,

    pub custom_brush: Option<Layer>,
    pub image: Option<RetainedImage>,
    pub brush_type: BrushType,
}

impl BrushTool {
    fn paint_brush(&self, editor: &mut AnsiEditor, pos: Position) {
        let mid = Position::new(-(self.size / 2), -(self.size / 2));

        let center = pos + mid;
        let gradient = ['\u{00B0}', '\u{00B1}', '\u{00B2}', '\u{00DB}'];
        let caret_attr = editor.buffer_view.lock().get_caret().get_attribute();
        if matches!(self.brush_type, BrushType::Custom) {
            editor.join_overlay("brush");
            return;
        }

        for y in 0..self.size {
            for x in 0..self.size {
                match self.brush_type {
                    BrushType::Shade => {
                        let ch = editor.get_char_from_cur_layer(center + Position::new(x, y));
                        let mut char_code = gradient[0];
                        if ch.ch == gradient[gradient.len() - 1] {
                            char_code = gradient[gradient.len() - 1];
                        } else {
                            for i in 0..gradient.len() - 1 {
                                if ch.ch == gradient[i] {
                                    char_code = gradient[i + 1];
                                    break;
                                }
                            }
                        }
                        editor.set_char(
                            center + Position::new(x, y),
                            AttributedChar::new(char_code, caret_attr),
                        );
                    }
                    BrushType::Solid => {
                        editor.set_char(
                            center + Position::new(x, y),
                            AttributedChar::new(*self.char_code.borrow(), caret_attr),
                        );
                    }
                    BrushType::Color => {
                        let ch = editor.get_char_from_cur_layer(center + Position::new(x, y));
                        let mut attribute = ch.attribute;
                        if self.use_fore {
                            attribute.set_foreground(caret_attr.get_foreground());
                        }
                        if self.use_back {
                            attribute.set_background(caret_attr.get_background());
                        }
                        editor.set_char(
                            center + Position::new(x, y),
                            AttributedChar::new(ch.ch, attribute),
                        );
                    }
                    BrushType::Custom => {}
                }
            }
        }
    }
}

impl Tool for BrushTool {
    fn get_icon_name(&self) -> &'static RetainedImage {
        &super::icons::BRUSH_SVG
    }

    fn use_caret(&self) -> bool {
        false
    }

    fn show_ui(
        &mut self,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        buffer_opt: &AnsiEditor,
    ) -> Option<Message> {
        let mut result = None;
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.use_fore, fl!(crate::LANGUAGE_LOADER, "tool-fg"))
                    .clicked()
                {
                    self.use_fore = !self.use_fore;
                }
                if ui
                    .selectable_label(self.use_back, fl!(crate::LANGUAGE_LOADER, "tool-bg"))
                    .clicked()
                {
                    self.use_back = !self.use_back;
                }
            });
        });
        ui.horizontal(|ui| {
            ui.label(fl!(crate::LANGUAGE_LOADER, "tool-size-label"));
            ui.add(
                egui::DragValue::new(&mut self.size)
                    .clamp_range(1..=20)
                    .speed(1),
            );
        });
        ui.radio_value(
            &mut self.brush_type,
            BrushType::Shade,
            fl!(crate::LANGUAGE_LOADER, "tool-shade"),
        );
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.brush_type,
                BrushType::Solid,
                fl!(crate::LANGUAGE_LOADER, "tool-character"),
            );

            result = draw_glyph(ui, buffer_opt, &self.char_code);
        });
        ui.radio_value(
            &mut self.brush_type,
            BrushType::Color,
            fl!(crate::LANGUAGE_LOADER, "tool-colorize"),
        );

        unsafe {
            if CUSTOM_BRUSH.is_some() {
                self.custom_brush = CUSTOM_BRUSH.take();

                let mut layer = self.custom_brush.as_ref().unwrap().clone();
                layer.set_offset((0, 0));
                layer.role = icy_engine::Role::Normal;
                let mut buf = icy_engine::Buffer::new(layer.get_size());
                layer.title = buf.layers[0].title.clone();
                buf.layers.clear();
                buf.layers.push(layer);
                self.image = Some(create_retained_image(&buf));
            }
        }

        if self.custom_brush.is_some() {
            ui.radio_value(
                &mut self.brush_type,
                BrushType::Custom,
                fl!(crate::LANGUAGE_LOADER, "tool-custom-brush"),
            );
            if let Some(image) = &self.image {
                let w = ui.available_width() - 16.0;
                let scale = w / image.width() as f32;
                ui.image(
                    image.texture_id(ui.ctx()),
                    Vec2::new(image.width() as f32 * scale, image.height() as f32 * scale),
                );
            }
        }
        result
    }

    fn handle_hover(
        &mut self,
        _ui: &egui::Ui,
        response: egui::Response,
        editor: &mut AnsiEditor,
        cur: Position,
        _cur_abs: Position,
    ) -> egui::Response {
        if matches!(self.brush_type, BrushType::Custom) {
            editor.clear_overlay_layer();
            if let Some(layer) = editor
                .buffer_view
                .lock()
                .get_buffer_mut()
                .get_overlay_layer()
            {
                if let Some(brush) = &self.custom_brush {
                    let mid = Position::new(-(brush.get_width() / 2), -(brush.get_height() / 2));
                    for y in 0..brush.get_height() {
                        for x in 0..brush.get_width() {
                            let pos = Position::new(x, y);
                            let ch = brush.get_char(pos);
                            layer.set_char(
                                cur + pos + mid,
                                AttributedChar::new(ch.ch, ch.attribute),
                            );
                        }
                    }
                }
            }
        } else {
            editor.buffer_view.lock().get_buffer_mut().remove_overlay();
        }

        response
    }

    fn handle_click(
        &mut self,
        editor: &mut AnsiEditor,
        button: i32,
        pos: Position,
        _pos_abs: Position,
        _response: &Response,
    ) -> super::Event {
        if button == 1 {
            let _op: AtomicUndoGuard =
                editor.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-paint-brush"));

            self.paint_brush(editor, pos);
        }
        super::Event::None
    }

    fn handle_drag(
        &mut self,
        _ui: &egui::Ui,
        response: egui::Response,
        editor: &mut AnsiEditor,
        _calc: &TerminalCalc,
    ) -> egui::Response {
        self.paint_brush(editor, editor.drag_pos.cur);
        response
    }

    fn handle_drag_begin(&mut self, editor: &mut AnsiEditor, _response: &egui::Response) -> Event {
        self.undo_op =
            Some(editor.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-paint-brush")));
        Event::None
    }

    fn handle_drag_end(&mut self, _editor: &mut AnsiEditor) -> Event {
        self.undo_op = None;
        Event::None
    }
}

pub fn draw_glyph(
    ui: &mut egui::Ui,
    editor: &AnsiEditor,
    ch: &Rc<RefCell<char>>,
) -> Option<Message> {
    let font_page = editor.buffer_view.lock().get_caret().get_font_page();
    if let Some(font) = editor.buffer_view.lock().get_buffer().get_font(font_page) {
        let scale = 1.5;
        let (id, stroke_rect) = ui.allocate_space(Vec2::new(
            scale * font.size.width as f32,
            scale * font.size.height as f32,
        ));
        let response = ui.interact(stroke_rect, id, Sense::click());

        let col = if response.hovered() {
            Color32::WHITE
        } else {
            Color32::GRAY
        };

        if response.clicked() {
            return Some(crate::Message::ShowCharacterSelectionDialog(ch.clone()));
        }

        let painter = ui.painter_at(stroke_rect);
        painter.rect_filled(stroke_rect, Rounding::none(), Color32::BLACK);
        let s = font.size;
        let ch = *ch.borrow();
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
            response.on_hover_ui(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(fl!(crate::LANGUAGE_LOADER, "glyph-char-label")).small(),
                    );
                    ui.label(
                        RichText::new(format!("{0}/0x{0:02X}", ch as u32))
                            .small()
                            .color(Color32::WHITE),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(fl!(crate::LANGUAGE_LOADER, "glyph-font-label")).small(),
                    );
                    ui.label(
                        RichText::new(font.name.to_string())
                            .small()
                            .color(Color32::WHITE),
                    );
                });
            });
        }
    }
    None
}
