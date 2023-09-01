use eframe::{
    egui::{self, menu, Modifiers, TopBottomPanel, Ui},
    epaint::Vec2,
};
use i18n_embed_fl::fl;

use crate::{button_with_shortcut, MainWindow, Message};

impl MainWindow {
    pub fn show_top_bar(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
    ) -> Option<Message> {
        let mut result = None;
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            result = self.main_menu(ui, frame);
        });
        result
    }

    fn main_menu(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) -> Option<Message> {
        let mut result = None;
        menu::bar(ui, |ui| {
            let mut buffer_opt = None;
            if let Some(doc) = self.get_active_document_mut() {
                buffer_opt = doc.get_ansi_editor_mut();
            }

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
                    if let Some(doc) = self.get_active_document_mut() {
                        let doc = doc.get_ansi_editor_mut();
                        if let Some(editor) = doc {
                            editor.flip_x();
                            editor.redraw_view();
                        }
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
                    if let Some(doc) = self.get_active_document_mut() {
                        let doc = doc.get_ansi_editor_mut();
                        if let Some(editor) = doc {
                            editor.flip_y();
                            editor.redraw_view();
                        }
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
                    if let Some(doc) = self.get_active_document_mut() {
                        let doc = doc.get_ansi_editor_mut();
                        if let Some(editor) = doc {
                            editor.justify_center();
                            editor.redraw_view();
                        }
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
                    if let Some(doc) = self.get_active_document_mut() {
                        let doc = doc.get_ansi_editor_mut();
                        if let Some(editor) = doc {
                            editor.justify_left();
                            editor.redraw_view();
                        }
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
                    if let Some(doc) = self.get_active_document_mut() {
                        let doc = doc.get_ansi_editor_mut();
                        if let Some(editor) = doc {
                            editor.justify_right();
                            editor.redraw_view();
                        }
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
                    if let Some(doc) = self.get_active_document_mut() {
                        let doc = doc.get_ansi_editor_mut();
                        if let Some(editor) = doc {
                            editor.crop();
                            editor.redraw_view();
                        }
                    }
                    ui.close_menu();
                }
            });

            ui.menu_button("View", |ui| {
                if ui.button("100%").clicked() {
                    self.tab_viewer.document_options.scale = Vec2::new(1.0, 1.0);
                    ui.close_menu();
                }
                if ui.button("200%").clicked() {
                    self.tab_viewer.document_options.scale = Vec2::new(2.0, 2.0);
                    ui.close_menu();
                }
                if ui.button("300%").clicked() {
                    self.tab_viewer.document_options.scale = Vec2::new(3.0, 3.0);
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
            //self.top_bar_ui(ui, frame);
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
    /*
    fn top_bar_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // From right-to-left:
            /*
            if hypex_ui::CUSTOM_WINDOW_DECORATIONS {
                ui.add_space(8.0);
                hypex_ui::native_window_buttons_ui(frame, ui);
                ui.separator();
            } else {
                ui.add_space(16.0);
            }

            self.hypex_ui.medium_icon_toggle_button(
                ui,
                &hypex_ui::icons::RIGHT_PANEL_TOGGLE,
                &mut self.right_panel,
            ); /*
               self.hypex_ui.medium_icon_toggle_button(
                   ui,
                   &hypex_ui::icons::BOTTOM_PANEL_TOGGLE,
                   &mut self.bottom_panel,
               );*/
            self.hypex_ui.medium_icon_toggle_button(
                ui,
                &hypex_ui::icons::LEFT_PANEL_TOGGLE,
                &mut self.left_panel,
            );*/
        });
    }}*/
}
