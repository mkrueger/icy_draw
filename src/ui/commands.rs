use std::collections::HashMap;

use eframe::egui::{self, Modifiers};
use egui_bind::{BindTarget, KeyOrPointer};
use i18n_embed_fl::fl;

use crate::{button_with_shortcut, DocumentTab, Message, SETTINGS};

pub trait CommandState {
    fn is_enabled(&self, _open_tab_opt: Option<&DocumentTab>) -> bool {
        true
    }
    fn is_checked(&self, _open_tab_opt: Option<&DocumentTab>) -> Option<bool> {
        None
    }
}

#[derive(Default)]
pub struct AlwaysEnabledState {}
impl CommandState for AlwaysEnabledState {}

#[derive(Default)]
pub struct BufferOpenState {}

impl CommandState for BufferOpenState {
    fn is_enabled(&self, open_tab_opt: Option<&DocumentTab>) -> bool {
        if let Some(pane) = open_tab_opt {
            if let Ok(doc) = pane.doc.lock() {
                return doc.get_ansi_editor().is_some();
            }
        }
        false
    }
}

#[derive(Default)]
pub struct LayerBordersState {}

impl CommandState for LayerBordersState {
    fn is_enabled(&self, _open_tab_opt: Option<&DocumentTab>) -> bool {
        true
    }

    fn is_checked(&self, _open_tab_opt: Option<&DocumentTab>) -> Option<bool> {
        unsafe { Some(SETTINGS.show_layer_borders) }
    }
}

#[derive(Default)]
pub struct LineNumberState {}

impl CommandState for LineNumberState {
    fn is_enabled(&self, _open_tab_opt: Option<&DocumentTab>) -> bool {
        true
    }

    fn is_checked(&self, _open_tab_opt: Option<&DocumentTab>) -> Option<bool> {
        unsafe { Some(SETTINGS.show_line_numbers) }
    }
}

#[derive(Default)]
pub struct FileOpenState {}

impl CommandState for FileOpenState {
    fn is_enabled(&self, open_tab_opt: Option<&DocumentTab>) -> bool {
        open_tab_opt.is_some()
    }
}

#[derive(Default)]
pub struct FileIsDirtyState {}

impl CommandState for FileIsDirtyState {
    fn is_enabled(&self, open_tab_opt: Option<&DocumentTab>) -> bool {
        if let Some(pane) = open_tab_opt {
            pane.is_dirty()
        } else {
            false
        }
    }
}

#[derive(Default)]
pub struct HasRecentFilesState {}

impl CommandState for HasRecentFilesState {
    fn is_enabled(&self, _open_tab_opt: Option<&DocumentTab>) -> bool {
        unsafe { !SETTINGS.get_recent_files().is_empty() }
    }
}

#[derive(Default)]
pub struct CanUndoState {}

impl CommandState for CanUndoState {
    fn is_enabled(&self, open_tab_opt: Option<&DocumentTab>) -> bool {
        if let Some(pane) = open_tab_opt {
            if let Ok(doc) = pane.doc.lock() {
                return doc.can_undo();
            }
        }
        false
    }
}
#[derive(Default)]
pub struct CanRedoState {}

impl CommandState for CanRedoState {
    fn is_enabled(&self, open_tab_opt: Option<&DocumentTab>) -> bool {
        if let Some(pane) = open_tab_opt {
            if let Ok(doc) = pane.doc.lock() {
                return doc.can_redo();
            }
        }
        false
    }
}

#[derive(Default)]
pub struct CanCutState {}

impl CommandState for CanCutState {
    fn is_enabled(&self, open_tab_opt: Option<&DocumentTab>) -> bool {
        if let Some(pane) = open_tab_opt {
            if let Ok(doc) = pane.doc.lock() {
                return doc.can_cut();
            }
        }
        false
    }
}

#[derive(Default)]
pub struct CanCopyState {}

impl CommandState for CanCopyState {
    fn is_enabled(&self, open_tab_opt: Option<&DocumentTab>) -> bool {
        if let Some(pane) = open_tab_opt {
            if let Ok(doc) = pane.doc.lock() {
                return doc.can_copy();
            }
        }
        false
    }
}

