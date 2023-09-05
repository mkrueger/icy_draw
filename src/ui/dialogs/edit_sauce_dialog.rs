use eframe::egui::{self, Layout};
use egui_modal::Modal;
use i18n_embed_fl::fl;
use icy_engine::{SauceData, SauceString};

use crate::{AnsiEditor, Message, ModalDialog, TerminalResult};

pub struct EditSauceDialog {
    pub should_commit: bool,
    pub sauce_data: SauceData,
}

impl EditSauceDialog {
    pub fn new(buf: &icy_engine::Buffer) -> Self {
        EditSauceDialog {
            should_commit: false,
            sauce_data: buf.sauce_data.clone().unwrap_or_default(),
        }
    }
}

impl ModalDialog for EditSauceDialog {
    fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut result = false;
        let modal: Modal = Modal::new(ctx, "edit_sauce_dialog");

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
                            let mut tmp_str = self.sauce_data.title.to_string();
                            ui.add(egui::TextEdit::singleline(&mut tmp_str).char_limit(35));
                            self.sauce_data.title = SauceString::from(&tmp_str);
                            ui.label(fl!(crate::LANGUAGE_LOADER, "edit-sauce-title-label-length"));
                        });
                        ui.end_row();

                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(fl!(crate::LANGUAGE_LOADER, "edit-sauce-author-label"));
                        });

                        ui.horizontal(|ui| {
                            let mut tmp_str = self.sauce_data.author.to_string();
                            ui.add(egui::TextEdit::singleline(&mut tmp_str).char_limit(20));
                            self.sauce_data.author = SauceString::from(&tmp_str);
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
                            let mut tmp_str = self.sauce_data.group.to_string();
                            ui.add(egui::TextEdit::singleline(&mut tmp_str).char_limit(20));
                            self.sauce_data.group = SauceString::from(&tmp_str);
                            ui.label(fl!(crate::LANGUAGE_LOADER, "edit-sauce-group-label-length"));
                        });
                        ui.end_row();

                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(fl!(crate::LANGUAGE_LOADER, "edit-sauce-letter-spacing"));
                        });
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.sauce_data.use_letter_spacing, "");
                        });
                        ui.end_row();

                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(fl!(crate::LANGUAGE_LOADER, "edit-sauce-aspect-ratio"));
                        });
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.sauce_data.use_aspect_ratio, "");
                        });
                        ui.end_row();
                    });
                ui.add_space(16.0);
                ui.label(fl!(crate::LANGUAGE_LOADER, "edit-sauce-comments-label"));
                ui.add_space(4.0);
                let mut tmp_str = String::new();
                for s in &self.sauce_data.comments {
                    tmp_str.push_str(&s.to_string());
                    tmp_str.push('\n');
                }
                self.sauce_data.comments.clear();
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
                    self.sauce_data.comments.push(SauceString::from(line));
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

    fn commit(&self, editor: &mut AnsiEditor) -> TerminalResult<Option<Message>> {
        let bv = &mut editor.buffer_view.lock();
        let buf = bv.get_buffer_mut();
        buf.sauce_data = Some(self.sauce_data.clone());
        Ok(None)
    }
}
