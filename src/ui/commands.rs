use eframe::egui::{self, Modifiers};
use egui_bind::{BindTarget, KeyOrPointer};
use i18n_embed_fl::fl;

use crate::{button_with_shortcut, Message};

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

    pub const ALT: Modifiers = Modifiers {
        alt: true,
        ctrl: false,
        shift: false,
        mac_cmd: false,
        command: false,
    };

    pub const ALT_CTRL: Modifiers = Modifiers {
        alt: true,
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
    () => {
        None
    };
    ($key:ident, $modifier: ident) => {
        Some((KeyOrPointer::Key(egui::Key::$key), modifier_keys::$modifier))
    };
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

impl Command {
    pub fn new(
        key: Option<(KeyOrPointer, Modifiers)>,
        message: Message,
        description: String,
    ) -> Self {
        Self {
            key,
            message,
            description,
        }
    }

    pub fn is_pressed(&self, ctx: &egui::Context) -> bool {
        self.key.pressed(ctx)
    }

    pub fn ui_enabled(&self, ui: &mut egui::Ui, enabled: bool, message: &mut Option<Message>) {
        let response = ui.with_layout(ui.layout().with_cross_justify(true), |ui| {
            ui.set_enabled(enabled);

            if let Some((KeyOrPointer::Key(k), modifier)) = self.key {
                let mut shortcut = k.name().to_string();

                if modifier.ctrl {
                    shortcut.insert_str(0, "Ctrl+");
                }

                if modifier.alt {
                    shortcut.insert_str(0, "Alt+");
                }

                if modifier.shift {
                    shortcut.insert_str(0, "Shift+");
                }

                button_with_shortcut(ui, true, &self.description, shortcut)
            } else {
                ui.add(egui::Button::new(&self.description).wrap(false))
            }
        });

        if response.inner.clicked() {
            *message = Some(self.message.clone());
            ui.close_menu();
        }
    }

    pub fn ui(&self, ui: &mut egui::Ui, message: &mut Option<Message>) {
        self.ui_enabled(ui, true, message)
    }
}

keys![
    (new_file, "menu-new", NewFile, N, CTRL),
    (save, "menu-save", SaveFile, S, CTRL),
    (save_as, "menu-save-as", SaveFileAs, S, CTRL_SHIFT),
    (open_file, "menu-open", OpenFile, O, CTRL),
    (export, "menu-export", ExportFile),
    (
        edit_font_outline,
        "menu-edit-font-outline",
        ShowOutlineDialog
    ),
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
    (
        justify_line_center,
        "menu-justify_line_center",
        CenterLine,
        C,
        ALT
    ),
    (
        justify_line_left,
        "menu-justify_line_left",
        JustifyLineLeft,
        L,
        ALT
    ),
    (
        justify_line_right,
        "menu-justify_line_right",
        JustifyLineRight,
        R,
        ALT
    ),
    (insert_row, "menu-insert_row", InsertRow, ArrowUp, CTRL),
    (delete_row, "menu-delete_row", DeleteRow, ArrowDown, CTRL),
    (
        insert_column,
        "menu-insert_colum",
        InsertColumn,
        ArrowRight,
        CTRL
    ),
    (
        delete_column,
        "menu-delete_colum",
        DeleteColumn,
        ArrowLeft,
        CTRL
    ),
    (erase_row, "menu-erase_row", EraseRow, E, ALT),
    (
        erase_row_to_start,
        "menu-erase_row_to_start",
        EraseRowToStart,
        Home,
        ALT
    ),
    (
        erase_row_to_end,
        "menu-erase_row_to_end",
        EraseRowToEnd,
        End,
        ALT
    ),
    (erase_column, "menu-erase_column", EraseColumn, E, ALT),
    (
        erase_column_to_start,
        "menu-erase_column_to_start",
        EraseColumnToStart,
        Home,
        ALT
    ),
    (
        erase_column_to_end,
        "menu-erase_column_to_end",
        EraseColumnToEnd,
        End,
        ALT
    ),
    (
        scroll_area_up,
        "menu-scroll_area_up",
        ScrollAreaUp,
        ArrowUp,
        ALT_CTRL
    ),
    (
        scroll_area_down,
        "menu-scroll_area_down",
        ScrollAreaDown,
        ArrowDown,
        ALT_CTRL
    ),
    (
        scroll_area_left,
        "menu-scroll_area_left",
        ScrollAreaLeft,
        ArrowLeft,
        ALT_CTRL
    ),
    (
        scroll_area_right,
        "menu-scroll_area_right",
        ScrollAreaRight,
        ArrowRight,
        ALT_CTRL
    ),
    (
        set_reference_image,
        "menu-reference-image",
        SetReferenceImage,
        O,
        CTRL_SHIFT
    ),
    (
        toggle_reference_image,
        "menu-toggle-reference-image",
        ToggleReferenceImage,
        Tab,
        CTRL
    ),
    (
        clear_reference_image,
        "menu-clear-reference-image",
        ClearReferenceImage
    ),
    (
        pick_attribute_under_caret,
        "menu-pick_attribute_under_caret",
        PickAttributeUnderCaret,
        U,
        ALT
    ),
    (
        switch_to_default_color,
        "menu-default_color",
        SwitchToDefaultColor,
        D,
        CTRL
    ),
    (toggle_color, "menu-toggle_color", ToggleColor, X, ALT),
    (
        fullscreen,
        "menu-toggle_fullscreen",
        ToggleFullScreen,
        F11,
        NONE
    ),
    (zoom_reset, "menu-zoom_reset", ZoomReset, Num0, CTRL),
    (zoom_in, "menu-zoom_in", ZoomIn, PlusEquals, CTRL),
    (zoom_out, "menu-zoom_out", ZoomOut, Minus, CTRL),
];
