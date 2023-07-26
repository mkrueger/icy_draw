use eframe::egui::{self, Layout};
use egui_modal::Modal;
use i18n_embed_fl::fl;
use icy_engine::Size;

pub struct SetCanvasSizeDialog {
    pub ok: bool,
    pub width: i32,
    pub height: i32
}

impl SetCanvasSizeDialog {
    pub fn new(buf: &icy_engine::Buffer) -> Self {
        SetCanvasSizeDialog {
            ok: false,
            width: buf.get_buffer_width(),
            height: buf.get_real_buffer_height()
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> bool {
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
                    self.ok = true;
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

    pub fn set_result(&self, editor: &mut crate::model::Editor) {
        editor.buf.set_buffer_size(Size::new(self.width, self.height));
    }
}
