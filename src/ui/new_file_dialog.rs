use eframe::egui::{self};
use i18n_embed_fl::fl;

pub struct NewFileDialog {
    pub width: i32,
    pub height: i32,

    pub create: bool,
}

impl NewFileDialog {
    pub fn new() -> Self {
        NewFileDialog {
            width: 80,
            height: 25,
            create: false,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut open = true;
        let mut create_file = true;
        egui::Window::new(fl!(crate::LANGUAGE_LOADER, "new-file-title"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(fl!(crate::LANGUAGE_LOADER, "new-file-width"));
                    let mut my_f32 = self.width as f32;
                    ui.add(egui::DragValue::new(&mut my_f32).speed(1));
                    self.width = my_f32 as i32;
                });
                ui.horizontal(|ui| {
                    ui.label(fl!(crate::LANGUAGE_LOADER, "new-file-height"));
                    let mut my_f32 = self.height as f32;
                    ui.add(egui::DragValue::new(&mut my_f32).speed(1));
                    self.height = my_f32 as i32;
                });
                if ui
                    .button(fl!(crate::LANGUAGE_LOADER, "new-file-ok"))
                    .clicked()
                {
                    self.create = true;
                    create_file = false;
                }
            });

        !(open && create_file)
    }
}
