use eframe::egui::{Modifiers, self};
use egui_bind::{KeyOrPointer, BindTarget};
use i18n_embed_fl::fl;


use crate::Message;

pub struct Command {
    key: Option<(KeyOrPointer, Modifiers)>,
    message: Message,
    description: String,
}

mod modifier_keys {
    use eframe::egui::Modifiers;

    pub const NONE: Modifiers = Modifiers {
        alt: false,
        ctrl: false,
        shift: false,
        mac_cmd: false,
        command: false,
    };

    pub const CTRL: Modifiers = Modifiers {
        alt: false,
        ctrl: true,
        shift: false,
        mac_cmd: false,
        command: false,
    };

    pub const CTRL_SHIFT: Modifiers = Modifiers {
        alt: false,
        ctrl: true,
        shift: true,
        mac_cmd: false,
        command: false,
    };
    
}


macro_rules! key {
    () => { None };
    ($key:ident, $modifier: ident) => {
        Some((KeyOrPointer::Key(egui::Key::$key), modifier_keys::$modifier))
    }
}

macro_rules! keys {
    ($( ($l:ident, $translation: expr, $message:ident$(, $key:ident, $modifier: ident)? ) ),* $(,)? ) => {
        pub struct Commands {
            $(
                pub $l: Command,
            )*
        }

        impl Default for Commands {
            fn default() -> Self {
                Self {
                    $(
                        $l: Command::new(key!($($key, $modifier)?), Message::$message, fl!(crate::LANGUAGE_LOADER, $translation)),
                    )*
                }
            }
        }

        impl Commands {
            pub fn check(&self, ctx: &egui::Context, message: &mut Option<Message>) {
                $(
                    if self.$l.is_pressed(ctx) {
                        *message = Some(self.$l.message.clone());
                        return;
                    }
                )*
            } 
        }
    };
}

keys![
    (new_file, "menu-new", NewFile, N, CTRL),
    (save, "menu-save", SaveFile, S, CTRL),
    (save_as, "menu-save-as", SaveFileAs, S, CTRL_SHIFT),
    (open_file, "menu-open", OpenFile, O, CTRL),
    (export, "menu-export", ExportFile),
    (edit_font_outline, "menu-edit-font-outline", ShowOutlineDialog),
    (close_window, "menu-close", CloseWindow, Q, CTRL),
    (undo, "menu-undo", Undo, Z, CTRL),
    (redo, "menu-redo", Redo, Z, CTRL_SHIFT),
    (cut, "menu-cut", Cut, X, CTRL),
    (copy, "menu-copy", Copy, C, CTRL),
    (paste, "menu-paste", Paste, V, CTRL),
    (select_all, "menu-select-all", SelectAll, A, CTRL),
    (deselect, "menu-deselect", Deselect, Escape, NONE),
    (erase_selection, "menu-erase", DeleteSelection, Delete, NONE),

    (flip_x, "menu-flipx", FlipX),
    (flip_y, "menu-flipy", FlipY),
    (justifycenter, "menu-justifycenter", Center),
    (justifyleft, "menu-justifyleft", JustifyLeft),
    (justifyright, "menu-justifyright", JustifyRight),
    (crop, "menu-crop", Crop),
    (about, "menu-about", ShowAboutDialog),
];



impl Command {
    pub fn new(key: Option<(KeyOrPointer, Modifiers)>, message: Message, description: String) -> Self {
        Self {
            key,
            message,
            description,
        }
    }

    pub fn is_pressed(&self, ctx: &egui::Context) -> bool {
        self.key.pressed(ctx)
    }

    pub fn ui_enabled(&self, ui: &mut egui::Ui, enabled: bool, message: &mut Option<Message>)  {
        if ui
            .add_enabled(
                enabled,
                egui::Button::new(&self.description).wrap(false),
            )
            .clicked()
        {
        *message = Some(self.message.clone());
            ui.close_menu();
        }
    }

    pub fn ui(&self, ui: &mut egui::Ui, message: &mut Option<Message>)  {
        if ui
            .add(
                egui::Button::new(&self.description).wrap(false),
            )
            .clicked()
        {
        *message = Some(self.message.clone());
            ui.close_menu();
        }
    }
/*

                let button = button_with_shortcut(
                    ui,
                    true,
                    fl!(crate::LANGUAGE_LOADER, "menu-close"),
                    "Ctrl+Q",
                );
                if button.clicked() {
                    frame.close();
                    ui.close_menu();
                }
*/
    
}