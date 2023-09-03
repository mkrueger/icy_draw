use std::sync::{Arc, Mutex};

use eframe::{
    egui::{self, RichText, Sense, TextStyle},
    emath::Align2,
    epaint::{pos2, Color32, Rect, Rounding, Vec2},
};
use egui_extras::RetainedImage;
use i18n_embed_fl::fl;

use crate::{AnsiEditor, Document, Message, ToolWindow, INVISIBLE_SVG, VISIBLE_SVG};

#[derive(Default)]
pub struct LayerToolWindow {}

impl ToolWindow for LayerToolWindow {
    fn get_title(&self) -> String {
        fl!(crate::LANGUAGE_LOADER, "layer_tool_title")
    }

    fn show_ui(
        &mut self,
        ui: &mut egui::Ui,
        active_document: Option<Arc<Mutex<Box<dyn Document>>>>,
    ) -> Option<Message> {
        if let Some(doc) = active_document {
            if let Some(editor) = doc.lock().unwrap().get_ansi_editor() {
                return show_layer_view(ui, editor);
            }
        }
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            ui.label(RichText::new(fl!(crate::LANGUAGE_LOADER, "no_document_selected")).small());
        });
        None
    }
}

fn show_layer_view(ui: &mut egui::Ui, editor: &AnsiEditor) -> Option<Message> {
    let row_height = 24.0;
    let mut result = None;

    let max = editor.buffer_view.lock().get_buffer().layers.len();
    let cur_layer = editor.get_cur_layer();
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    ui.set_height(row_height * 6.0);
    egui::ScrollArea::vertical()
        .id_source("layer_view_scroll_area")
        .max_height(180.)
        .show_rows(ui, row_height, max, |ui, range| {
            for i in range.rev() {
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    let (is_visible, title, color) = {
                        let lock = editor.buffer_view.lock();
                        let layer = &lock.get_buffer().layers[i];
                        (layer.is_visible, layer.title.clone(), layer.color)
                    };
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
                    let visible_icon_response =
                        ui.interact(stroke_rect, id.with("visible"), Sense::click());

                    let painter = ui.painter_at(stroke_rect);

                    if let Some(color) = color {
                        let (r, g, b) = color.into();
                        painter.rect_filled(
                            stroke_rect,
                            Rounding::none(),
                            Color32::from_rgb(r, g, b),
                        );
                    }

                    let image = if is_visible {
                        VISIBLE_SVG.texture_id(ui.ctx())
                    } else {
                        INVISIBLE_SVG.texture_id(ui.ctx())
                    };

                    let tint = if i == cur_layer {
                        ui.visuals().widgets.active.fg_stroke.color
                    } else {
                        ui.visuals().widgets.inactive.fg_stroke.color
                    };

                    painter.image(image, stroke_rect, uv, tint);

                    let color = if i == cur_layer {
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

                    response = response.context_menu(|ui| {
                        if ui
                            .button(fl!(
                                crate::LANGUAGE_LOADER,
                                "layer_tool_menu_layer_properties"
                            ))
                            .clicked()
                        {
                            result = Some(Message::EditLayer(i));
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui
                            .button(fl!(crate::LANGUAGE_LOADER, "layer_tool_menu_new_layer"))
                            .clicked()
                        {
                            result = Some(Message::AddLayer);
                            ui.close_menu();
                        }
                        if ui
                            .button(fl!(
                                crate::LANGUAGE_LOADER,
                                "layer_tool_menu_duplicate_layer"
                            ))
                            .clicked()
                        {
                            result = Some(Message::DuplicateLayer(i));
                            ui.close_menu();
                        }
                        if ui
                            .button(fl!(crate::LANGUAGE_LOADER, "layer_tool_menu_merge_layer"))
                            .clicked()
                        {
                            result = Some(Message::MergeLayer(i));
                            ui.close_menu();
                        }
                        if ui
                            .button(fl!(crate::LANGUAGE_LOADER, "layer_tool_menu_delete_layer"))
                            .clicked()
                        {
                            result = Some(Message::RemoveLayer(i));
                            ui.close_menu();
                        }
                    });

                    if response.clicked() {
                        result = Some(Message::SelectLayer(i));
                    }

                    if response.double_clicked() {
                        result = Some(Message::EditLayer(i));
                    }
                });
            }
        });
    ui.add_space(ui.available_height());
    ui.separator();
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.spacing_mut().item_spacing = eframe::epaint::Vec2::new(0.0, 0.0);
        let r = medium_hover_button(ui, &crate::ADD_LAYER_SVG).on_hover_ui(|ui| {
            ui.label(RichText::new(fl!(crate::LANGUAGE_LOADER, "add_layer_tooltip")).small());
        });

        if r.clicked() {
            result = Some(Message::AddLayer);
        }

        let r = medium_hover_button(ui, &crate::MOVE_UP_SVG).on_hover_ui(|ui| {
            ui.label(RichText::new(fl!(crate::LANGUAGE_LOADER, "move_layer_up_tooltip")).small());
        });

        if r.clicked() && cur_layer > 0 {
            result = Some(Message::MoveLayerUp(cur_layer));
        }

        let r = medium_hover_button(ui, &crate::MOVE_DOWN_SVG).on_hover_ui(|ui| {
            ui.label(RichText::new(fl!(crate::LANGUAGE_LOADER, "move_layer_down_tooltip")).small());
        });

        if r.clicked() && (1 + cur_layer) < max {
            result = Some(Message::MoveLayerDown(cur_layer));
        }

        let r = medium_hover_button(ui, &crate::DELETE_SVG).on_hover_ui(|ui| {
            ui.label(RichText::new(fl!(crate::LANGUAGE_LOADER, "delete_layer_tooltip")).small());
        });

        if r.clicked() && cur_layer < max {
            result = Some(Message::RemoveLayer(cur_layer));
        }
    });
    result
}

pub fn medium_hover_button(ui: &mut egui::Ui, image: &RetainedImage) -> egui::Response {
    let size_points = egui::Vec2::splat(28.0);

    let (id, rect) = ui.allocate_space(size_points);
    let response = ui.interact(rect, id, Sense::click());
    let painter = ui.painter_at(rect);

    let tint = if response.hovered() {
        ui.painter().rect_filled(
            rect,
            Rounding::same(4.0),
            ui.style().visuals.extreme_bg_color,
        );

        ui.visuals().widgets.active.fg_stroke.color
    } else {
        ui.visuals().widgets.inactive.fg_stroke.color
    };

    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    painter.image(image.texture_id(ui.ctx()), rect.shrink(4.0), uv, tint);

    response
}
