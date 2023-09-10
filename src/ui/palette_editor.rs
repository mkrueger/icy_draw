use crate::{AnsiEditor, Message, Settings, SWAP_SVG};
use eframe::egui::{self, color_picker, Sense, TextEdit, TextStyle, Ui};
use eframe::emath::Align2;
use eframe::epaint::{Color32, FontId, Pos2, Rect, Rounding, Stroke, Vec2};
use i18n_embed_fl::fl;
use icy_engine::XTERM_256_PALETTE;
use std::cmp::min;

pub fn palette_switcher(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    editor: &AnsiEditor,
) -> Option<Message> {
    let mut result = None;

    let tex_id = SWAP_SVG.texture_id(ctx);
    let caret_attr = editor.buffer_view.lock().get_caret().get_attribute();
    let palette = editor.buffer_view.lock().get_buffer().palette.clone();

    let height = 62.0;
    let (id, rect) = ui.allocate_space(Vec2::new(height, height));
    let response = ui.interact(rect, id, Sense::click());
    let painter = ui.painter_at(rect);

    let rect_height = height * 0.618;

    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(height - rect_height, height - rect_height) + rect.left_top().to_vec2(),
            Vec2::new(rect_height, rect_height),
        ),
        Rounding::none(),
        Color32::BLACK,
    );

    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(height - rect_height + 1., height - rect_height + 1.)
                + rect.left_top().to_vec2(),
            Vec2::new(rect_height - 2., rect_height - 2.),
        ),
        Rounding::none(),
        Color32::WHITE,
    );

    let (r, g, b) = palette.colors[caret_attr.get_background() as usize].get_rgb();
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(height - rect_height + 2., height - rect_height + 2.)
                + rect.left_top().to_vec2(),
            Vec2::new(rect_height - 4., rect_height - 4.),
        ),
        Rounding::none(),
        Color32::from_rgb(r, g, b),
    );

    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(0., 0.) + rect.left_top().to_vec2(),
            Vec2::new(rect_height, rect_height),
        ),
        Rounding::none(),
        Color32::BLACK,
    );

    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(1., 1.) + rect.left_top().to_vec2(),
            Vec2::new(rect_height - 2., rect_height - 2.),
        ),
        Rounding::none(),
        Color32::WHITE,
    );

    let (r, g, b) = palette.colors[caret_attr.get_foreground() as usize].get_rgb();
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(2., 2.) + rect.left_top().to_vec2(),
            Vec2::new(rect_height - 4., rect_height - 4.),
        ),
        Rounding::none(),
        Color32::from_rgb(r, g, b),
    );

    let s_rect_height = height * 0.382;
    let rh = s_rect_height / 1.8;
    let (r, g, b) = palette.colors[7].get_rgb();

    let overlap = 2.0;

    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(rh - overlap, height - rh - overlap) + rect.left_top().to_vec2(),
            Vec2::new(rh, rh),
        ),
        Rounding::none(),
        Color32::from_rgb(r ^ 0xFF, g ^ 0xFF, b ^ 0xFF),
    );

    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(rh - overlap + 1., height - rh - overlap + 1.) + rect.left_top().to_vec2(),
            Vec2::new(rh - 2., rh - 2.),
        ),
        Rounding::none(),
        Color32::from_rgb(r, g, b),
    );

    let (r, g, b) = palette.colors[0].get_rgb();
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(overlap, height - 2. * rh + 2. + overlap) + rect.left_top().to_vec2(),
            Vec2::new(rh, rh),
        ),
        Rounding::none(),
        Color32::from_rgb(r ^ 0xFF, g ^ 0xFF, b ^ 0xFF),
    );

    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(1. + overlap, height - 2. * rh + 3. + overlap) + rect.left_top().to_vec2(),
            Vec2::new(rh - 2., rh - 2.),
        ),
        Rounding::none(),
        Color32::from_rgb(r, g, b),
    );

    painter.image(
        tex_id,
        Rect::from_min_size(
            Pos2::new(rect_height + 1., 0.) + rect.left_top().to_vec2(),
            Vec2::new(s_rect_height, s_rect_height),
        ),
        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );

    if let Some(hp) = response.hover_pos() {
        let pos = hp.to_vec2() - rect.left_top().to_vec2();

        if response.clicked() {
            if pos.x > rect_height && pos.y < rect_height {
                result = Some(Message::ToggleColor);
            }

            if pos.x < rect_height && pos.y > rect_height {
                result = Some(Message::SwitchToDefaultColor);
            }
        }
    }

    result
}

