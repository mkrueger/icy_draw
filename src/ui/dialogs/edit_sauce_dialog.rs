use eframe::egui::{self, Layout};
use egui_modal::Modal;
use i18n_embed_fl::fl;
use icy_engine::SauceString;

use crate::{ModalDialog, TerminalResult};

pub struct EditSauceDialog {
    pub should_commit: bool,
    pub title: SauceString<35, b' '>,
    pub author: SauceString<20, b' '>,
    pub group: SauceString<20, b' '>,
    pub comments: Vec<SauceString<64, 0>>,
}

impl EditSauceDialog {
    pub fn new(buf: &icy_engine::Buffer) -> Self {
        EditSauceDialog {
            should_commit: false,
            title: buf.title.clone(),
            author: buf.author.clone(),
            group: buf.group.clone(),
            comments: buf.comments.clone(),
        }
    }
}

impl ModalDialog for EditSauceDialog {
    fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut result = false;
        let modal: Modal = Modal::new(ctx, "my_modal");

        modal.show(|ui| {
            modal.title(ui, fl!(crate::LANGUAGE_LOADER, "edit-sauce-title"));

            modal.frame(ui, |ui| {
                egui::Grid::new("some_unique_id")
                    .num_columns(2)
                    .spacing([4.0, 8.0])
                    .show(ui, |ui| {
                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(fl!(crate::LANGUAGE_LOADER, "edit-sauce-title-label"));
                        });
                        ui.horizontal(|ui| {
                            let mut tmp_str = self.title.to_string();
                            ui.add(egui::TextEdit::singleline(&mut tmp_str).char_limit(35));
                            self.title = SauceString::from(&tmp_str);
                            ui.label(fl!(crate::LANGUAGE_LOADER, "edit-sauce-title-label-length"));
                        });
                        ui.end_row();

                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(fl!(crate::LANGUAGE_LOADER, "edit-sauce-author-label"));
                        });

                        ui.horizontal(|ui| {
                            let mut tmp_str = self.author.to_string();
                            ui.add(egui::TextEdit::singleline(&mut tmp_str).char_limit(20));
                            self.author = SauceString::from(&tmp_str);
                            ui.label(fl!(
                                crate::LANGUAGE_LOADER,
                                "edit-sauce-author-label-length"
                            ));
                        });
                        ui.end_row();

                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(fl!(crate::LANGUAGE_LOADER, "edit-sauce-group-label"));
                        });
                        ui.horizontal(|ui| {
                            let mut tmp_str = self.group.to_string();
                            ui.add(egui::TextEdit::singleline(&mut tmp_str).char_limit(20));
                            self.group = SauceString::from(&tmp_str);
                            ui.label(fl!(crate::LANGUAGE_LOADER, "edit-sauce-group-label-length"));
                        });
                        ui.end_row();
                    });
                ui.add_space(16.0);
                ui.label(fl!(crate::LANGUAGE_LOADER, "edit-sauce-comments-label"));
                ui.add_space(4.0);
                let mut tmp_str = String::new();
                for s in &self.comments {
                    tmp_str.push_str(&s.to_string());
                    tmp_str.push('\n');
                }
                self.comments.clear();
                egui::ScrollArea::vertical()
                    .max_height(180.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut tmp_str)
                                .desired_rows(6)
                                .desired_width(f32::INFINITY),
                        );
                    });

                let mut number = 0;
                for line in tmp_str.lines() {
                    self.comments.push(SauceString::from(line));
                    number += 1;
                    // limit to 255 chars which is the maximum for sauce comment lines.
                    if number > 255 {
                        break;
                    }
                }
            });

            modal.buttons(ui, |ui| {
                if ui
                    .button(fl!(crate::LANGUAGE_LOADER, "new-file-ok"))
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

    fn commit(&self, editor: &mut crate::model::Editor) -> TerminalResult<bool> {
        editor.buf.title = self.title.clone();
        editor.buf.author = self.author.clone();
        editor.buf.group = self.group.clone();
        editor.buf.comments = self.comments.clone();
        Ok(true)
    }
}
