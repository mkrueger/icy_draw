use eframe::{
    egui::{self, RichText},
    epaint::{Vec2, Color32},
};
use egui_extras::{Column, TableBuilder};
use i18n_embed_fl::fl;

use crate::AnsiEditor;

pub enum Message {
    NewLayer,
    EditLayer(usize),
    DeleteLayer(usize),
    MoveLayerUp(usize),
    MoveLayerDown(usize),
    ToggleVisibility(usize),
    SelectLayer(usize),
}

pub fn show_layer_view(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    editor: &AnsiEditor,
) -> Option<Message> {
    let mut result = None;

    let max = editor.buffer_view.lock().buf.layers.len();
    let cur_layer = editor.cur_layer;

    let table = TableBuilder::new(ui)
        .striped(false)
        .resizable(false)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::initial(40.0).at_least(40.0).clip(true))
        .column(Column::remainder())
        .min_scrolled_height(0.0);

    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("Visible");
            });
            header.col(|ui| {
                ui.strong("Layer title");
            });
        })
        .body(|mut body| {
            for i in 0..max {
                let (is_visible, title, color) = {
                    let layer = &editor.buffer_view.lock().buf.layers[i];
                    (layer.is_visible, layer.title.clone(), layer.color)
                };

                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        let r = ui
                            .add(
                                egui::ImageButton::new(
                                    if is_visible {
                                        super::VISIBLE_SVG.texture_id(ctx)
                                    } else {
                                        super::INVISIBLE_SVG.texture_id(ctx)
                                    },
                                    Vec2::new(16., 16.),
                                )
                                .frame(false),
                            )
                            .on_hover_ui(|ui| {
                                ui.label(
                                    RichText::new(fl!(
                                        crate::LANGUAGE_LOADER,
                                        "move_layer_up_tooltip"
                                    ))
                                    .small(),
                                );
                            });

                        if r.clicked() {
                            result = Some(Message::ToggleVisibility(i));
                        }
                    });
                    row.col(|ui| {
                        let mut text = RichText::new(title);
                        if let Some(color) = color {
                            let (r, g, b) = color.into();
                            text = text.color(Color32::from_rgb(r, g, b));
                        }
                        let r = ui.selectable_label(i == cur_layer, text);
                        if r.clicked() {
                            result = Some(Message::SelectLayer(i));
                        }

                        if r.double_clicked() {
                            result = Some(Message::EditLayer(i));
                        }
                    });
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