#[derive(Default)]
pub struct CanPasteState {}

impl CommandState for CanPasteState {
    fn is_enabled(&self, open_tab_opt: Option<&DocumentTab>) -> bool {
        if let Some(pane) = open_tab_opt {
            if let Ok(doc) = pane.doc.lock() {
                return doc.can_paste();
            }
        }
        false
    }
}

pub struct CommandWrapper {
    key: Option<(KeyOrPointer, Modifiers)>,
    message: Message,
    label: String,
    pub is_enabled: bool,
    pub is_checked: Option<bool>,
    state_key: u32,
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
    ($( ($l:ident, $translation: expr, $message:ident, $cmd_state: ident$(, $key:ident, $modifier: ident)? ) ),* $(,)? ) => {
        pub struct Commands {
            state_map: HashMap<u32, Box<dyn CommandState>>,
            $(
                pub $l: CommandWrapper,
            )*
        }

        impl Default for Commands {
            fn default() -> Self {
                let mut state_map = HashMap::<u32, Box<dyn CommandState>>::new();
                $(
                    state_map.insert(hash(stringify!($cmd_state)), Box::<$cmd_state>::default());
                )*

                Self {
                    state_map,
                    $(
                        $l: CommandWrapper::new(key!($($key, $modifier)?), Message::$message, fl!(crate::LANGUAGE_LOADER, $translation), hash(stringify!($cmd_state))),
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

            pub fn update_states(&mut self, open_tab_opt: Option<&DocumentTab>) {
                let mut result_map = HashMap::new();
                for (k, v) in &self.state_map {
                    let is_enabled = v.is_enabled(open_tab_opt);
                    let is_checked = v.is_checked(open_tab_opt);
                    result_map.insert(k, (is_enabled, is_checked));
                }

                $(
                    self.$l.update_state(&result_map);
                )*
            }

        }
    };
}

fn hash(str: impl Into<String>) -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    str.into().hash(&mut hasher);
    hasher.finish() as u32
}

impl CommandWrapper {
    pub fn new(
        key: Option<(KeyOrPointer, Modifiers)>,
        message: Message,
        description: String,
        state_key: u32,
    ) -> Self {
        Self {
            key,
            message,
            label: description,
            state_key,
            is_enabled: true,
            is_checked: None,
        }
    }

    pub fn update_state(&mut self, result_map: &HashMap<&u32, (bool, Option<bool>)>) {
        let (is_enabled, is_checked) = result_map.get(&self.state_key).unwrap();
        self.is_enabled = *is_enabled;
        self.is_checked = *is_checked;
    }

    pub fn is_pressed(&self, ctx: &egui::Context) -> bool {
        self.key.pressed(ctx)
    }

    pub fn ui(&self, ui: &mut egui::Ui, message: &mut Option<Message>) {
        if let Some(mut checked) = self.is_checked {
            if ui
                .add(egui::Checkbox::new(&mut checked, &self.label))
                .clicked()
            {
                *message = Some(self.message.clone());
                ui.close_menu();
            }
            return;
        }

        let response = ui.with_layout(ui.layout().with_cross_justify(true), |ui| {
            ui.set_enabled(self.is_enabled);
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

                button_with_shortcut(ui, true, &self.label, shortcut)
            } else {
                ui.add(egui::Button::new(&self.label).wrap(false))
            }
        });

        if response.inner.clicked() {
            *message = Some(self.message.clone());
            ui.close_menu();
        }
    }
}

keys![
    (
        new_file,
        "menu-new",
        NewFileDialog,
        AlwaysEnabledState,
        N,
        CTRL
    ),
    (save, "menu-save", SaveFile, FileIsDirtyState, S, CTRL),
    (
        save_as,
        "menu-save-as",
        SaveFileAs,
        FileOpenState,
        S,
        CTRL_SHIFT
    ),
    (
        open_file,
        "menu-open",
        OpenFileDialog,
        AlwaysEnabledState,
        O,
        CTRL
    ),
    (export, "menu-export", ExportFile, BufferOpenState),
    (
        edit_font_outline,
        "menu-edit-font-outline",
        ShowOutlineDialog,
        AlwaysEnabledState
    ),
    (
        close_window,
        "menu-close",
        CloseWindow,
        BufferOpenState,
        Q,
        CTRL
    ),
    (undo, "menu-undo", Undo, CanUndoState, Z, CTRL),
    (redo, "menu-redo", Redo, CanRedoState, Z, CTRL_SHIFT),
    (cut, "menu-cut", Cut, CanCutState, X, CTRL),
    (copy, "menu-copy", Copy, CanCopyState, C, CTRL),
    (paste, "menu-paste", Paste, CanPasteState, V, CTRL),
    (
        show_settings,
        "menu-show_settings",
        ShowSettings,
        AlwaysEnabledState
    ),
    (
        select_all,
        "menu-select-all",
        SelectAll,
        BufferOpenState,
        A,
        CTRL
    ),
    (
        deselect,
        "menu-select_nothing",
        SelectNothing,
        BufferOpenState
    ),
    (
        erase_selection,
        "menu-erase",
        DeleteSelection,
        BufferOpenState,
        Delete,
        NONE
    ),
    (flip_x, "menu-flipx", FlipX, BufferOpenState),
    (flip_y, "menu-flipy", FlipY, BufferOpenState),
    (justifycenter, "menu-justifycenter", Center, BufferOpenState),
    (
        justifyleft,
        "menu-justifyleft",
        JustifyLeft,
        BufferOpenState
    ),
    (
        justifyright,
        "menu-justifyright",
        JustifyRight,
        BufferOpenState
    ),
    (crop, "menu-crop", Crop, BufferOpenState),
    (about, "menu-about", ShowAboutDialog, AlwaysEnabledState),
    (
        justify_line_center,
        "menu-justify_line_center",
        CenterLine,
        BufferOpenState,
        C,
        ALT
    ),
    (
        justify_line_left,
        "menu-justify_line_left",
        JustifyLineLeft,
        BufferOpenState,
        L,
        ALT
    ),
    (
        justify_line_right,
        "menu-justify_line_right",
        JustifyLineRight,
        BufferOpenState,
        R,
        ALT
    ),
    (
        insert_row,
        "menu-insert_row",
        InsertRow,
        BufferOpenState,
        ArrowUp,
        ALT
    ),
    (
        delete_row,
        "menu-delete_row",
        DeleteRow,
        BufferOpenState,
        ArrowDown,
        ALT
    ),
    (
        insert_column,
        "menu-insert_colum",
        InsertColumn,
        BufferOpenState,
        ArrowRight,
        ALT
    ),
    (
        delete_column,
        "menu-delete_colum",
        DeleteColumn,
        BufferOpenState,
        ArrowLeft,
        ALT
    ),
    (
        erase_row,
        "menu-erase_row",
        EraseRow,
        BufferOpenState,
        E,
        ALT
    ),
    (
        erase_row_to_start,
        "menu-erase_row_to_start",
        EraseRowToStart,
        BufferOpenState,
        Home,
        ALT
    ),
    (
        erase_row_to_end,
        "menu-erase_row_to_end",
        EraseRowToEnd,
        BufferOpenState,
        End,
        ALT
    ),
    (
        erase_column,
        "menu-erase_column",
        EraseColumn,
        BufferOpenState,
        E,
        ALT
    ),
    (
        erase_column_to_start,
        "menu-erase_column_to_start",
        EraseColumnToStart,
        BufferOpenState,
        Home,
        ALT
    ),
    (
        erase_column_to_end,
        "menu-erase_column_to_end",
        EraseColumnToEnd,
        BufferOpenState,
        End,
        ALT
    ),
    (
        scroll_area_up,
        "menu-scroll_area_up",
        ScrollAreaUp,
        BufferOpenState,
        ArrowUp,
        ALT_CTRL
    ),
    (
        scroll_area_down,
        "menu-scroll_area_down",
        ScrollAreaDown,
        BufferOpenState,
        ArrowDown,
        ALT_CTRL
    ),
    (
        scroll_area_left,
        "menu-scroll_area_left",
        ScrollAreaLeft,
        BufferOpenState,
        ArrowLeft,
        ALT_CTRL
    ),
    (
        scroll_area_right,
        "menu-scroll_area_right",
        ScrollAreaRight,
        BufferOpenState,
        ArrowRight,
        ALT_CTRL
    ),
    (
        set_reference_image,
        "menu-reference-image",
        SetReferenceImage,
        BufferOpenState,
        O,
        CTRL_SHIFT
    ),
    (
        toggle_reference_image,
        "menu-toggle-reference-image",
        ToggleReferenceImage,
        BufferOpenState,
        Tab,
        CTRL
    ),
    (
        clear_reference_image,
        "menu-clear-reference-image",
        ClearReferenceImage,
        BufferOpenState
    ),
    (
        pick_attribute_under_caret,
        "menu-pick_attribute_under_caret",
        PickAttributeUnderCaret,
        BufferOpenState,
        U,
        ALT
    ),
    (
        switch_to_default_color,
        "menu-default_color",
        SwitchToDefaultColor,
        BufferOpenState,
        D,
        CTRL
    ),
    (
        toggle_color,
        "menu-toggle_color",
        ToggleColor,
        BufferOpenState,
        X,
        ALT
    ),
    (
        fullscreen,
        "menu-toggle_fullscreen",
        ToggleFullScreen,
        AlwaysEnabledState,
        F11,
        NONE
    ),
    (
        zoom_reset,
        "menu-zoom_reset",
        ZoomReset,
        BufferOpenState,
        Num0,
        CTRL
    ),
    (
        zoom_in,
        "menu-zoom_in",
        ZoomIn,
        BufferOpenState,
        PlusEquals,
        CTRL
    ),
    (
        zoom_out,
        "menu-zoom_out",
        ZoomOut,
        BufferOpenState,
        Minus,
        CTRL
    ),
    (
        open_tdf_directory,
        "menu-open_tdf_directoy",
        OpenTdfDirectory,
        AlwaysEnabledState
    ),
    (
        open_font_selector,
        "menu-open_font_selector",
        OpenFontSelector,
        BufferOpenState
    ),
    (
        open_font_manager,
        "menu-open_font_manager",
        OpenFontManager,
        BufferOpenState
    ),
    (
        open_font_directory,
        "menu-open_font_directoy",
        OpenFontDirectory,
        AlwaysEnabledState
    ),
    (
        open_palettes_directory,
        "menu-open_palettes_directoy",
        OpenPalettesDirectory,
        AlwaysEnabledState
    ),
    (
        mirror_mode,
        "menu-mirror_mode",
        ToggleMirrorMode,
        BufferOpenState
    ),
    (
        clear_recent_open,
        "menu-open_recent_clear",
        ClearRecentOpenFiles,
        HasRecentFilesState
    ),
    (
        inverse_selection,
        "menu-inverse_selection",
        InverseSelection,
        BufferOpenState
    ),
    (
        clear_selection,
        "menu-delete_row",
        ClearSelection,
        BufferOpenState,
        Escape,
        NONE
    ),
    (
        select_palette,
        "menu-select_palette",
        SelectPalette,
        BufferOpenState
    ),
    (
        show_layer_borders,
        "menu-show_layer_borders",
        ToggleLayerBorders,
        LayerBordersState
    ),
    (
        show_line_numbers,
        "menu-show_line_numbers",
        ToggleLineNumbers,
        LineNumberState
    ),
    (
        open_plugin_directory,
        "menu-open_plugin_directory",
        OpenPluginDirectory,
        AlwaysEnabledState
    ),
    (
        next_fg_color,
        "menu-next_fg_color",
        NextFgColor,
        BufferOpenState,
        ArrowDown,
        CTRL
    ),
    (
        prev_fg_color,
        "menu-prev_fg_color",
        PreviousFgColor,
        BufferOpenState,
        ArrowUp,
        CTRL
    ),
    (
        next_bg_color,
        "menu-next_bg_color",
        NextBgColor,
        BufferOpenState,
        ArrowRight,
        CTRL
    ),
    (
        prev_bg_color,
        "menu-prev_bg_color",
        PreviousBgColor,
        BufferOpenState,
        ArrowLeft,
        CTRL
    ),
];
