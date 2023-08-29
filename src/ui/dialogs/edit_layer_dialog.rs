use eframe::egui::{self, Layout};
use egui_modal::Modal;
use i18n_embed_fl::fl;
use icy_engine::Position;

use crate::{AnsiEditor, ModalDialog, TerminalResult};

pub struct EditLayerDialog {
    pub should_commit: bool,

    layer: usize,

    title: String,
    is_visible: bool,
    is_locked: bool,
    is_position_locked: bool,

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
            is_visible: l.is_visible,
            is_locked: l.is_locked,
            is_position_locked: l.is_position_locked,
            x_offset: l.offset.x,
            y_offset: l.offset.y,
        }
    }
}

impl ModalDialog for EditLayerDialog {
    fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut result = false;
        let modal: Modal = Modal::new(ctx, "my_modal");

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

    fn commit(&self, editor: &mut AnsiEditor) -> TerminalResult<bool> {
        let layer = &mut editor.buffer_view.lock().buf.layers[self.layer];
        layer.title = self.title.clone();
        layer.is_visible = self.is_visible;
        layer.is_locked = self.is_locked;
        layer.is_position_locked = self.is_position_locked;
        layer.offset = Position::new(self.x_offset, self.y_offset);

        Ok(true)
    }
}
