use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex},
};

use eframe::{
    egui::{self},
    epaint::Vec2,
};
use icy_engine::{
    util::pop_data, BitFont, EngineResult, Layer, Size, TextAttribute, TextPane, TheDrawFont,
};

use crate::{
    util::autosave::{self},
    AnsiEditor, DocumentOptions, MainWindow, NewFileDialog, OpenFileDialog, SaveFileDialog,
    SelectCharacterDialog, SelectOutlineDialog, Settings,
};

#[derive(Clone)]
pub enum Message {
    NewFileDialog,
    OpenFileDialog,
    SaveFile,
    SaveFileAs,
    ExportFile,
    ShowOutlineDialog,
    CloseWindow,

    AddNewLayer(usize),
    EditLayer(usize),
    RemoveLayer(usize),
    RaiseLayer(usize),
    LowerLayer(usize),
    ToggleLayerVisibility(usize),
    SelectLayer(usize),
    DuplicateLayer(usize),
    MergeLayerDown(usize),

    Undo,
    Redo,
    EditSauce,
    SetCanvasSize,
    SelectAll,
    Deselect,
    DeleteSelection,

    ShowAboutDialog,
    ShowCharacterSelectionDialog(Rc<RefCell<char>>),
    SelectFontDialog(Arc<Mutex<Vec<TheDrawFont>>>, Arc<Mutex<i32>>),
    ShowError(String),
    SetFontPage(usize),
    CharTable(char),
    ResizeLayer(usize),
    SelectTool(usize),
    AnchorLayer,
    AddFloatingLayer,
    JustifyLeft,
    JustifyRight,
    Center,
    FlipX,
    FlipY,
    Crop,
    Paste,
    ResizeBuffer(bool, i32, i32),
    PasteAsNewImage,
    PasteAsBrush,
    Copy,
    Cut,
    RemoveFloatingLayer,
    SelectOutline(usize),

    CenterLine,
    JustifyLineLeft,
    JustifyLineRight,

    InsertRow,
    DeleteRow,
    InsertColumn,
    DeleteColumn,

    EraseRow,
    EraseRowToStart,
    EraseRowToEnd,

    EraseColumn,
    EraseColumnToStart,
    EraseColumnToEnd,

    ScrollAreaUp,
    ScrollAreaDown,
    ScrollAreaLeft,
    ScrollAreaRight,

    SetReferenceImage,
    ToggleReferenceImage,
    ClearReferenceImage,

    PickAttributeUnderCaret,
    SwitchToDefaultColor,
    ToggleColor,

    StampLayerDown,
    RotateLayer,
    MakeLayerTransparent,

    ToggleFullScreen,

    ZoomReset,
    ZoomIn,
    ZoomOut,

    OpenFontManager,
    OpenFontDirectory,
    OpenTdfDirectory,
    OpenPalettesDirectory,
    ToggleMirrorMode,
    SetGuide(i32, i32),
    SetRaster(i32, i32),
    LoadFile(PathBuf, bool),
    TryLoadFile(PathBuf),
    ClearLayer(usize),
}

pub const CTRL_SHIFT: egui::Modifiers = egui::Modifiers {
    alt: false,
    ctrl: true,
    shift: true,
    mac_cmd: false,
    command: false,
};

