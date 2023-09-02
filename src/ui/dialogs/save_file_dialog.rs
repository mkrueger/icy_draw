use eframe::egui;
use egui_file::FileDialog;
use icy_engine::SaveOptions;
use std::path::PathBuf;

use crate::MainWindow;

pub struct SaveFileDialog {
    open_file: bool,
    dialog: FileDialog,
    opened_file: Option<PathBuf>,
}

impl Default for SaveFileDialog {
    fn default() -> Self {
        let mut dialog = FileDialog::save_file(None);
        dialog.open();

        Self {
            open_file: false,
            dialog,
            opened_file: None,
        }
    }
}

impl crate::ModalDialog for SaveFileDialog {
    fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut result = false;

        if self.dialog.show(ctx).selected() {
            if let Some(file) = self.dialog.path() {
                self.opened_file = Some(file.to_path_buf());
                self.open_file = true;
            }
            result = true;
        }

        result
    }

    fn should_commit(&self) -> bool {
        self.open_file
    }

    fn commit_self(&self, window: &mut MainWindow) -> crate::TerminalResult<bool> {
        if let Some(file) = &self.opened_file.clone() {
            let file = file.with_extension("icd");
            if let Some(editor) = window.get_ansi_editor() {
                let options = SaveOptions::new();
                editor
                    .save_content(file.to_path_buf().as_path(), &options)
                    .unwrap();
                editor.set_file_name(file);
            }
        }
        Ok(true)
    }
}
