#![allow(clippy::needless_range_loop)]

use std::path::PathBuf;

use eframe::egui::{self, TextEdit, Ui};
use egui_file::FileDialog;
use egui_modal::Modal;
use i18n_embed_fl::fl;
use icy_engine::{Rectangle, SaveOptions, TextPane};

use crate::{AnsiEditor, Message, ModalDialog, TerminalResult};

mod ansi;
mod artworx;
mod ascii;
mod avatar;
mod bin;
mod ice_draw;
mod pcboard;
mod png;
mod tundra_draw;
mod xbin;

pub struct ExportFileDialog {
    pub should_commit: bool,
    pub file_name: PathBuf,
    save_options: SaveOptions,
    folder_dialog: Option<FileDialog>,
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
            folder_dialog: None,
        }
    }
}

impl ModalDialog for ExportFileDialog {
    fn show(&mut self, ctx: &egui::Context) -> bool {
        if let Some(ed) = &mut self.folder_dialog {
            if ed.show(ctx).selected() {
                if let Some(res) = ed.path() {
                    self.file_name = res.to_path_buf();
                }
                self.folder_dialog = None
            } else {
                return false;
            }
        }

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
                        let mut initial_path = None;
                        crate::set_default_initial_directory_opt(&mut initial_path);
                        let mut dialog = FileDialog::save_file(initial_path);
                        dialog.open();
                        self.folder_dialog = Some(dialog);

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
                    self.file_name.set_extension(TYPE_DESCRIPTIONS[format_type].2);
                    //    });
                });

                TYPE_DESCRIPTIONS[format_type].1(ui, &mut self.save_options);
            });

            modal.buttons(ui, |ui| {
                if ui.button(fl!(crate::LANGUAGE_LOADER, "export-button-title")).clicked() {
                    self.should_commit = true;
                    result = true;
                }
                if ui.button(fl!(crate::LANGUAGE_LOADER, "new-file-cancel")).clicked() {
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
        if let Some(ext) = self.file_name.extension() {
            if let Some(ext) = ext.to_str() {
                let ext = ext.to_lowercase();
                if ext == "png" {
                    let lock = &editor.buffer_view.lock();
                    let buf = lock.get_buffer();
                    let (size, pixels) = buf.render_to_rgba(Rectangle::from(0, 0, buf.get_width(), buf.get_height()));
                    let image_buffer = image::RgbaImage::from_raw(size.width as u32, size.height as u32, pixels);
                    match image_buffer {
                        Some(img) => {
                            if let Err(err) = img.save(&self.file_name) {
                                return Ok(Some(Message::ShowError(format!("Failed to save image: {}", err))));
                            }
                        }
                        None => {
                            return Ok(Some(Message::ShowError("Failed to save image".to_string())));
                        }
                    }

                    return Ok(None);
                }
            }
        }

        editor.save_content(self.file_name.as_path(), &self.save_options)?;
        Ok(None)
    }
}

type CreateSettingsFunction = fn(&mut Ui, &mut SaveOptions);

const TYPE_DESCRIPTIONS: [(&str, CreateSettingsFunction, &str); 11] = [
    ("Ansi (.ans)", ansi::create_settings_page, "ans"),
    ("Avatar (.avt)", avatar::create_settings_page, "avt"),
    ("PCBoard (.pcb)", pcboard::create_settings_page, "pcb"),
    ("Ascii (.asc)", ascii::create_settings_page, "asc"),
    ("Artworx (.adf)", artworx::create_settings_page, "adf"),
    ("Ice Draw (.idf)", ice_draw::create_settings_page, "idf"),
    ("Tundra Draw (.tnd)", tundra_draw::create_settings_page, "tnd"),
    ("Bin (.bin)", bin::create_settings_page, "bin"),
    ("XBin (.xb)", xbin::create_settings_page, "xb"),
    ("CtrlA (.msg)", pcboard::create_settings_page, "msg"),
    ("PNG (.png)", png::create_settings_page, "png"),
];
