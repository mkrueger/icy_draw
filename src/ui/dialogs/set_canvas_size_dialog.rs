use eframe::egui::{self, Layout};
use egui_modal::Modal;
use i18n_embed_fl::fl;
use icy_engine::Size;

use crate::{TerminalResult, ModalDialog};

pub struct SetCanvasSizeDialog {
    pub should_commit: bool,
    pub width: i32,
    pub height: i32
}

impl SetCanvasSizeDialog {
    pub fn new(buf: &icy_engine::Buffer) -> Self {
        SetCanvasSizeDialog {
            should_commit: false,
            width: buf.get_buffer_width(),
            height: buf.get_real_buffer_height()
        }
    }
}

impl ModalDialog for SetCanvasSizeDialog {

    fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut result = false;
        let modal = Modal::new(ctx, "my_modal");

        modal.show(|ui| {
            modal.title(ui, fl!(crate::LANGUAGE_LOADER, "edit-canvas-size-title"));

            modal.frame(ui, |ui| {
                egui::Grid::new("some_unique_id")
                    .num_columns(2)
                    .spacing([4.0, 8.0])
                    .show(ui, |ui| {
                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(fl!(crate::LANGUAGE_LOADER, "edit-canvas-size-width-label"));
                        });
                        ui.add(egui::DragValue::new(&mut self.width));
                        ui.end_row();

                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(fl!(crate::LANGUAGE_LOADER, "edit-canvas-size-height-label"));
                        });
                        ui.add(egui::DragValue::new(&mut self.height));
                        ui.end_row();

                    });
                ui.add_space(4.0);
            });

            modal.buttons(ui, |ui| {
                if ui
                    .button(fl!(crate::LANGUAGE_LOADER, "edit-canvas-size-resize"))
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

    fn should_commit(&self) -> bool { self.should_commit }

    fn commit(&self, editor: &mut crate::model::Editor) -> TerminalResult<bool> {
        editor.buf.set_buffer_size(Size::new(self.width, self.height));
        Ok(true)
    }
}
