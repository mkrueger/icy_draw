use crate::ansi_editor::BufferView;
use crate::model::brush_imp::draw_glyph;
use eframe::{
    egui::{self, RichText},
    epaint::Vec2,
};
use egui_extras::{Column, TableBuilder};
use i18n_embed_fl::fl;
use icy_engine::{AsciiParser, BufferParser};
use std::sync::{Arc, Mutex};

pub fn show_layer_view(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    buffer_opt: Option<Arc<Mutex<BufferView>>>,
) {
    if buffer_opt.is_none() {
        return;
    }

    let mut table = TableBuilder::new(ui)
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
            for layer in &mut buffer_opt.unwrap().lock().unwrap().editor.buf.layers {
                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        if ui
                            .add(egui::ImageButton::new(
                                if layer.is_visible {
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
                            })
                            .clicked()
                        {
                            layer.is_visible = !layer.is_visible;
                        }
                    });
                    row.col(|ui| {
                        ui.label(&layer.title);
                    });
                });
            }
        });

    let img_size = Vec2::new(24., 24.);
    ui.horizontal(|ui| {
        if ui
            .add(egui::ImageButton::new(
                super::ADD_LAYER_SVG.texture_id(ctx),
                img_size,
            ))
            .on_hover_ui(|ui| {
                ui.label(RichText::new(fl!(crate::LANGUAGE_LOADER, "add_layer_tooltip")).small());
            })
            .clicked()
        {}

        if ui
            .add(egui::ImageButton::new(
                super::MOVE_UP_SVG.texture_id(ctx),
                img_size,
            ))
            .on_hover_ui(|ui| {
                ui.label(
                    RichText::new(fl!(crate::LANGUAGE_LOADER, "move_layer_up_tooltip")).small(),
                );
            })
            .clicked()
        {}

        if ui
            .add(egui::ImageButton::new(
                super::MOVE_DOWN_SVG.texture_id(ctx),
                img_size,
            ))
            .on_hover_ui(|ui| {
                ui.label(
                    RichText::new(fl!(crate::LANGUAGE_LOADER, "move_layer_down_tooltip")).small(),
                );
            })
            .clicked()
        {}

        if ui
            .add(egui::ImageButton::new(
                super::DELETE_SVG.texture_id(ctx),
                img_size,
            ))
            .on_hover_ui(|ui| {
                ui.label(
                    RichText::new(fl!(crate::LANGUAGE_LOADER, "delete_layer_tooltip")).small(),
                );
            })
            .clicked()
        {}
    });
}