impl MainWindow {
    pub fn handle_message(&mut self, msg_opt: Option<Message>) {
        if msg_opt.is_none() {
            return;
        }
        match msg_opt.unwrap() {
            Message::NewFileDialog => {
                self.open_dialog(NewFileDialog::default());
            }
            Message::OpenFileDialog => {
                self.open_dialog(OpenFileDialog::default());
            }

            Message::TryLoadFile(path) => {
                let auto_save = autosave::get_autosave_file(&path);
                if auto_save.exists() {
                    self.open_dialog(crate::AutoSaveDialog::new(path));
                    return;
                }

                self.open_file(&path, false);
            }
            Message::LoadFile(path, load_autosave) => {
                self.open_file(&path, load_autosave);
            }

            Message::SaveFile => {
                let msg = if let Some(pane) = self.get_active_pane() {
                    if pane.is_untitled() {
                        Some(Message::SaveFileAs)
                    } else {
                        pane.save()
                    }
                } else {
                    None
                };

                self.handle_message(msg);
            }
            Message::SaveFileAs => {
                if self.get_active_document().is_some() {
                    self.open_dialog(SaveFileDialog::default());
                }
            }
            Message::ExportFile => {
                self.run_editor_command(0, |window, editor, _| {
                    let view = editor.buffer_view.clone();
                    window.open_dialog(crate::ExportFileDialog::new(view.lock().get_buffer()));
                    None
                });
                if let Some(editor) = self
                    .get_active_document()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .get_ansi_editor()
                {
                    let view = editor.buffer_view.clone();
                    self.open_dialog(crate::ExportFileDialog::new(view.lock().get_buffer()));
                }
            }
            Message::ShowOutlineDialog => {
                self.open_dialog(SelectOutlineDialog::default());
            }
            Message::Undo => {
                if let Some(editor) = self.get_active_document() {
                    self.handle_result(editor.lock().unwrap().undo());
                }
            }
            Message::Redo => {
                if let Some(editor) = self.get_active_document() {
                    self.handle_result(editor.lock().unwrap().redo());
                }
            }

            Message::SelectAll => {
                self.run_editor_command(0, |_, editor, _| {
                    let buf = &mut editor.buffer_view.lock();
                    let w = buf.get_buffer().get_width();
                    let h = buf.get_buffer().get_height();

                    buf.set_selection(icy_engine::Rectangle::from(0, 0, w, h));
                    None
                });
            }
            Message::Deselect => {
                self.run_editor_command(0, |_, editor, _| {
                    editor.buffer_view.lock().clear_selection();
                    None
                });
            }
            Message::DeleteSelection => {
                self.run_editor_command(0, |_, editor: &mut AnsiEditor, _| {
                    to_message(
                        editor
                            .buffer_view
                            .lock()
                            .get_edit_state_mut()
                            .delete_selection(),
                    )
                });
            }
            Message::ShowCharacterSelectionDialog(ch) => {
                self.run_editor_command(ch, |window, editor: &mut AnsiEditor, ch| {
                    let buf = editor.buffer_view.clone();
                    window.open_dialog(SelectCharacterDialog::new(buf, ch));
                    None
                });
            }
            Message::SelectFontDialog(fonts, selected_font) => {
                self.open_dialog(crate::SelectFontDialog::new(fonts, selected_font));
            }
            Message::EditSauce => {
                if let Some(editor) = self
                    .get_active_document()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .get_ansi_editor()
                {
                    let view = editor.buffer_view.clone();
                    self.open_dialog(crate::EditSauceDialog::new(view.lock().get_buffer()));
                }
            }

            Message::SetCanvasSize => {
                if let Some(editor) = self
                    .get_active_document()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .get_ansi_editor()
                {
                    let view = editor.buffer_view.clone();
                    self.open_dialog(crate::SetCanvasSizeDialog::new(view.lock().get_buffer()));
                }
            }

            Message::EditLayer(i) => {
                if let Some(editor) = self
                    .get_active_document()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .get_ansi_editor()
                {
                    let view = editor.buffer_view.clone();
                    self.open_dialog(crate::EditLayerDialog::new(view.lock().get_buffer(), i));
                }
            }
            Message::ResizeLayer(i) => {
                if let Some(editor) = self
                    .get_active_document()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .get_ansi_editor_mut()
                {
                    let view = editor.buffer_view.clone();
                    self.open_dialog(crate::ResizeLayerDialog::new(view.lock().get_buffer(), i));
                }
            }
            Message::AddNewLayer(cur_layer) => {
                self.run_editor_command(cur_layer, |_, editor, cur_layer| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().add_new_layer(cur_layer))
                });
            }
            Message::RaiseLayer(cur_layer) => {
                self.run_editor_command(cur_layer, |_, editor, cur_layer| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().raise_layer(cur_layer))
                });
            }
            Message::LowerLayer(cur_layer) => {
                self.run_editor_command(cur_layer, |_, editor, cur_layer| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().lower_layer(cur_layer))
                });
            }
            Message::RemoveLayer(cur_layer) => {
                self.run_editor_command(
                    cur_layer,
                    |_, editor: &mut crate::AnsiEditor, cur_layer| {
                        let mut lock = editor.buffer_view.lock();
                        to_message(lock.get_edit_state_mut().remove_layer(cur_layer))
                    },
                );

                self.run_editor_command(0, |_, editor: &mut crate::AnsiEditor, _| {
                    let mut lock = editor.buffer_view.lock();

                    if lock.get_buffer().layers.is_empty() {
                        to_message(lock.get_edit_state_mut().add_new_layer(0))
                    } else {
                        None
                    }
                });
            }
            Message::ClearLayer(cur_layer) => {
                self.run_editor_command(cur_layer, |_, editor, cur_layer| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().clear_layer(cur_layer))
                });
            }
            Message::RemoveFloatingLayer => {
                self.run_editor_command(0, |_, editor: &mut crate::AnsiEditor, _| {
                    let mut lock = editor.buffer_view.lock();
                    let layer = lock.get_edit_state().get_current_layer();
                    to_message(lock.get_edit_state_mut().remove_layer(layer))
                });
            }
            Message::DuplicateLayer(cur_layer) => {
                self.run_editor_command(
                    cur_layer,
                    |_, editor: &mut crate::AnsiEditor, cur_layer| {
                        let mut lock = editor.buffer_view.lock();
                        to_message(lock.get_edit_state_mut().duplicate_layer(cur_layer))
                    },
                );
            }
            Message::MergeLayerDown(cur_layer) => {
                self.run_editor_command(
                    cur_layer,
                    |_, editor: &mut crate::AnsiEditor, cur_layer| {
                        let mut lock = editor.buffer_view.lock();
                        to_message(lock.get_edit_state_mut().merge_layer_down(cur_layer))
                    },
                );
            }

            Message::ToggleLayerVisibility(cur_layer) => {
                self.run_editor_command(
                    cur_layer,
                    |_, editor: &mut crate::AnsiEditor, cur_layer| {
                        let mut lock = editor.buffer_view.lock();
                        to_message(lock.get_edit_state_mut().toggle_layer_visibility(cur_layer))
                    },
                );
            }

            Message::SelectLayer(cur_layer) => {
                self.run_editor_command(cur_layer, |_, editor, cur_layer| {
                    editor.set_cur_layer_index(cur_layer);
                    None
                });
            }

            Message::AnchorLayer => {
                self.run_editor_command(0, |_, editor: &mut crate::AnsiEditor, _| {
                    to_message(
                        editor
                            .buffer_view
                            .lock()
                            .get_edit_state_mut()
                            .anchor_layer(),
                    )
                });
            }

            Message::AddFloatingLayer => {
                self.run_editor_command(0, |_, editor: &mut crate::AnsiEditor, _| {
                    to_message(
                        editor
                            .buffer_view
                            .lock()
                            .get_edit_state_mut()
                            .add_floating_layer(),
                    )
                });
            }

            Message::SetFontPage(page) => {
                self.run_editor_command(page, |_, editor, page| {
                    editor
                        .buffer_view
                        .lock()
                        .get_caret_mut()
                        .set_font_page(page);

                    let lock = &mut editor.buffer_view.lock();
                    let buf = &mut lock.get_buffer_mut();
                    if buf.get_font(page).is_none() {
                        match BitFont::from_ansi_font_page(page) {
                            Ok(font) => {
                                buf.set_font(page, font);
                            }
                            Err(err) => {
                                log::error!("Failed to load font: {err}");
                            }
                        }
                    }
                    None
                });
            }

            Message::CharTable(ch) => {
                let ch = ch as u8;
                self.run_editor_command(ch, |_, editor, ch| {
                    if let Err(err) = editor.print_char(ch) {
                        return Some(Message::ShowError(format!("{err}")));
                    }
                    None
                });
            }

            Message::SelectTool(tool) => {
                self.document_behavior.selected_tool = tool;
            }

            Message::ShowAboutDialog => {
                self.open_dialog(crate::AboutDialog::default());
            }

            Message::ShowError(msg) => {
                log::error!("{msg}");
                self.toasts.error(msg);
            }

            Message::Paste => {
                if let Some(doc) = self.get_active_document() {
                    self.handle_result(doc.lock().unwrap().paste());
                }
            }

            Message::Cut => {
                if let Some(doc) = self.get_active_document() {
                    self.handle_result(doc.lock().unwrap().cut());
                }
            }
            Message::Copy => {
                if let Some(doc) = self.get_active_document() {
                    self.handle_result(doc.lock().unwrap().copy());
                }
            }

            Message::JustifyLeft => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().justify_left())
                });
            }

            Message::JustifyRight => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().justify_right())
                });
            }

            Message::Center => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().center())
                });
            }

            Message::FlipX => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().flip_x())
                });
            }

            Message::FlipY => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().flip_y())
                });
            }

            Message::Crop => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().crop())
                });
            }
            Message::ResizeBuffer(resize_layer, w, h) => {
                self.run_editor_command((resize_layer, w, h), |_, editor, (resize_layer, w, h)| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(
                        lock.get_edit_state_mut()
                            .resize_buffer(resize_layer, Size::new(w, h)),
                    )
                });
            }

            Message::PasteAsNewImage => {
                if let Some(data) = pop_data(icy_engine::util::BUFFER_DATA) {
                    if let Some(mut layer) = Layer::from_clipboard_data(&data) {
                        layer.set_offset((0, 0));
                        layer.role = icy_engine::Role::Normal;
                        let mut buf = icy_engine::Buffer::new(layer.get_size());
                        layer.title = buf.layers[0].title.clone();
                        buf.layers.clear();
                        buf.layers.push(layer);
                        let id = self.create_id();
                        buf.is_terminal_buffer = false;
                        buf.set_height(buf.get_line_count());
                        let editor = AnsiEditor::new(&self.gl, id, buf);
                        crate::add_child(&mut self.document_tree, None, Box::new(editor));
                    }
                }
            }

            Message::PasteAsBrush => {
                if let Some(data) = pop_data(icy_engine::util::BUFFER_DATA) {
                    if let Some(layer) = Layer::from_clipboard_data(&data) {
                        unsafe {
                            crate::model::brush_imp::CUSTOM_BRUSH = Some(layer);
                            self.document_behavior.selected_tool = crate::BRUSH_TOOL;
                        }
                    }
                }
            }
            Message::CloseWindow => {
                self.is_closed = true;
            }
            Message::CenterLine => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().center_line())
                });
            }
            Message::JustifyLineLeft => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().justify_line_left())
                });
            }
            Message::JustifyLineRight => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().justify_line_right())
                });
            }
            Message::InsertRow => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().insert_row())
                });
            }
            Message::DeleteRow => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().delete_row())
                });
            }
            Message::InsertColumn => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().insert_column())
                });
            }
            Message::DeleteColumn => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().delete_column())
                });
            }
            Message::EraseRow => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().erase_row())
                });
            }
            Message::EraseRowToStart => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().erase_row_to_start())
                });
            }
            Message::EraseRowToEnd => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().erase_row_to_end())
                });
            }
            Message::EraseColumn => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().erase_column())
                });
            }
            Message::EraseColumnToStart => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().erase_column_to_start())
                });
            }
            Message::EraseColumnToEnd => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().erase_column_to_end())
                });
            }
            Message::ScrollAreaUp => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().scroll_area_up())
                });
            }
            Message::ScrollAreaDown => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().scroll_area_down())
                });
            }
            Message::ScrollAreaLeft => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().scroll_area_left())
                });
            }
            Message::ScrollAreaRight => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().scroll_area_right())
                });
            }

            Message::StampLayerDown => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().stamp_layer_down())
                });
            }

            Message::RotateLayer => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().rotate_layer())
                });
            }

            Message::MakeLayerTransparent => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    to_message(lock.get_edit_state_mut().make_layer_transparent())
                });
            }

            Message::SetReferenceImage => {
                let dialog = rfd::FileDialog::new();
                let res = dialog.pick_file();
                if let Some(res) = res {
                    self.run_editor_command(res, |_, editor, res| {
                        let mut lock = editor.buffer_view.lock();
                        lock.load_reference_image(res.as_path());
                        None
                    });
                }
            }
            Message::ToggleReferenceImage => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    lock.toggle_reference_image();
                    None
                });
            }
            Message::ClearReferenceImage => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock: eframe::epaint::mutex::MutexGuard<
                        '_,
                        icy_engine_egui::BufferView,
                    > = editor.buffer_view.lock();
                    lock.clear_reference_image();
                    None
                });
            }

            Message::PickAttributeUnderCaret => {
                self.run_editor_command(0, |_, editor, _| {
                    let bv = &mut editor.buffer_view.lock();
                    let pos = bv.get_caret().get_position();

                    let attr = if let Some(layer) = bv.get_edit_state().get_cur_layer() {
                        let ch = layer.get_char(pos);
                        ch.attribute
                    } else {
                        TextAttribute::default()
                    };

                    let fg = attr.get_foreground();
                    let bg = attr.get_background();
                    let caret = bv.get_caret_mut();
                    caret.set_foreground(bg);
                    caret.set_background(fg);
                    None
                });
            }

            Message::SwitchToDefaultColor => {
                self.run_editor_command(0, |_, editor, _| {
                    let bv = &mut editor.buffer_view.lock();
                    let caret = bv.get_caret_mut();
                    caret.set_foreground(7);
                    caret.set_background(0);
                    None
                });
            }

            Message::ToggleColor => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut attr = editor.buffer_view.lock().get_caret().get_attribute();
                    let fg = attr.get_foreground();
                    let bg = attr.get_background();
                    attr.set_foreground(bg);
                    attr.set_background(fg);
                    editor.buffer_view.lock().get_caret_mut().set_attr(attr);
                    None
                });
            }

            Message::SelectOutline(outline) => {
                Settings::set_character_set(outline);
            }

            Message::ToggleFullScreen => {
                self.is_fullscreen = !self.is_fullscreen;
            }

            Message::ZoomReset => {
                self.document_behavior
                    .document_options
                    .set_scale(DocumentOptions::default().get_scale());
            }

            Message::ZoomIn => {
                self.document_behavior
                    .document_options
                    .set_scale(self.document_behavior.document_options.get_scale() * 1.2);
            }

            Message::ZoomOut => {
                self.document_behavior
                    .document_options
                    .set_scale(self.document_behavior.document_options.get_scale() * 0.8);
            }

            Message::OpenFontDirectory => match Settings::get_font_diretory() {
                Ok(dir) => {
                    if let Err(err) = open::that(dir) {
                        self.handle_message(Some(Message::ShowError(format!(
                            "Can't open font directory: {err}"
                        ))));
                    }
                }
                Err(err) => {
                    self.handle_message(Some(Message::ShowError(format!("{err}"))));
                }
            },

            Message::OpenTdfDirectory => match Settings::get_tdf_diretory() {
                Ok(dir) => {
                    if let Err(err) = open::that(dir) {
                        self.handle_message(Some(Message::ShowError(format!(
                            "Can't open font directory: {err}"
                        ))));
                    }
                }
                Err(err) => {
                    self.handle_message(Some(Message::ShowError(format!("{err}"))));
                }
            },

            Message::OpenPalettesDirectory => match Settings::get_palettes_diretory() {
                Ok(dir) => {
                    if let Err(err) = open::that(dir) {
                        self.handle_message(Some(Message::ShowError(format!(
                            "Can't open font directory: {err}"
                        ))));
                    }
                }
                Err(err) => {
                    self.handle_message(Some(Message::ShowError(format!("{err}"))));
                }
            },
            Message::OpenFontManager => {
                self.run_editor_command(0, |window, editor, _| {
                    window.open_dialog(crate::FontManager::new(editor));
                    None
                });
            }
            Message::SetGuide(x, y) => {
                self.run_editor_command((x, y), |_, editor, (x, y)| {
                    if x <= 0 && y <= 0 {
                        editor.guide = None;
                    } else {
                        editor.guide = Some(Vec2::new(x as f32, y as f32));
                    }
                    None
                });
            }
            Message::SetRaster(x, y) => {
                self.run_editor_command((x, y), |_, editor, (x, y)| {
                    if x <= 0 && y <= 0 {
                        editor.raster = None;
                    } else {
                        editor.raster = Some(Vec2::new(x as f32, y as f32));
                    }
                    None
                });
            }

            Message::ToggleMirrorMode => {
                self.run_editor_command(0, |_, editor, _| {
                    let mode = editor
                        .buffer_view
                        .lock()
                        .get_edit_state_mut()
                        .get_mirror_mode();
                    editor
                        .buffer_view
                        .lock()
                        .get_edit_state_mut()
                        .set_mirror_mode(!mode);
                    None
                });
            }
        }
    }
}

pub fn to_message<T>(result: EngineResult<T>) -> Option<Message> {
    if let Err(result) = result {
        Some(Message::ShowError(format!("{result}")))
    } else {
        None
    }
}
