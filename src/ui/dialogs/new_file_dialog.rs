use eframe::egui::{self};
use egui_modal::Modal;
use i18n_embed_fl::fl;
use icy_engine::Buffer;

use crate::{add_child, AnsiEditor, MainWindow};

pub struct NewFileDialog {
    pub width: i32,
    pub height: i32,

    pub create: bool,
}

impl Default for NewFileDialog {
    fn default() -> Self {
        Self {
            width: 80,
            height: 25,
            create: false,
        }
    }
}

impl crate::ModalDialog for NewFileDialog {
    fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut result = false;
        let modal = Modal::new(ctx, "new_file_dialog");

        modal.show(|ui| {
            modal.title(ui, fl!(crate::LANGUAGE_LOADER, "new-file-title"));

            modal.frame(ui, |ui| {
                egui::Grid::new("some_unique_id")
                    .num_columns(2)
                    .spacing([4.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(fl!(crate::LANGUAGE_LOADER, "new-file-width"));
                        ui.add(egui::DragValue::new(&mut self.width));
                        ui.end_row();

                        ui.label(fl!(crate::LANGUAGE_LOADER, "new-file-height"));
                        ui.add(egui::DragValue::new(&mut self.height));
                        ui.end_row();
                    });
                ui.add_space(4.0);
            });

            modal.buttons(ui, |ui| {
                if ui
                    .button(fl!(crate::LANGUAGE_LOADER, "new-file-create"))
                    .clicked()
                {
                    self.create = true;
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
        self.create
    }

    fn commit_self(&self, window: &mut MainWindow) -> crate::TerminalResult<bool> {
        let buf = Buffer::create((self.width, self.height));
        let id = window.create_id();
        let editor = AnsiEditor::new(&window.gl, id, buf);

        add_child(&mut window.document_tree, None, Box::new(editor));
        Ok(true)
    }
}
