use eframe::{
    egui::{self, RichText, Sense, TextStyle},
    epaint::{Color32, Vec2, Rounding, Rect, pos2}, emath::Align2,
};
use i18n_embed_fl::fl;

use crate::{AnsiEditor, Message};

pub fn show_layer_view(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    editor: &AnsiEditor,
) -> Option<Message> {
    let row_height = 24.0;
    let mut result = None;

    let max = editor.buffer_view.lock().buf.layers.len();
    let cur_layer = editor.cur_layer;
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));

    egui::ScrollArea::vertical()
    .id_source("layer_view_scroll_area")
    .max_height(200.)
    .show_rows(ui, row_height, max, |ui, range| {
        for i in range {           
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                let (is_visible, title, color) = {
                    let layer = &editor.buffer_view.lock().buf.layers[i];
                    (layer.is_visible, layer.title.clone(), layer.color)
                };
                let width =  ui.available_width();

                let (id, back_rect) = ui.allocate_space(Vec2::new(width, row_height));
                let response = ui.interact(back_rect, id, Sense::click());

                let back_painter = ui.painter_at(back_rect);

                if response.hovered() {
                    back_painter.rect_filled(
                        back_rect,
                        Rounding::none(),
                        ui.style().visuals.widgets.active.bg_fill,
                    );
                } else if i == cur_layer {
                    back_painter.rect_filled(
                        back_rect,
                        Rounding::none(),
                        ui.style().visuals.extreme_bg_color,
                    );
                }

                let stroke_rect = Rect::from_min_size(
                    back_rect.min + Vec2::new(0.0, 1.0),
                    Vec2::new(22.0, 22.0),
                );
                let visible_icon_response = ui.interact(stroke_rect, id.with("visible"), Sense::click());

                let painter = ui.painter_at(stroke_rect);

    
                if let Some(color) = color {
                    let (r, g, b) = color.into();
                    painter.rect_filled(
                        stroke_rect,
                        Rounding::none(),
                        Color32::from_rgb(r, g, b)
                    );
                } 

                let image = if is_visible {
                    super::VISIBLE_SVG.texture_id(ctx)
                } else {
                    super::INVISIBLE_SVG.texture_id(ctx)
                };

                let tint = if i == cur_layer {
                    ui.visuals().widgets.active.fg_stroke.color
                } else {
                    ui.visuals().widgets.inactive.fg_stroke.color
                };
                
                painter.image(image, stroke_rect, uv, tint);
                

                let color = if  i == cur_layer {
                    ui.style().visuals.strong_text_color()
                } else {
                    ui.style().visuals.text_color()
                };
                let font_id = TextStyle::Button.resolve(ui.style());

                back_painter.text(
                    stroke_rect.right_center() + Vec2::new(4., 0.),
                    Align2::LEFT_CENTER,
                    title,
                    font_id,
                    color,
                );

                if visible_icon_response.clicked() {
                    result = Some(Message::ToggleVisibility(i));
                }

                if response.clicked() {
                    result = Some(Message::SelectLayer(i));
                }

                if response.double_clicked() {
                    result = Some(Message::EditLayer(i));
                }
            });
        }
    });
            

    let img_size = Vec2::new(24., 24.);
    ui.horizontal(|ui| {
        let r = ui
            .add(egui::ImageButton::new(
                super::ADD_LAYER_SVG.texture_id(ctx),
                img_size,
            ))
            .on_hover_ui(|ui| {
                ui.label(RichText::new(fl!(crate::LANGUAGE_LOADER, "add_layer_tooltip")).small());
            });

        if r.clicked() {
            result = Some(Message::NewLayer);
        }

        let r = ui
            .add(egui::ImageButton::new(
                super::MOVE_UP_SVG.texture_id(ctx),
                img_size,
            ))
            .on_hover_ui(|ui| {
                ui.label(
                    RichText::new(fl!(crate::LANGUAGE_LOADER, "move_layer_up_tooltip")).small(),
                );
            });

        if r.clicked() && cur_layer > 0 {
            result = Some(Message::MoveLayerUp(cur_layer));
        }

        let r = ui
            .add(egui::ImageButton::new(
                super::MOVE_DOWN_SVG.texture_id(ctx),
                img_size,
            ))
            .on_hover_ui(|ui| {
                ui.label(
                    RichText::new(fl!(crate::LANGUAGE_LOADER, "move_layer_down_tooltip")).small(),
                );
            });

        if r.clicked() && (1 + cur_layer) < max {
            result = Some(Message::MoveLayerDown(cur_layer));
        }

        let r = ui
            .add(egui::ImageButton::new(
                super::DELETE_SVG.texture_id(ctx),
                img_size,
            ))
            .on_hover_ui(|ui| {
                ui.label(
                    RichText::new(fl!(crate::LANGUAGE_LOADER, "delete_layer_tooltip")).small(),
                );
            });

        if r.clicked() && cur_layer < max {
            result = Some(Message::DeleteLayer(cur_layer));
        }
    });
    result
}
