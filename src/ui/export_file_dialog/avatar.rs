use eframe::egui::{Ui, self, Layout};
use i18n_embed_fl::fl;
use icy_engine::{SaveOptions, ScreenPreperation};

pub fn create_settings_page(ui: &mut Ui, options: &mut SaveOptions)
{
    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
        ui.label(fl!(crate::LANGUAGE_LOADER, "export-video-preparation-label"));
    });

    let label = match options.screen_preparation {
        ScreenPreperation::None => fl!(crate::LANGUAGE_LOADER, "export-video-preparation-None"),
        ScreenPreperation::ClearScreen => fl!(crate::LANGUAGE_LOADER, "export-video-preparation-Clear"),
        ScreenPreperation::Home => fl!(crate::LANGUAGE_LOADER, "export-video-preparation-Home"),
    };
    
    egui::ComboBox::from_id_source("screen_prep_combo")
    .selected_text(label)
    .show_ui(ui, |ui| {
        ui.selectable_value(&mut options.screen_preparation, ScreenPreperation::None,  fl!(crate::LANGUAGE_LOADER, "export-video-preparation-None"));
        ui.selectable_value(&mut options.screen_preparation, ScreenPreperation::ClearScreen,  fl!(crate::LANGUAGE_LOADER, "export-video-preparation-Clear"));
        ui.selectable_value(&mut options.screen_preparation, ScreenPreperation::Home,  fl!(crate::LANGUAGE_LOADER, "export-video-preparation-Home"));
    });
    ui.end_row();

    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
        ui.label("");
    });
    ui.add(egui::Checkbox::new(&mut options.save_sauce, fl!(crate::LANGUAGE_LOADER, "export-save-sauce-label")));
    ui.end_row();
}
