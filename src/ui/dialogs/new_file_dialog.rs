use eframe::egui::{self};
use egui_modal::Modal;
use i18n_embed_fl::fl;

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

impl NewFileDialog {
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut result = false;
        let modal = Modal::new(ctx, "my_modal");

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
}
