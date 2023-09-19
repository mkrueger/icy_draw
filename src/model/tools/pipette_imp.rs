use eframe::{
    egui::{self, TextureOptions},
    emath::Align2,
    epaint::{Color32, FontId, Rounding, Vec2},
};
use egui_extras::RetainedImage;
use i18n_embed_fl::fl;
use icy_engine::{AttributedChar, Buffer, TextAttribute};

use crate::{create_retained_image, AnsiEditor, Message};

use super::{Position, Tool};

pub static mut CUR_CHAR: Option<AttributedChar> = None;

#[derive(Default)]
pub struct PipetteTool {
    ch: Option<char>,
    char_image: Option<RetainedImage>,
}

impl Tool for PipetteTool {
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage {
        &super::icons::DROPPER_SVG
    }

    fn use_caret(&self) -> bool {
        false
    }

    fn use_selection(&self) -> bool {
        false
    }

    fn show_ui(
        &mut self,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        editor_opt: Option<&AnsiEditor>,
    ) -> Option<Message> {
        let Some(editor) = editor_opt else {
            return None;
        };

        if let Some(ch) = unsafe { CUR_CHAR } {
            ui.vertical_centered(|ui| {
                ui.label(fl!(
                    crate::LANGUAGE_LOADER,
                    "pipette_tool_char_code",
                    code = (ch.ch as u32)
                ));

                if self.ch.is_none() || !self.ch.unwrap().eq(&ch.ch) {
                    self.ch = Some(ch.ch);

                    let mut buf = Buffer::new((1, 1));
                    buf.clear_font_table();
                    buf.set_font(
                        0,
                        editor
                            .buffer_view
                            .lock()
                            .get_buffer()
                            .get_font(ch.get_font_page())
                            .unwrap()
                            .clone(),
                    );
                    buf.layers[0]
                        .set_char((0, 0), AttributedChar::new(ch.ch, TextAttribute::default()));
                    self.char_image =
                        Some(create_retained_image(&buf).with_options(TextureOptions::NEAREST));
                }

                if let Some(image) = &self.char_image {
                    image.show_scaled(ui, 2.0);
                }

                ui.label(fl!(
                    crate::LANGUAGE_LOADER,
                    "pipette_tool_foreground",
                    fg = ch.attribute.get_foreground()
                ));
                paint_color(
                    ui,
                    editor
                        .buffer_view
                        .lock()
                        .get_buffer()
                        .palette
                        .get_color(ch.attribute.get_foreground() as usize),
                );
                ui.label(fl!(
                    crate::LANGUAGE_LOADER,
                    "pipette_tool_background",
                    bg = ch.attribute.get_background()
                ));
                paint_color(
                    ui,
                    editor
                        .buffer_view
                        .lock()
                        .get_buffer()
                        .palette
                        .get_color(ch.attribute.get_background() as usize),
                );
            });
        }
        None
    }

    fn handle_hover(
        &mut self,
        _ui: &egui::Ui,
        response: egui::Response,
        editor: &mut AnsiEditor,
        cur: Position,
        _cur_abs: Position,
    ) -> egui::Response {
        unsafe {
            CUR_CHAR = Some(editor.get_char(cur));
        }
        response.on_hover_cursor(egui::CursorIcon::Crosshair)
    }

    fn handle_no_hover(&mut self, _editor: &mut AnsiEditor) {
        unsafe {
            CUR_CHAR = None;
        }
    }

    fn handle_click(
        &mut self,
        editor: &mut AnsiEditor,
        button: i32,
        pos: Position,
        _pos_abs: Position,
        _response: &egui::Response,
    ) -> Option<Message> {
        if button == 1 {
            let ch = editor.get_char(pos);
            editor.set_caret_attribute(ch.attribute);
            return Some(Message::SelectPreviousTool);
        }
        None
    }
}

fn paint_color(ui: &mut egui::Ui, color: icy_engine::Color) {
    let (_, stroke_rect) = ui.allocate_space(Vec2::new(100.0, 32.0));

    let painter = ui.painter_at(stroke_rect);

    let (r, g, b) = color.get_rgb();
    painter.rect_filled(stroke_rect, Rounding::none(), Color32::BLACK);
    painter.rect_filled(stroke_rect.shrink(1.0), Rounding::none(), Color32::WHITE);
    let color = Color32::from_rgb(r, g, b);
    painter.rect_filled(stroke_rect.shrink(2.0), Rounding::none(), color);

    let text_color = if (r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) > 186.0 {
        Color32::BLACK
    } else {
        Color32::WHITE
    };

    let text = format!("#{r:02x}{g:02x}{b:02x}");
    let font_id: eframe::epaint::FontId = FontId::monospace(16.0);
    painter.text(
        stroke_rect.center(),
        Align2::CENTER_CENTER,
        text,
        font_id,
        text_color,
    );
}
