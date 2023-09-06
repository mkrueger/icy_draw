#![allow(clippy::needless_range_loop)]

use std::path::PathBuf;

use eframe::egui::{self, TextEdit, Ui};
use egui_modal::Modal;
use i18n_embed_fl::fl;
use icy_engine::SaveOptions;

use crate::{AnsiEditor, ModalDialog, TerminalResult};

mod ansi;
mod artworx;
mod ascii;
mod avatar;
mod bin;
mod ice_draw;
mod pcboard;
mod tundra_draw;
mod xbin;

pub struct ExportFileDialog {
    pub should_commit: bool,
    pub file_name: PathBuf,
    save_options: SaveOptions,
}

impl ExportFileDialog {
    pub fn new(buf: &icy_engine::Buffer) -> Self {
        ExportFileDialog {
            should_commit: false,
            file_name: match &buf.file_name {
                Some(path) => path.clone(),
                _ => PathBuf::from("Untitled.ans"),
            },
            save_options: SaveOptions::new(),
        }
    }
}

impl ModalDialog for ExportFileDialog {
    fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut result = false;

        let modal = Modal::new(ctx, "export_file-dialog");
        modal.show(|ui| {
            modal.title(ui, fl!(crate::LANGUAGE_LOADER, "export-title"));

            modal.frame(ui, |ui| {
                let mut format_type = 0;

                ui.horizontal(|ui| {
                    ui.label(fl!(crate::LANGUAGE_LOADER, "export-file-label"));

                    let mut path_edit = self.file_name.to_str().unwrap().to_string();
                    let response = ui.add(
                        //    ui.available_size(),
                        TextEdit::singleline(&mut path_edit),
                    );
                    if response.changed() {
                        self.file_name = path_edit.into();
                    }

                    if ui.add(egui::Button::new("…").wrap(false)).clicked() {
                        let mut dialog = rfd::FileDialog::new();
                        if let Some(parent) = self.file_name.parent() {
                            dialog = dialog.set_directory(parent);
                        }
                        let res = dialog.pick_file();

                        if let Some(file) = res {
                            self.file_name = file;
                        }
                        ui.close_menu();
                    }
                    if let Some(ext) = self.file_name.extension() {
                        if let Some(ext) = ext.to_str() {
                            let ext = ext.to_lowercase();
                            for i in 0..TYPE_DESCRIPTIONS.len() {
                                let td = TYPE_DESCRIPTIONS[i];
                                if ext == td.2 {
                                    format_type = i;
                                    break;
                                }
                            }
                        }
                    }
                    let old_format = format_type;
                    // ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {

                    egui::ComboBox::from_id_source("format_combo")
                        .selected_text(TYPE_DESCRIPTIONS[format_type].0)
                        .width(190.)
                        .show_ui(ui, |ui| {
                            (0..TYPE_DESCRIPTIONS.len()).for_each(|i| {
                                let td = TYPE_DESCRIPTIONS[i];
                                ui.selectable_value(&mut format_type, i, td.0);
                            });
                        });
                    if old_format != format_type {
                        self.file_name
                            .set_extension(TYPE_DESCRIPTIONS[format_type].2);
                    }
                    //    });
                });

                TYPE_DESCRIPTIONS[format_type].1(ui, &mut self.save_options);
            });

            modal.buttons(ui, |ui| {
                if ui
                    .button(fl!(crate::LANGUAGE_LOADER, "export-button-title"))
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

    fn commit(&self, editor: &mut AnsiEditor) -> TerminalResult<Option<crate::Message>> {
        editor.save_content(self.file_name.as_path(), &self.save_options)?;
        Ok(None)
    }
}

type CreateSettingsFunction = fn(&mut Ui, &mut SaveOptions);

const TYPE_DESCRIPTIONS: [(&str, CreateSettingsFunction, &str); 9] = [
    ("Ansi (.ans)", ansi::create_settings_page, "ans"),
    ("Avatar (.avt)", avatar::create_settings_page, "avt"),
    ("PCBoard (.pcb)", pcboard::create_settings_page, "pcb"),
    ("Ascii (.asc)", ascii::create_settings_page, "asc"),
    ("Artworx (.adf)", artworx::create_settings_page, "adf"),
    ("Ice Draw (.idf)", ice_draw::create_settings_page, "idf"),
    (
        "Tundra Draw (.tnd)",
        tundra_draw::create_settings_page,
        "tnd",
    ),
    ("Bin (.bin)", bin::create_settings_page, "bin"),
    ("XBin (.xb)", xbin::create_settings_page, "xb"),
];
