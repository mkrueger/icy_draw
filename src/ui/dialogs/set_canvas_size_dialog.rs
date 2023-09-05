use eframe::egui::{self, Layout};
use egui_modal::Modal;
use i18n_embed_fl::fl;

use crate::{AnsiEditor, Message, ModalDialog, TerminalResult};

pub struct SetCanvasSizeDialog {
    pub should_commit: bool,
    pub width: i32,
    pub height: i32,
}

impl SetCanvasSizeDialog {
    pub fn new(buf: &icy_engine::Buffer) -> Self {
        SetCanvasSizeDialog {
            should_commit: false,
            width: buf.get_width(),
            height: buf.get_height(),
        }
    }
}

impl ModalDialog for SetCanvasSizeDialog {
    fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut result = false;
        let modal = Modal::new(ctx, "set_canvas_size_dialog");

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

    fn should_commit(&self) -> bool {
        self.should_commit
    }

    fn commit(&self, _editor: &mut AnsiEditor) -> TerminalResult<Option<Message>> {
        Ok(Some(Message::ResizeBuffer(self.width, self.height)))
    }
}
