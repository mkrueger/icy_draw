use eframe::egui::{Ui, self, Layout};
use i18n_embed_fl::fl;
use icy_engine::SaveOptions;

pub fn create_settings_page(ui: &mut Ui, options: &mut SaveOptions)
{
    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
        ui.label("");
    });
    ui.add(egui::Checkbox::new(&mut options.save_sauce, fl!(crate::LANGUAGE_LOADER, "export-save-sauce-label")));
    ui.end_row();
}
