use eframe::egui::{self, color_picker, Layout, RichText};
use egui_modal::Modal;
use i18n_embed_fl::fl;
use icy_engine::{Color, Mode};

use crate::{AnsiEditor, Message, ModalDialog, TerminalResult};

pub struct EditLayerDialog {
    pub should_commit: bool,

    layer: usize,

    title: String,
    is_visible: bool,
    is_locked: bool,
    is_position_locked: bool,
    has_alpha_channel: bool,
    is_alpha_channel_locked: bool,

    mode: Mode,

    color: Option<Color>,

    x_offset: i32,
    y_offset: i32,
}

impl EditLayerDialog {
    pub fn new(buf: &icy_engine::Buffer, layer: usize) -> Self {
        let l = &buf.layers[layer];
        EditLayerDialog {
            should_commit: false,
            layer,
            title: l.title.clone(),
            color: l.color,
            is_visible: l.is_visible,
            is_locked: l.is_locked,
            is_position_locked: l.is_position_locked,
            has_alpha_channel: l.has_alpha_channel,
            is_alpha_channel_locked: l.is_alpha_channel_locked,
            mode: l.mode,
            x_offset: l.get_offset().x,
            y_offset: l.get_offset().y,
        }
    }
}

impl ModalDialog for EditLayerDialog {
    fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut result = false;
        let modal: Modal = Modal::new(ctx, "edit_layer_dialog");

        modal.show(|ui| {
            modal.title(ui, fl!(crate::LANGUAGE_LOADER, "edit-layer-dialog-title"));

            modal.frame(ui, |ui| {
                egui::Grid::new("some_unique_id")
                    .num_columns(2)
                    .spacing([4.0, 8.0])
                    .show(ui, |ui| {
                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(fl!(crate::LANGUAGE_LOADER, "edit-layer-dialog-name-label"));
                        });
                        ui.add(egui::TextEdit::singleline(&mut self.title));
                        ui.end_row();

                        if self.color.is_some() {
                            let mut use_color = true;

                            if let Some(color) = &mut self.color {
                                ui.label("");
                                ui.horizontal(|ui| {
                                    let mut c = (*color).into();
                                    color_picker::color_edit_button_srgb(ui, &mut c);
                                    *color = c.into();

                                    ui.checkbox(&mut use_color, "Use Color");
                                });
                                ui.end_row();
                            }

                            if !use_color {
                                self.color = None;
                            }
                        } else {
                            ui.label("");
                            let mut use_color = false;
                            ui.checkbox(&mut use_color, "Use Color");
                            ui.end_row();
                            if use_color {
                                self.color = Some(Color::new(255, 255, 255));
                            }
                        }

                        ui.label("");
                        ui.checkbox(
                            &mut self.is_visible,
                            fl!(
                                crate::LANGUAGE_LOADER,
                                "edit-layer-dialog-is-visible-checkbox"
                            ),
                        );
                        ui.end_row();
                        ui.label("");
                        ui.checkbox(
                            &mut self.is_locked,
                            fl!(
                                crate::LANGUAGE_LOADER,
                                "edit-layer-dialog-is-edit-locked-checkbox"
                            ),
                        );
                        ui.end_row();
                        ui.label("");
                        ui.checkbox(
                            &mut self.is_position_locked,
                            fl!(
                                crate::LANGUAGE_LOADER,
                                "edit-layer-dialog-is-position-locked-checkbox"
                            ),
                        );
                        ui.end_row();

                        ui.label("");
                        ui.checkbox(
                            &mut self.has_alpha_channel,
                            fl!(
                                crate::LANGUAGE_LOADER,
                                "edit-layer-dialog-has-alpha-checkbox"
                            ),
                        );
                        ui.end_row();

                        if self.has_alpha_channel {
                            ui.label("");
                            ui.checkbox(
                                &mut self.is_alpha_channel_locked,
                                fl!(
                                    crate::LANGUAGE_LOADER,
                                    "edit-layer-dialog-is-alpha-locked-checkbox"
                                ),
                            );
                            ui.end_row();
                        }

                        ui.label("Mode:");

                        egui::ComboBox::from_id_source("combobox1")
                            .selected_text(RichText::new(match self.mode {
                                Mode::Normal => "Normal",
                                Mode::Chars => "Chars only",
                                Mode::Attributes => "Attribute only",
                            }))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.mode, Mode::Normal, "Normal");
                                ui.selectable_value(&mut self.mode, Mode::Chars, "Chars only");
                                ui.selectable_value(
                                    &mut self.mode,
                                    Mode::Attributes,
                                    "Attribute only",
                                );
                            });
                        ui.end_row();

                        ui.label(fl!(
                            crate::LANGUAGE_LOADER,
                            "edit-layer-dialog-is-x-offset-label"
                        ));
                        ui.add(egui::DragValue::new(&mut self.x_offset));
                        ui.end_row();

                        ui.label(fl!(
                            crate::LANGUAGE_LOADER,
                            "edit-layer-dialog-is-y-offset-label"
                        ));
                        ui.add(egui::DragValue::new(&mut self.y_offset));
                        ui.end_row();
                    });
                ui.add_space(16.0);
            });

            modal.buttons(ui, |ui| {
                if ui
                    .button(fl!(crate::LANGUAGE_LOADER, "new-file-ok"))
                    .clicked()
                {
                    self.should_commit = true;
                    result = true;
                }
                if ui
                    .button(fl!(crate::LANGUAGE_LOADER, "new-file-cancel"))
                    .clicked()
                {
                    result = true;
                }
            });
        });
        modal.open();
        result
    }

    fn should_commit(&self) -> bool {
        self.should_commit
    }

    fn commit(&self, editor: &mut AnsiEditor) -> TerminalResult<Option<Message>> {
        let mut bv = editor.buffer_view.lock();
        let layer = &mut bv.get_buffer_mut().layers[self.layer];
        layer.title = self.title.clone();
        layer.color = self.color;
        layer.is_visible = self.is_visible;
        layer.is_locked = self.is_locked;
        layer.is_position_locked = self.is_position_locked;
        layer.set_offset((self.x_offset, self.y_offset));
        layer.has_alpha_channel = self.has_alpha_channel;
        layer.is_alpha_channel_locked = self.is_alpha_channel_locked;
        layer.mode = self.mode;

        Ok(None)
    }
}
