use std::sync::Arc;

use eframe::{
    egui::{self, color_picker, Layout, Modifiers, RichText},
    epaint::{mutex::Mutex, Color32, Vec2},
};
use i18n_embed_fl::fl;
use icy_engine::{AttributedChar, Buffer, Color, Size, TextAttribute};
use icy_engine_egui::{
    show_monitor_settings, show_terminal_area, BufferView, MarkerSettings, MonitorSettings,
};

use crate::{CharTableToolWindow, CharacterSet, Commands, SelectOutlineDialog, SETTINGS};
pub struct SettingsDialog {
    settings_category: usize,
    select_outline_dialog: SelectOutlineDialog,

    monitor_settings: MonitorSettings,
    marker_settings: MarkerSettings,
    key_filter: String,
    key_bindings: Vec<(String, eframe::egui::Key, Modifiers)>,
    char_set: CharacterSet,
    views: Vec<Arc<Mutex<BufferView>>>,
    char_view: CharTableToolWindow,
}
const MONITOR_CAT: usize = 0;
const MARKER_CAT: usize = 1;
const OUTLINE_CAT: usize = 2;
const CHAR_SET_CAT: usize = 3;
const KEYBIND_CAT: usize = 4;

impl SettingsDialog {
    pub fn new(gl: &Arc<glow::Context>) -> Self {
        let mut views = Vec::new();

        for _ in 0..15 {
            let mut buffer = Buffer::new(Size::new(10, 1));
            buffer.is_terminal_buffer = true;
            let mut buffer_view = BufferView::from_buffer(gl, buffer);
            buffer_view.interactive = true;
            views.push(Arc::new(Mutex::new(buffer_view)));
        }
        let char_view = CharTableToolWindow::new(32);
        Self {
            settings_category: MONITOR_CAT,
            select_outline_dialog: SelectOutlineDialog::default(),
            monitor_settings: Default::default(),
            marker_settings: Default::default(),
            key_filter: String::new(),
            key_bindings: Commands::default_keybindings(),
            char_set: Default::default(),
            views,
            char_view,
        }
    }

    pub(crate) fn init(&mut self) {
        self.monitor_settings = unsafe { SETTINGS.monitor_settings.clone() };
        self.marker_settings = unsafe { SETTINGS.marker_settings.clone() };
        self.key_bindings = unsafe { SETTINGS.key_bindings.clone() };
        self.char_set = unsafe { SETTINGS.character_sets[0].clone() };
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
                            settings_category == CHAR_SET_CAT,
                            fl!(crate::LANGUAGE_LOADER, "settings-char-set-category"),
                        )
                        .clicked()
                    {
                        self.settings_category = CHAR_SET_CAT;
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

                    CHAR_SET_CAT => {
                        ui.add_space(8.0);
                        self.show_charset_editor(ui);
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
                    }
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
                            SETTINGS.character_sets.clear();
                            SETTINGS.character_sets.push(self.char_set.clone());
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

                    if (self.settings_category == MONITOR_CAT
                        || self.settings_category == MARKER_CAT
                        || self.settings_category == CHAR_SET_CAT
                        || self.settings_category == KEYBIND_CAT)
                        && ui
                            .button(fl!(crate::LANGUAGE_LOADER, "settings-reset_button"))
                            .clicked()
                    {
                        unsafe {
                            match self.settings_category {
                                MONITOR_CAT => SETTINGS.monitor_settings = Default::default(),
                                MARKER_CAT => SETTINGS.marker_settings = Default::default(),
                                CHAR_SET_CAT => self.char_set = Default::default(),
                                KEYBIND_CAT => {
                                    self.key_bindings = Commands::default_keybindings();
                                }
                                _ => {}
                            }
                        }
                    }
                });
            });

        open && dialog_open
    }

    pub fn show_charset_editor(&mut self, ui: &mut egui::Ui) {
        ui.set_height(540.);
        let mut id = 0;
        ui.add_space(48.0);
        egui::Grid::new("paste_mode_grid")
            .num_columns(6)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                for view in &self.views {
                    let opt = icy_engine_egui::TerminalOptions {
                        stick_to_bottom: false,
                        scale: Some(Vec2::new(2.0, 2.0)),
                        id: Some(egui::Id::new(200 + id)),
                        terminal_size: Some(Vec2::new(8. * 10. * 2.0, 16.0 * 2.0)),
                        ..Default::default()
                    };

                    for x in 0..10 {
                        let ch = self.char_set.table[id][x];
                        view.lock().get_buffer_mut().layers[0]
                            .set_char((x, 0), AttributedChar::new(ch, TextAttribute::default()));
                    }
                    if id % 3 == 0 {
                        ui.end_row();
                    }
                    id += 1;

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if view.lock().calc.has_focus {
                            ui.strong(fl!(crate::LANGUAGE_LOADER, "settings-set-label", set = id));
                        } else {
                            ui.label(fl!(crate::LANGUAGE_LOADER, "settings-set-label", set = id));
                        }
                    });
                    show_terminal_area(ui, view.clone(), opt);
                }
            });
        self.char_view.show_plain_char_table(ui);
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
        let (r, g, b) = marker_settings.border_color.get_rgb();
        let mut color = Color32::from_rgb(r, g, b);
        color_picker::color_edit_button_srgba(ui, &mut color, color_picker::Alpha::Opaque);
        marker_settings.border_color = Color::new(r, g, b);
    });

    ui.add(
        egui::Slider::new(&mut marker_settings.reference_image_alpha, 0.1..=0.9)
            .text(fl!(crate::LANGUAGE_LOADER, "settings-reference-alpha")),
    );

    ui.horizontal(|ui| {
        ui.label(fl!(crate::LANGUAGE_LOADER, "settings-raster-label"));
        let (r, g, b) = marker_settings.raster_color.get_rgb();
        let mut color = Color32::from_rgb(r, g, b);

        color_picker::color_edit_button_srgba(ui, &mut color, color_picker::Alpha::Opaque);
        marker_settings.raster_color = Color::new(r, g, b);
        ui.add(
            egui::Slider::new(&mut marker_settings.raster_alpha, 0.1..=0.9)
                .text(fl!(crate::LANGUAGE_LOADER, "settings-alpha")),
        );
    });

    ui.horizontal(|ui| {
        ui.label(fl!(crate::LANGUAGE_LOADER, "settings-guide-label"));
        let (r, g, b) = marker_settings.guide_color.get_rgb();
        let mut color = Color32::from_rgb(r, g, b);

        color_picker::color_edit_button_srgba(ui, &mut color, color_picker::Alpha::Opaque);
        marker_settings.guide_color = Color::new(r, g, b);

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
