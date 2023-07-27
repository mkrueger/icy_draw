use std::{rc::Rc, cell::RefCell};

use eframe::egui::{self};
use egui_modal::Modal;
use i18n_embed_fl::fl;

use crate::{TerminalResult, ModalDialog};

pub struct SelectCharacterDialog {
    pub should_commit: bool,
    ch: Rc<RefCell<char>>,
    selected_ch: char
}

impl SelectCharacterDialog {
    pub fn new(ch: Rc<RefCell<char>>) -> Self {
        let selected_ch = *ch.borrow();
        SelectCharacterDialog {
            should_commit: false,
            ch,
            selected_ch
        }
    }
}

impl ModalDialog for SelectCharacterDialog {
    fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut result = false;
        let modal = Modal::new(ctx, "my_modal");

        modal.show(|ui| {
            modal.title(ui, fl!(crate::LANGUAGE_LOADER, "select-character-title"));

            modal.frame(ui, |ui| {
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

    fn should_commit(&self) -> bool { self.should_commit }

    fn commit(&self, editor: &mut crate::model::Editor) -> TerminalResult<bool>  {
        self.ch.swap(&RefCell::new(self.selected_ch));
        Ok(true)
    }
}