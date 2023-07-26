use std::path::PathBuf;

use eframe::egui::{self, Layout, TextEdit, Ui};
use egui_file::FileDialog;
use egui_modal::Modal;
use i18n_embed_fl::fl;
use icy_engine::SaveOptions;

use crate::{TerminalResult, ModalDialog};

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
    save_file_dialog: Option<FileDialog>,
    save_options: SaveOptions
}

impl ExportFileDialog {
    pub fn new(buf: &icy_engine::Buffer) -> Self {
        ExportFileDialog {
            should_commit: false,
            save_file_dialog: None,
            file_name: match &buf.file_name { 
                Some(path) => path.clone(),
                _ => PathBuf::from("Untitled.ans")
            },
            save_options: SaveOptions::new()
        }
    }
}

impl ModalDialog for ExportFileDialog {
    fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut result = false;

        if let Some(dialog) = &mut self.save_file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    self.file_name = file.to_path_buf();
                }
                self.save_file_dialog = None;
            }
            return false;
        }

        let modal = Modal::new(ctx, "my_modal");

        modal.show(|ui| {
            modal.title(ui, fl!(crate::LANGUAGE_LOADER, "export-title"));

            modal.frame(ui, |ui| {
                egui::Grid::new("some_unique_id")
                    .num_columns(2)
                    .spacing([4.0, 8.0])
                    .show(ui, |ui| {
                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(fl!(crate::LANGUAGE_LOADER, "export-file-label"));
                        });

                        let mut path_edit = self.file_name.to_str().unwrap().to_string();
                        let response = ui.add(
                        //    ui.available_size(),
                            TextEdit::singleline(&mut path_edit),
                        );
                        if response.changed() {
                            self.file_name = path_edit.into();
                        }

                        if ui.add(egui::Button::new("…").wrap(false)).clicked() {
                            let mut dialog = FileDialog::save_file(Some(self.file_name.clone()));
                            dialog.open();
                            self.save_file_dialog = Some(dialog);
                            ui.close_menu();
                        }

                        let mut format_type = 0;
                        if let Some(ext) = self.file_name.extension().unwrap().to_str() {
                            let ext = ext.to_lowercase();
                            for i in 0..TYPE_DESCRIPTIONS.len() {
                                let td = TYPE_DESCRIPTIONS[i];
                                if ext == td.2 {
                                    format_type = i;
                                    break;
                                }
                            }
                        }
                        let old_format = format_type;
                        
                        egui::ComboBox::from_id_source("format_combo")
                            .selected_text(TYPE_DESCRIPTIONS[format_type].0)
                            .show_ui(ui, |ui| {
                                for i in 0..TYPE_DESCRIPTIONS.len() {
                                    let td = TYPE_DESCRIPTIONS[i];
                                    ui.selectable_value(&mut format_type, i,  td.0);
                                }
                            }
                        );
                        if old_format != format_type {
                            println!("change extension to {}", format_type);
                            self.file_name.set_extension(TYPE_DESCRIPTIONS[format_type].2);
                        }
                        ui.end_row();

                        TYPE_DESCRIPTIONS[format_type].1(ui, &mut self.save_options);

                    });
                ui.add_space(4.0);
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

    fn should_commit(&self) -> bool { self.should_commit }

    fn commit(&self, editor: &mut crate::model::Editor) -> TerminalResult<bool> {
        editor.save_content(self.file_name.as_path(), &self.save_options)?;
        Ok(true)
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
    ("Tundra Draw (.tnd)", tundra_draw::create_settings_page, "tnd"),

    ("Bin (.bin)", bin::create_settings_page, "bin"),  
    ("XBin (.xb)", xbin::create_settings_page, "xb"),
];
