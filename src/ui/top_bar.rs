use eframe::{
    egui::{self, menu, ImageButton, TopBottomPanel, Ui},
    epaint::Vec2,
};
use egui_extras::RetainedImage;
use i18n_embed_fl::fl;
use icy_engine::util::{pop_data, BUFFER_DATA};

use crate::{button_with_shortcut, MainWindow, Message};

pub struct TopBar {
    pub dock_left: RetainedImage,
    pub dock_right: RetainedImage,
}

impl TopBar {
    pub fn new(_ctx: &egui::Context) -> Self {
        let left_bytes = include_bytes!("../../data/icons/dock_left.svg");
        let right_bytes = include_bytes!("../../data/icons/dock_right.svg");

        Self {
            dock_left: RetainedImage::from_svg_bytes("dock_left.svg", left_bytes).unwrap(),
            dock_right: RetainedImage::from_svg_bytes("dock_right.svg", right_bytes).unwrap(),
        }
    }
}

impl MainWindow {
    pub fn show_top_bar(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
    ) -> Option<Message> {
        let mut result = None;
        TopBottomPanel::top("top_panel")
            .exact_height(24.0)
            .show(ctx, |ui| {
                result = self.main_menu(ui, frame);
            });
        result
    }

    fn main_menu(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) -> Option<Message> {
        let mut result = None;
        menu::bar(ui, |ui| {
            let mut has_buffer = false;
            let mut is_dirty = false;
            if let Some(doc) = self.get_active_document() {
                has_buffer = doc.lock().unwrap().get_ansi_editor().is_some();
                is_dirty = doc.lock().unwrap().is_dirty();
            }

            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-file"), |ui| {
                self.commands.new_file.ui(ui, &mut result);
                self.commands.open_file.ui(ui, &mut result);
                ui.separator();
                self.commands.save.ui_enabled(ui, is_dirty, &mut result);
                self.commands.save_as.ui_enabled(ui, is_dirty, &mut result);
                self.commands.export.ui_enabled(ui, has_buffer, &mut result);
                ui.separator();
                self.commands.edit_font_outline.ui(ui, &mut result);
                ui.separator();
                self.commands.close_window.ui(ui, &mut result);
            });

            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-edit"), |ui| {
                ui.set_width(250.0);
                if let Some(doc) = self.get_active_document() {
                    if doc.lock().unwrap().can_undo() {
                        let enabled = doc.lock().unwrap().can_undo();

                        let button = button_with_shortcut(
                            ui,
                            enabled,
                            fl!(
                                crate::LANGUAGE_LOADER,
                                "menu-undo-op",
                                op = doc.lock().unwrap().undo_description().unwrap()
                            ),
                            "Ctrl+Z",
                        );
                        if button.clicked() {
                            result = Some(Message::Undo);
                            ui.close_menu();
                        }
                    } else {
                        self.commands.undo.ui_enabled(ui, false, &mut result);
                    }

                    if doc.lock().unwrap().can_redo() {
                        let button = button_with_shortcut(
                            ui,
                            true,
                            fl!(
                                crate::LANGUAGE_LOADER,
                                "menu-redo-op",
                                op = doc.lock().unwrap().redo_description().unwrap()
                            ),
                            "Ctrl+Shift+Z",
                        );
                        if button.clicked() {
                            result = Some(Message::Redo);
                            ui.close_menu();
                        }
                    } else {
                        self.commands.redo.ui_enabled(ui, false, &mut result);
                    }
                } else {
                    self.commands.undo.ui_enabled(ui, false, &mut result);
                    self.commands.redo.ui_enabled(ui, false, &mut result);
                }
                ui.separator();
                if let Some(doc) = self.get_active_document() {
                    self.commands
                        .cut
                        .ui_enabled(ui, doc.lock().unwrap().can_cut(), &mut result);
                    self.commands
                        .copy
                        .ui_enabled(ui, doc.lock().unwrap().can_copy(), &mut result);
                    self.commands.paste.ui_enabled(
                        ui,
                        doc.lock().unwrap().can_paste(),
                        &mut result,
                    );
                }

                ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-paste-as"), |ui| {
                    let button = button_with_shortcut(
                        ui,
                        pop_data(BUFFER_DATA).is_some(),
                        fl!(crate::LANGUAGE_LOADER, "menu-paste-as-new-image"),
                        "",
                    );
                    if button.clicked() {
                        result = Some(Message::PasteAsNewImage);
                        ui.close_menu();
                    }

                    let button = button_with_shortcut(
                        ui,
                        pop_data(BUFFER_DATA).is_some(),
                        fl!(crate::LANGUAGE_LOADER, "menu-paste-as-brush"),
                        "",
                    );
                    if button.clicked() {
                        result = Some(Message::PasteAsBrush);
                        ui.close_menu();
                    }
                });
                ui.separator();

                self.commands
                    .justify_line_left
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .justify_line_right
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .justify_line_center
                    .ui_enabled(ui, has_buffer, &mut result);
                ui.separator();
                self.commands
                    .insert_row
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .delete_row
                    .ui_enabled(ui, has_buffer, &mut result);
                ui.separator();
                self.commands
                    .insert_column
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .delete_column
                    .ui_enabled(ui, has_buffer, &mut result);
                ui.separator();
                self.commands
                    .erase_row
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .erase_row_to_start
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .erase_row_to_end
                    .ui_enabled(ui, has_buffer, &mut result);
                ui.separator();
                self.commands
                    .erase_column
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .erase_column_to_end
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .erase_column_to_start
                    .ui_enabled(ui, has_buffer, &mut result);
                ui.separator();
                self.commands
                    .scroll_area_up
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .scroll_area_down
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .scroll_area_left
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .scroll_area_right
                    .ui_enabled(ui, has_buffer, &mut result);
                ui.separator();
                if ui
                    .add_enabled(
                        has_buffer,
                        egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-edit-sauce"))
                            .wrap(false),
                    )
                    .clicked()
                {
                    result = Some(Message::EditSauce);
                    ui.close_menu();
                }

                if ui
                    .add_enabled(
                        has_buffer,
                        egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-set-canvas-size"))
                            .wrap(false),
                    )
                    .clicked()
                {
                    result = Some(Message::SetCanvasSize);
                    ui.close_menu();
                }
            });

            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-selection"), |ui| {
                self.commands
                    .select_all
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .deselect
                    .ui_enabled(ui, has_buffer, &mut result);
                ui.separator();
                self.commands
                    .erase_selection
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands.flip_x.ui_enabled(ui, has_buffer, &mut result);
                self.commands.flip_y.ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .justifycenter
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .justifyleft
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .justifyright
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands.crop.ui_enabled(ui, has_buffer, &mut result);
            });
            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-colors"), |ui| {
                self.commands
                    .pick_attribute_under_caret
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .toggle_color
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .switch_to_default_color
                    .ui_enabled(ui, has_buffer, &mut result);
            });
            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-view"), |ui| {
                if ui.button("100%").clicked() {
                    self.document_behavior.document_options.scale = Vec2::new(1.0, 1.0);
                    ui.close_menu();
                }
                if ui.button("200%").clicked() {
                    self.document_behavior.document_options.scale = Vec2::new(2.0, 2.0);
                    ui.close_menu();
                }
                if ui.button("300%").clicked() {
                    self.document_behavior.document_options.scale = Vec2::new(3.0, 3.0);
                    ui.close_menu();
                }
                ui.separator();
                self.commands
                    .set_reference_image
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .toggle_reference_image
                    .ui_enabled(ui, has_buffer, &mut result);
                self.commands
                    .clear_reference_image
                    .ui_enabled(ui, has_buffer, &mut result);
            });

            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-help"), |ui| {
                let r = ui.hyperlink_to(
                    fl!(crate::LANGUAGE_LOADER, "menu-discuss"),
                    "https://github.com/mkrueger/icy_draw/discussions",
                );
                if r.clicked() {
                    ui.close_menu();
                }
                let r = ui.hyperlink_to(
                    fl!(crate::LANGUAGE_LOADER, "menu-report-bug"),
                    "https://github.com/mkrueger/icy_draw/issues/new",
                );
                if r.clicked() {
                    ui.close_menu();
                }
                ui.separator();
                self.commands.about.ui(ui, &mut result);
            });
            self.top_bar_ui(ui, frame);
        });

        result
    }

    fn top_bar_ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let right = medium_toggle_button(ui, &self.top_bar.dock_right, self.right_panel);
            if right.clicked() {
                self.right_panel = !self.right_panel;
            }

            let left = medium_toggle_button(ui, &self.top_bar.dock_left, self.left_panel);
            if left.clicked() {
                self.left_panel = !self.left_panel;
            }
        });
    }
}

pub fn medium_toggle_button(
    ui: &mut egui::Ui,
    icon: &RetainedImage,
    selected: bool,
) -> egui::Response {
    let size_points = egui::Vec2::splat(20.0);

    let tint = if selected {
        ui.visuals().widgets.active.fg_stroke.color
    } else {
        ui.visuals().widgets.inactive.fg_stroke.color
    };

    ui.add(ImageButton::new(icon.texture_id(ui.ctx()), size_points).tint(tint))
}
