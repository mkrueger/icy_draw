use eframe::{
    egui::{self, color_picker, Layout, RichText, Modifiers},
    epaint::Vec2,
};
use i18n_embed_fl::fl;
use icy_engine_egui::{show_monitor_settings, MarkerSettings, MonitorSettings};

use crate::{SelectOutlineDialog, SETTINGS, Commands};

#[derive(Default)]
pub struct SettingsDialog {
    settings_category: usize,
    select_outline_dialog: SelectOutlineDialog,

    monitor_settings: MonitorSettings,
    marker_settings: MarkerSettings,
    key_filter: String,
    key_bindings: Vec<(String, eframe::egui::Key, Modifiers)>,

}
const MONITOR_CAT: usize = 0 ;
const MARKER_CAT: usize = 1 ;
const OUTLINE_CAT: usize = 2;
const KEYBIND_CAT: usize = 3 ;

impl SettingsDialog {

    pub(crate) fn init(&mut self) {
        self.monitor_settings = unsafe { SETTINGS.monitor_settings.clone() };
        self.marker_settings = unsafe { SETTINGS.marker_settings.clone() };
        self.key_bindings = unsafe { SETTINGS.key_bindings.clone() };
    }
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut open = true;
        let mut dialog_open = true;
        let title = RichText::new(fl!(crate::LANGUAGE_LOADER, "settings-heading"));
        if ctx.input(|i| i.key_down(egui::Key::Escape)) {
            open = false;
        }

        egui::Window::new(title)
            .open(&mut open)
            .collapsible(false)
            .fixed_size(Vec2::new(400., 300.))
            .resizable(false)
            .frame(egui::Frame::window(&ctx.style()))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    egui::widgets::global_dark_light_mode_switch(ui);
                    let settings_category = self.settings_category;

                    if ui
                        .selectable_label(
                            settings_category == MONITOR_CAT,
                            fl!(crate::LANGUAGE_LOADER, "settings-monitor-category"),
                        )
                        .clicked()
                    {
                        self.settings_category = MONITOR_CAT;
                    }

                    if ui
                        .selectable_label(
                            settings_category == MARKER_CAT,
                            fl!(crate::LANGUAGE_LOADER, "settings-markers-guides-category"),
                        )
                        .clicked()
                    {
                        self.settings_category = MARKER_CAT;
                    }

                    if ui
                        .selectable_label(
                            settings_category == OUTLINE_CAT,
                            fl!(crate::LANGUAGE_LOADER, "settings-font-outline-category"),
                        )
                        .clicked()
                    {
                        self.settings_category = OUTLINE_CAT;
                    }


                    if ui
                        .selectable_label(
                            settings_category == KEYBIND_CAT,
                            fl!(crate::LANGUAGE_LOADER, "settings-keybindings-category"),
                        )
                        .clicked()
                    {
                        self.settings_category = KEYBIND_CAT;
                    }
                });
                ui.separator();
                match self.settings_category {
                    MONITOR_CAT => unsafe {
                        if let Some(new_settings) =
                            show_monitor_settings(ui, &SETTINGS.monitor_settings)
                        {
                            SETTINGS.monitor_settings = new_settings;
                        }
                    },
                    MARKER_CAT => {
                        ui.add_space(8.0);
                        unsafe {
                            if let Some(new_settings) =
                                show_marker_settings(ui, &SETTINGS.marker_settings)
                            {
                                SETTINGS.marker_settings = new_settings;
                            }
                        }
                    }

                    OUTLINE_CAT => {
                        ui.add_space(8.0);
                        self.select_outline_dialog
                            .show_outline_ui(ui, 4, Vec2::new(8.0, 8.0));
                    }

                    KEYBIND_CAT => {
                        let mut map = std::collections::HashMap::new();
                        for (s, key, modifier) in &self.key_bindings {
                            map.insert(s.clone(), (*key, *modifier));
                        }
                        crate::Commands::show_keybinds_settings(ui, &mut self.key_filter, &mut map);
                        self.key_bindings.clear();
                        for (s, (key, modifier)) in map {
                            self.key_bindings.push((s, key, modifier));
                        }
                    },
                    _ => {}
                }

                ui.separator();
                ui.add_space(4.0);
                ui.with_layout(Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui
                        .button(fl!(crate::LANGUAGE_LOADER, "new-file-ok"))
                        .clicked()
                    {
                        unsafe {
                            SETTINGS.key_bindings = self.key_bindings.clone();
                        }
                        dialog_open = false;
                    }

                    if ui
                    .button(fl!(crate::LANGUAGE_LOADER, "new-file-cancel"))
                    .clicked()
                    {
                        unsafe {
                            SETTINGS.monitor_settings = self.monitor_settings.clone();
                            SETTINGS.marker_settings = self.marker_settings.clone();
                            SETTINGS.key_bindings = self.key_bindings.clone();
                        }
                        dialog_open = false;
                    }


                    if (self.settings_category == MONITOR_CAT || self.settings_category == 1|| self.settings_category == 3)
                        && ui
                            .button(fl!(crate::LANGUAGE_LOADER, "settings-reset_button"))
                            .clicked()
                    {
                        unsafe {
                            match self.settings_category {
                                MONITOR_CAT => SETTINGS.monitor_settings = Default::default(),
                                MARKER_CAT => SETTINGS.marker_settings = Default::default(),
                                KEYBIND_CAT => {
                                    self.key_bindings = Commands::default_keybindings();
                                },
                                _ => {}
                            }
                        }
                    }
                });
            });

        open && dialog_open
    }

}

pub fn show_marker_settings(
    ui: &mut egui::Ui,
    old_settings: &MarkerSettings,
) -> Option<MarkerSettings> {
    let mut result = None;

    let mut marker_settings = old_settings.clone();

    ui.horizontal(|ui| {
        ui.label(fl!(
            crate::LANGUAGE_LOADER,
            "settings-background_color-label"
        ));
        color_picker::color_edit_button_srgba(
            ui,
            &mut marker_settings.border_color,
            color_picker::Alpha::Opaque,
        );
    });

    ui.add(
        egui::Slider::new(&mut marker_settings.reference_image_alpha, 0.1..=0.9)
            .text(fl!(crate::LANGUAGE_LOADER, "settings-reference-alpha")),
    );

    ui.horizontal(|ui| {
        ui.label(fl!(crate::LANGUAGE_LOADER, "settings-raster-label"));

        color_picker::color_edit_button_srgba(
            ui,
            &mut marker_settings.raster_color,
            color_picker::Alpha::Opaque,
        );
        ui.add(
            egui::Slider::new(&mut marker_settings.raster_alpha, 0.1..=0.9)
                .text(fl!(crate::LANGUAGE_LOADER, "settings-alpha")),
        );
    });

    ui.horizontal(|ui| {
        ui.label(fl!(crate::LANGUAGE_LOADER, "settings-guide-label"));
        color_picker::color_edit_button_srgba(
            ui,
            &mut marker_settings.guide_color,
            color_picker::Alpha::Opaque,
        );

        ui.add(
            egui::Slider::new(&mut marker_settings.guide_alpha, 0.1..=0.9)
                .text(fl!(crate::LANGUAGE_LOADER, "settings-alpha")),
        );
    });

    if marker_settings != *old_settings {
        result = Some(marker_settings);
    }

    result
}
