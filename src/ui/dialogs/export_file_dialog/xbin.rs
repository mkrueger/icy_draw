use eframe::egui::{Ui, self, Layout};
use i18n_embed_fl::fl;
use icy_engine::{SaveOptions, CompressionLevel};

pub fn create_settings_page(ui: &mut Ui, options: &mut SaveOptions)
{
    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
        ui.label(fl!(crate::LANGUAGE_LOADER, "export-compression-level-label"));
    });

    let label = match options.compression_level {
        CompressionLevel::Off => fl!(crate::LANGUAGE_LOADER, "export-compression-level-off"),
        CompressionLevel::Medium => fl!(crate::LANGUAGE_LOADER, "export-compression-level-medium"),
        CompressionLevel::High => fl!(crate::LANGUAGE_LOADER, "export-compression-level-high"),
    };
    
    egui::ComboBox::from_id_source("compr_level_combo")
    .selected_text(label)
    .show_ui(ui, |ui| {
        ui.selectable_value(&mut options.compression_level, CompressionLevel::Off,  fl!(crate::LANGUAGE_LOADER, "export-compression-level-off"));
        ui.selectable_value(&mut options.compression_level, CompressionLevel::Medium,  fl!(crate::LANGUAGE_LOADER, "export-compression-level-medium"));
        ui.selectable_value(&mut options.compression_level, CompressionLevel::High,  fl!(crate::LANGUAGE_LOADER, "export-compression-level-high"));
    });
    ui.end_row();


    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
        ui.label("");
    });
    ui.add(egui::Checkbox::new(&mut options.save_sauce, fl!(crate::LANGUAGE_LOADER, "export-save-sauce-label")));
    ui.end_row();
}