pub fn palette_editor_16(ui: &mut egui::Ui, editor: &AnsiEditor) {
    let caret_attr = editor.buffer_view.lock().get_caret().get_attribute();
    let palette = editor.buffer_view.lock().get_buffer().palette.clone();
    let buffer_view = editor.buffer_view.clone();

    ui.horizontal(|ui| {
        ui.add_space(4.0);

        let height = (ui.available_width()) / 8.0;
        let (id, stroke_rect) =
            ui.allocate_space(Vec2::new(ui.available_width() - 4.0, height * 2.0));
        let mut response = ui.interact(stroke_rect, id, Sense::click());

        let painter = ui.painter_at(stroke_rect);

        for i in 0..16 {
            let (r, g, b) = palette.colors[i].get_rgb();
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(
                        stroke_rect.left() + (i % 8) as f32 * height,
                        stroke_rect.top() + (i / 8) as f32 * height,
                    ),
                    Vec2::new(height, height),
                ),
                Rounding::none(),
                Color32::from_rgb(r, g, b),
            );
        }

        let marker_len = height / 3.;
        // paint fg marker
        let stroke = Stroke::new(1., Color32::WHITE);
        let origin = Pos2::new(
            stroke_rect.left() + (caret_attr.get_foreground() % 8) as f32 * height,
            stroke_rect.top() + (caret_attr.get_foreground() / 8) as f32 * height,
        );
        painter.line_segment([origin, origin + Vec2::new(marker_len, 0.)], stroke);
        painter.line_segment([origin, origin + Vec2::new(0., marker_len)], stroke);
        for i in 0..marker_len as usize {
            painter.line_segment(
                [
                    origin + Vec2::new(i as f32, 0.),
                    origin + Vec2::new(0., i as f32),
                ],
                stroke,
            );
        }
        let stroke = Stroke::new(1., Color32::GRAY);
        painter.line_segment([origin, origin + Vec2::new(marker_len, 0.)], stroke);
        painter.line_segment([origin, origin + Vec2::new(0., marker_len)], stroke);
        painter.line_segment(
            [
                origin + Vec2::new(marker_len, 0.),
                origin + Vec2::new(0., marker_len),
            ],
            stroke,
        );

        // paint bg marker
        let stroke = Stroke::new(1., Color32::WHITE);
        let origin = Pos2::new(
            stroke_rect.left() + (1 + caret_attr.get_background() % 8) as f32 * height,
            stroke_rect.top() + (1 + caret_attr.get_background() / 8) as f32 * height,
        );
        painter.line_segment([origin, origin - Vec2::new(marker_len, 0.)], stroke);
        painter.line_segment([origin, origin - Vec2::new(0., marker_len)], stroke);
        for i in 0..marker_len as usize {
            painter.line_segment(
                [
                    origin - Vec2::new(i as f32, 0.),
                    origin - Vec2::new(0., i as f32),
                ],
                stroke,
            );
        }
        let stroke = Stroke::new(1., Color32::GRAY);
        painter.line_segment([origin, origin - Vec2::new(marker_len, 0.)], stroke);
        painter.line_segment([origin, origin - Vec2::new(0., marker_len)], stroke);
        painter.line_segment(
            [
                origin - Vec2::new(marker_len, 0.),
                origin - Vec2::new(0., marker_len),
            ],
            stroke,
        );
        if let Some(hp) = response.hover_pos() {
            let pos = (hp.to_vec2() - stroke_rect.left_top().to_vec2()) / Vec2::new(height, height);
            let color = min(palette.len() - 1, pos.x as u32 + pos.y as u32 * 8);
            if response.clicked() {
                buffer_view.lock().get_caret_mut().set_foreground(color);
                response.mark_changed();
            }
            if response.secondary_clicked() {
                buffer_view.lock().get_caret_mut().set_background(color);
                response.mark_changed();
            }
        }
    });
}

