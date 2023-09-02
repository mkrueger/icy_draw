use eframe::{
    egui::{self, menu, ImageButton, Modifiers, TopBottomPanel, Ui},
    epaint::Vec2,
};
use egui_extras::RetainedImage;
use i18n_embed_fl::fl;

use crate::{button_with_shortcut, MainWindow, Message};

pub struct TopBar {
    pub dock_left: RetainedImage,
    pub dock_right: RetainedImage,
}

impl TopBar {
    pub fn new(ctx: &egui::Context) -> Self {
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
        TopBottomPanel::top("top_panel").exact_height(24.0).show(ctx, |ui| {
            result = self.main_menu(ui, frame);
        });
        result
    }

    fn main_menu(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) -> Option<Message> {
        let mut result = None;
        menu::bar(ui, |ui| {
            let mut buffer_opt = self.get_ansi_editor();

            let has_buffer = buffer_opt.is_some();

            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-file"), |ui| {
                if ui
                    .add(egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-new")).wrap(false))
                    .clicked()
                {
                    result = Some(Message::NewFile);
                    ui.close_menu();
                }

                if ui
                    .add(egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-open")).wrap(false))
                    .clicked()
                {
                    result = Some(Message::OpenFile);
                    ui.close_menu();
                }
                ui.separator();
                if ui
                    .add_enabled(
                        has_buffer,
                        egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-save")).wrap(false),
                    )
                    .clicked()
                {
                    result = Some(Message::SaveFile);
                    ui.close_menu();
                }
                if ui
                    .add_enabled(
                        has_buffer,
                        egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-save-as")).wrap(false),
                    )
                    .clicked()
                {
                    result = Some(Message::SaveFileAs);
                    ui.close_menu();
                }

                if ui
                    .add_enabled(
                        has_buffer,
                        egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-export")).wrap(false),
                    )
                    .clicked()
                {
                    result = Some(Message::ExportFile);
                    ui.close_menu();
                }
                ui.separator();

                if ui
                    .add(
                        egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-edit-font-outline"))
                            .wrap(false),
                    )
                    .clicked()
                {
                    result = Some(Message::ShowOutlineDialog);
                    ui.close_menu();
                }

                ui.separator();
                let button = button_with_shortcut(
                    ui,
                    true,
                    fl!(crate::LANGUAGE_LOADER, "menu-close"),
                    "Ctrl+Q",
                );
                if button.clicked() {
                    frame.close();
                    ui.close_menu();
                }
            });

            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-edit"), |ui| {
                let button = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-undo"),
                    "Ctrl+Z",
                );
                if button.clicked() {
                    result = Some(Message::Undo);
                    ui.close_menu();
                }

                let button = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-redo"),
                    "Ctrl+Shift+Z",
                );
                if button.clicked() {
                    result = Some(Message::Redo);
                    ui.close_menu();
                }
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
                let button = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-select-all"),
                    "Ctrl+A",
                );
                if button.clicked() {
                    result = Some(Message::SelectAll);
                    ui.close_menu();
                }

                let button = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-deselect"),
                    "Esc",
                );
                if button.clicked() {
                    result = Some(Message::Deselect);
                    ui.close_menu();
                }
                ui.separator();

                let button = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-erase"),
                    "Del",
                );
                if button.clicked() {
                    result = Some(Message::DeleteSelection);
                    ui.close_menu();
                }

                let button = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-flipx"),
                    "X",
                );
                if button.clicked() {
                    if let Some(editor) = self.get_ansi_editor() {
                        editor.flip_x();
                            editor.redraw_view();
                        }
                    ui.close_menu();
                }

                let button = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-flipy"),
                    "Y",
                );
                if button.clicked() {
                    if let Some(editor) = self.get_ansi_editor() {
                        editor.flip_y();
                            editor.redraw_view();
                        }
                    ui.close_menu();
                }

                let button = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-justifycenter"),
                    "Y",
                );
                if button.clicked() {
                    if let Some(editor) = self.get_ansi_editor() {
                        editor.justify_center();
                            editor.redraw_view();
                        }
                    ui.close_menu();
                }

                let button = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-justifyleft"),
                    "L",
                );
                if button.clicked() {
                    if let Some(editor) = self.get_ansi_editor() {
                        editor.justify_left();
                            editor.redraw_view();
                    }
                    ui.close_menu();
                }

                let button = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-justifyright"),
                    "R",
                );
                if button.clicked() {
                    if let Some(editor) = self.get_ansi_editor() {
                        editor.justify_right();
                            editor.redraw_view();
                    }
                    ui.close_menu();
                }
                ui.separator();

                let button = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-crop"),
                    "",
                );
                if button.clicked() {
                    if let Some(editor) = self.get_ansi_editor() {
                        editor.crop();
                            editor.redraw_view();
                    }
                    ui.close_menu();
                }
            });

            ui.menu_button("View", |ui| {
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
                if ui
                    .button(fl!(crate::LANGUAGE_LOADER, "menu-about"))
                    .clicked()
                {
                    result = Some(Message::ShowAboutDialog);
                    ui.close_menu();
                }
            });
            self.top_bar_ui(ui, frame);
        });

        if ui.input(|i| i.key_pressed(egui::Key::Q) && i.modifiers.ctrl) {
            frame.close();
        }

        if ui.input(|i| i.key_pressed(egui::Key::A) && i.modifiers.ctrl) {
            ui.input_mut(|i| i.consume_key(Modifiers::CTRL, egui::Key::A));
            result = Some(Message::SelectAll);
        }

        if ui.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.ctrl && !i.modifiers.shift) {
            ui.input_mut(|i| i.consume_key(Modifiers::CTRL, egui::Key::Z));
            result = Some(Message::Undo);
        }

        if ui.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.shift && i.modifiers.ctrl) {
            ui.input_mut(|i| i.consume_key(crate::CTRL_SHIFT, egui::Key::Z));
            result = Some(Message::Redo);
        }
        result
    }

    fn top_bar_ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let tint = if self.right_panel {
                ui.visuals().widgets.active.fg_stroke.color
            } else {
                ui.visuals().widgets.inactive.fg_stroke.color
            };
            let icon_size = 20.0;

            let right = ui.add(
                ImageButton::new(
                    self.top_bar.dock_right.texture_id(ui.ctx()),
                    Vec2::new(icon_size, icon_size),
                )
                .tint(tint),
            );
            if right.clicked() {
                self.right_panel = !self.right_panel;
            }

            let tint = if self.left_panel {
                ui.visuals().widgets.active.fg_stroke.color
            } else {
                ui.visuals().widgets.inactive.fg_stroke.color
            };

            let left = ui.add(
                ImageButton::new(
                    self.top_bar.dock_left.texture_id(ui.ctx()),
                    Vec2::new(icon_size, icon_size),
                )
                .tint(tint),
            );
            if left.clicked() {
                self.left_panel = !self.left_panel;
            }
        });
    }
}
