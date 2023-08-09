use crate::ansi_editor::BufferView;
use eframe::{
    egui::{self, RichText},
    epaint::Vec2,
};
use egui_extras::{Column, TableBuilder};
use i18n_embed_fl::fl;
use std::sync::{Arc, Mutex};

pub fn show_layer_view(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    buffer_opt: Option<Arc<Mutex<BufferView>>>,
) {
    if buffer_opt.is_none() {
        return;
    }
    let buffer_opt = buffer_opt.unwrap();
    let max = buffer_opt.lock().unwrap().editor.buf.layers.len();
    let cur_layer = buffer_opt.lock().unwrap().editor.cur_layer;

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
                let (is_visible, title) = {
                    let layer = &buffer_opt.lock().unwrap().editor.buf.layers[i];
                    (layer.is_visible, layer.title.clone())
                };

                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        let r = ui
                            .add(egui::ImageButton::new(
                                if is_visible {
                                    super::VISIBLE_SVG.texture_id(ctx)
                                } else {
                                    super::INVISIBLE_SVG.texture_id(ctx)
                                },
                                Vec2::new(16., 16.),
                            ))
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
                            buffer_opt.lock().unwrap().editor.buf.layers[i].is_visible =
                                !is_visible;
                        }
                    });
                    row.col(|ui| {
                        let r = ui.selectable_label((i as i32) == cur_layer, &title);
                        if r.clicked() {
                            buffer_opt.lock().unwrap().editor.cur_layer = i as i32;
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
            let mut new_layer = icy_engine::Layer::new();
            new_layer.title = "New layer".to_string();
            buffer_opt
                .lock()
                .unwrap()
                .editor
                .buf
                .layers
                .insert(0, new_layer);
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
            buffer_opt
                .lock()
                .unwrap()
                .editor
                .buf
                .layers
                .swap(cur_layer as usize, cur_layer as usize - 1);
            buffer_opt.lock().unwrap().editor.cur_layer -= 1;
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

        if r.clicked() && (1 + cur_layer as usize) < max {
            buffer_opt
                .lock()
                .unwrap()
                .editor
                .buf
                .layers
                .swap(cur_layer as usize, cur_layer as usize + 1);
            buffer_opt.lock().unwrap().editor.cur_layer += 1;
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

        if r.clicked() && cur_layer >= 0 && cur_layer < max as i32 {
            buffer_opt
                .lock()
                .unwrap()
                .editor
                .buf
                .layers
                .remove(cur_layer as usize);
            buffer_opt.lock().unwrap().editor.cur_layer =
                std::cmp::min(cur_layer, (max as i32) - 1);
        }
    });
}