pub fn show_extended_palette(ui: &mut Ui, editor: &AnsiEditor) {
    let row_height = 24.0;
    egui::ScrollArea::vertical()
        .id_source("bitfont_scroll_area")
        .max_height(200.)
        .show_rows(ui, row_height, XTERM_256_PALETTE.len(), |ui, range| {
            for idx in range {
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    let width = ui.available_width();

                    let (id, back_rect) = ui.allocate_space(Vec2::new(width, row_height));
                    let mut response = ui.interact(back_rect, id, Sense::click());

                    let back_painter = ui.painter_at(back_rect);

                    if response.hovered() {
                        back_painter.rect_filled(
                            back_rect,
                            Rounding::none(),
                            ui.style().visuals.widgets.active.bg_fill,
                        );
                    }

                    let stroke_rect = Rect::from_min_size(
                        back_rect.min + Vec2::new(0.0, 1.0),
                        Vec2::new(52.0, 22.0),
                    );

                    let painter = ui.painter_at(stroke_rect);

                    let (r, g, b) = XTERM_256_PALETTE[idx].1.get_rgb();
                    painter.rect_filled(stroke_rect, Rounding::none(), Color32::BLACK);
                    painter.rect_filled(stroke_rect.shrink(1.0), Rounding::none(), Color32::WHITE);
                    let color = Color32::from_rgb(r, g, b);
                    painter.rect_filled(stroke_rect.shrink(2.0), Rounding::none(), color);

                    let text_color =
                        if (r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) > 186.0 {
                            Color32::BLACK
                        } else {
                            Color32::WHITE
                        };

                    let text = format!("#{:02x}{:02x}{:02x}", r, g, b);
                    let font_id: eframe::epaint::FontId = FontId::monospace(10.0);
                    painter.text(
                        stroke_rect.left_center() + Vec2::new(4., 0.),
                        Align2::LEFT_CENTER,
                        text,
                        font_id,
                        text_color,
                    );

                    let font_id = TextStyle::Button.resolve(ui.style());

                    let color = if response.hovered() {
                        ui.style().visuals.strong_text_color()
                    } else {
                        ui.style().visuals.text_color()
                    };

                    back_painter.text(
                        stroke_rect.right_center() + Vec2::new(4., 0.),
                        Align2::LEFT_CENTER,
                        XTERM_256_PALETTE[idx].0,
                        font_id,
                        color,
                    );

                    let buffer_view = editor.buffer_view.clone();
                    if response.clicked() {
                        let color = buffer_view
                            .lock()
                            .get_buffer_mut()
                            .palette
                            .insert_color_rgb(r, g, b);
                        buffer_view.lock().get_caret_mut().set_foreground(color);
                        response.mark_changed();
                    }

                    if response.secondary_clicked() {
                        let color = buffer_view
                            .lock()
                            .get_buffer_mut()
                            .palette
                            .insert_color_rgb(r, g, b);
                        buffer_view.lock().get_caret_mut().set_background(color);
                        response.mark_changed();
                    }
                    response
                });
            }
        });
}

static mut CUR_COLOR: [u8; 3] = [0xFF, 0xFF, 0xFF];
static mut RENAME_INDEX: Option<usize> = None;
static mut EDIT_INDEX: Option<usize> = None;

