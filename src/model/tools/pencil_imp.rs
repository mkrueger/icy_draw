use eframe::{
    egui::{self, RichText, Sense},
    epaint::{Color32, Pos2, Rect, Rounding, Vec2},
};
use egui_extras::RetainedImage;
use i18n_embed_fl::fl;
use icy_engine::{AttributedChar, Rectangle};

use crate::{model::ScanLines, AnsiEditor};

use super::{brush_imp::draw_glyph, line_imp::set_half_block, Position, Tool, ToolUiResult};

#[derive(PartialEq, Eq)]
pub enum PencilType {
    HalfBlock,
    Shade,
    Solid,
    Color,
}

pub struct PencilTool {
    pub use_fore: bool,
    pub use_back: bool,
    pub char_code: std::rc::Rc<std::cell::RefCell<char>>,
    pub font_page: usize,

    pub last_pos: Position,
    pub brush_type: PencilType,
}

impl PencilTool {
    fn paint_brush(&self, editor: &mut AnsiEditor, pos: Position) {
        let center = pos;
        let gradient = ['\u{00B0}', '\u{00B1}', '\u{00B2}', '\u{00DB}'];
        match self.brush_type {
            PencilType::HalfBlock => {
                let mut lines = ScanLines::new(1);
                lines.add_line(
                    Position::new(self.last_pos.x, self.last_pos.y * 2),
                    Position::new(pos.x, pos.y * 2),
                );
                let draw = move |rect: Rectangle| {
                    let col = editor
                        .buffer_view
                        .lock()
                        .caret
                        .get_attribute()
                        .get_foreground();
                    for y in 0..rect.size.height {
                        for x in 0..rect.size.width {
                            set_half_block(
                                editor,
                                Position::new(rect.start.x + x, rect.start.y + y),
                                col,
                            );
                        }
                    }
                };
                lines.fill(draw);
            }
            PencilType::Shade => {
                let ch = editor.get_char_from_cur_layer(center);
                let attribute = editor.buffer_view.lock().caret.get_attribute();

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
                editor.set_char(center, AttributedChar::new(char_code, attribute));
            }
            PencilType::Solid => {
                let attribute = editor.buffer_view.lock().caret.get_attribute();
                editor.set_char(
                    center,
                    AttributedChar::new(*self.char_code.borrow(), attribute),
                );
            }
            PencilType::Color => {
                let ch = editor.get_char_from_cur_layer(center);
                let mut attribute = ch.attribute;
                if self.use_fore {
                    attribute.set_foreground(
                        editor
                            .buffer_view
                            .lock()
                            .caret
                            .get_attribute()
                            .get_foreground(),
                    );
                }
                if self.use_back {
                    attribute.set_background(
                        editor
                            .buffer_view
                            .lock()
                            .caret
                            .get_attribute()
                            .get_background(),
                    );
                }
                editor.set_char(center, AttributedChar::new(ch.ch, attribute));
            }
        }
    }
}

impl Tool for PencilTool {
    fn get_icon_name(&self) -> &'static RetainedImage {
        &super::icons::PENCIL_SVG
    }

    fn use_caret(&self) -> bool {
        false
    }

    fn show_ui(
        &mut self,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        buffer_opt: &mut AnsiEditor,
    ) -> ToolUiResult {
        let mut result = ToolUiResult::default();
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
        ui.radio_value(
            &mut self.brush_type,
            PencilType::HalfBlock,
            fl!(crate::LANGUAGE_LOADER, "tool-half-block"),
        );
        ui.radio_value(
            &mut self.brush_type,
            PencilType::Shade,
            fl!(crate::LANGUAGE_LOADER, "tool-shade"),
        );
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.brush_type,
                PencilType::Solid,
                fl!(crate::LANGUAGE_LOADER, "tool-character"),
            );

            draw_glyph(ui, buffer_opt, &mut result, &self.char_code, self.font_page);
        });
        ui.radio_value(
            &mut self.brush_type,
            PencilType::Color,
            fl!(crate::LANGUAGE_LOADER, "tool-colorize"),
        );
        result
    }

    fn handle_click(
        &mut self,
        buffer_view: &mut AnsiEditor,
        button: i32,
        pos: Position,
    ) -> super::Event {
        if button == 1 {
            self.last_pos = pos;
            self.paint_brush(buffer_view, pos);
        }
        super::Event::None
    }

    fn handle_drag(
        &mut self,
        buffer_view: &mut AnsiEditor,
        _start: Position,
        cur: Position,
    ) -> super::Event {
        self.paint_brush(buffer_view, cur);
        self.last_pos = cur;

        super::Event::None
    }
}

pub fn draw_glyph_plain(editor: &AnsiEditor, ch: char, font_page: usize) -> impl egui::Widget {
    let bv = editor.buffer_view.clone();
    move |ui: &mut egui::Ui| {
        if let Some(font) = bv.lock().buf.get_font(font_page) {
            let scale = 1.8;
            let padding = 2.;
            let (id, stroke_rect) = ui.allocate_space(Vec2::new(
                2. * padding + scale * font.size.width as f32,
                2. * padding + scale * font.size.height as f32,
            ));
            let mut response = ui.interact(stroke_rect, id, Sense::click());

            let col = if response.hovered() {
                Color32::WHITE
            } else {
                Color32::GRAY
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
                                        padding + stroke_rect.left() + x as f32 * scale,
                                        padding + stroke_rect.top() + y as f32 * scale,
                                    ),
                                    Vec2::new(scale.ceil(), scale.ceil()),
                                ),
                                Rounding::none(),
                                col,
                            );
                        }
                    }
                }
                response = response.on_hover_ui(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Char").small());
                        ui.label(
                            RichText::new(format!("{0}/0x{0:02X}", ch as u32))
                                .small()
                                .color(Color32::WHITE),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Font").small());
                        ui.label(
                            RichText::new(font.name.to_string())
                                .small()
                                .color(Color32::WHITE),
                        );
                    });
                });
            }
            response
        } else {
            let (id, stroke_rect) = ui.allocate_space(Vec2::new(1., 1.));
            ui.interact(stroke_rect, id, Sense::click())
        }
    }
}