pub fn show_custom_palette(ui: &mut egui::Ui, editor: &AnsiEditor) {
    let pal = Settings::get_custom_palette();

    let row_height = 24.0;
    egui::ScrollArea::vertical()
        .id_source("bitfont_scroll_area")
        .max_height(200.)
        .show_rows(ui, row_height, pal.len() + 1, |ui, range| {
            for idx in range {
                if idx > pal.len() {
                    break;
                }
                if idx == pal.len() {
                    ui.horizontal(|ui| {
                        ui.add_space(4.0);
                        unsafe {
                            color_picker::color_edit_button_srgb(ui, &mut CUR_COLOR);
                            if ui.button("+").clicked() {
                                Settings::get_custom_palette().push_rgb(
                                    CUR_COLOR[0],
                                    CUR_COLOR[1],
                                    CUR_COLOR[2],
                                );
                            }
                        }
                    });
                    continue;
                }

                unsafe {
                    if let Some(edit_index) = EDIT_INDEX {
                        if edit_index == idx {
                            ui.horizontal(|ui| {
                                ui.add_space(4.0);
                                let (r, g, b) = pal.colors[idx].get_rgb();
                                let mut col = [r, g, b];
                                color_picker::color_edit_button_srgb(ui, &mut col);
                                pal.colors[idx].set_rgb(col[0], col[1], col[2]);

                                if ui
                                    .button(fl!(crate::LANGUAGE_LOADER, "new-file-ok"))
                                    .clicked()
                                {
                                    EDIT_INDEX = None;
                                }
                            });
                            continue;
                        }
                    }
                }

                let mut in_rename = false;
                unsafe {
                    if let Some(rename_index) = RENAME_INDEX {
                        if rename_index == idx {
                            in_rename = true;
                        }
                    }
                }

                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    let width = if in_rename {
                        55.0
                    } else {
                        ui.available_width()
                    };
                    let (id, back_rect) = ui.allocate_space(Vec2::new(width, row_height));
                    let mut response = ui.interact(
                        Rect::from_min_size(
                            back_rect.min,
                            Vec2::new(back_rect.width(), back_rect.height()),
                        ),
                        id,
                        Sense::click(),
                    );

                    let back_painter = ui.painter_at(back_rect);

                    let is_hovered = response.hovered();

                    if is_hovered {
                        back_painter.rect_filled(
                            back_rect,
                            Rounding::none(),
                            ui.style().visuals.widgets.active.bg_fill,
                        );
                    }

                    let stroke_rect = Rect::from_min_size(
                        back_rect.min + Vec2::new(0.0, 1.0),
                        Vec2::new(52.0, 22.0),
                    );

                    let painter = ui.painter_at(stroke_rect);

                    let (r, g, b) = pal.colors[idx].get_rgb();
                    painter.rect_filled(stroke_rect, Rounding::none(), Color32::BLACK);
                    painter.rect_filled(stroke_rect.shrink(1.0), Rounding::none(), Color32::WHITE);
                    let color = Color32::from_rgb(r, g, b);
                    painter.rect_filled(stroke_rect.shrink(2.0), Rounding::none(), color);

                    let text_color =
                        if (r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) > 186.0 {
                            Color32::BLACK
                        } else {
                            Color32::WHITE
                        };

                    let text = pal.colors[idx].get_rgb_text();
                    let font_id: eframe::epaint::FontId = FontId::monospace(10.0);
                    painter.text(
                        stroke_rect.left_center() + Vec2::new(4., 0.),
                        Align2::LEFT_CENTER,
                        text,
                        font_id,
                        text_color,
                    );

                    let font_id = TextStyle::Button.resolve(ui.style());

                    let color = if response.hovered() {
                        ui.style().visuals.strong_text_color()
                    } else {
                        ui.style().visuals.text_color()
                    };

                    if in_rename {
                        let mut name = pal.colors[idx].get_name();
                        ui.add(TextEdit::singleline(&mut name).desired_width(120.0));
                        pal.colors[idx].set_name(name);

                        if ui
                            .button(fl!(crate::LANGUAGE_LOADER, "new-file-ok"))
                            .clicked()
                        {
                            unsafe {
                                RENAME_INDEX = None;
                            }
                        }
                    } else {
                        back_painter.text(
                            stroke_rect.right_center() + Vec2::new(4., 0.),
                            Align2::LEFT_CENTER,
                            pal.colors[idx].get_name(),
                            font_id,
                            color,
                        );

                        let buffer_view = editor.buffer_view.clone();
                        let mut add_context_menu = true;
                        if let Some(pos) = response.interact_pointer_pos() {
                            if stroke_rect.contains(pos) {
                                add_context_menu = false;
                                if response.clicked() {
                                    let color = buffer_view
                                        .lock()
                                        .get_buffer_mut()
                                        .palette
                                        .insert_color_rgb(r, g, b);
                                    buffer_view.lock().get_caret_mut().set_foreground(color);
                                }

                                if response.secondary_clicked() {
                                    let color = buffer_view
                                        .lock()
                                        .get_buffer_mut()
                                        .palette
                                        .insert_color_rgb(r, g, b);
                                    buffer_view.lock().get_caret_mut().set_background(color);
                                    return response;
                                }
                            }
                        }

                        if add_context_menu {
                            response = response.context_menu(|ui| {
                                if ui
                                    .button(fl!(crate::LANGUAGE_LOADER, "menu-rename"))
                                    .clicked()
                                {
                                    unsafe {
                                        RENAME_INDEX = Some(idx);
                                    }
                                    ui.close_menu();
                                }

                                if ui
                                    .button(fl!(crate::LANGUAGE_LOADER, "menu-edit"))
                                    .clicked()
                                {
                                    unsafe {
                                        EDIT_INDEX = Some(idx);
                                    }
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui
                                    .button(fl!(crate::LANGUAGE_LOADER, "menu-delete"))
                                    .clicked()
                                {
                                    if idx < pal.colors.len() {
                                        pal.colors.remove(idx);
                                    }
                                    ui.close_menu();
                                }
                            });
                        }
                    }

                    response
                });
            }
        });
}
