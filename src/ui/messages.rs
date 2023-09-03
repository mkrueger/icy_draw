use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex},
};

use eframe::egui;
use icy_engine::{BitFont, Selection, TheDrawFont};

use crate::{
    MainWindow, NewFileDialog, OpenFileDialog, SaveFileDialog, SelectCharacterDialog,
    SelectOutlineDialog,
};

pub enum Message {
    NewFile,
    OpenFile,
    SaveFile,
    SaveFileAs,
    ExportFile,
    ShowOutlineDialog,

    AddLayer,
    EditLayer(usize),
    RemoveLayer(usize),
    MoveLayerUp(usize),
    MoveLayerDown(usize),
    ToggleVisibility(usize),
    SelectLayer(usize),
    DuplicateLayer(usize),
    MergeLayer(usize),

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
            Message::NewFile => {
                self.open_dialog(NewFileDialog::default());
            }

            Message::OpenFile => {
                self.open_dialog(OpenFileDialog::default());
            }

            Message::SaveFile => {
                if let Some(doc) = self.get_active_pane() {
                    let mut save_as = true;
                    if let Some(str) = &doc.full_path {
                        let path = PathBuf::from(str);
                        if let Some(ext) = path.extension() {
                            if ext == "icd" {
                                doc.doc.lock().unwrap().save(str).unwrap();
                                save_as = false;
                            }
                        }
                    }
                    if save_as {
                        self.handle_message(Some(Message::SaveFileAs))
                    }
                }
            }
            Message::SaveFileAs => {
                if self.get_active_document().is_some() {
                    self.open_dialog(SaveFileDialog::default());
                }
            }
            Message::ExportFile => {
                self.run_editor_command(0, |window, editor, _| {
                    let view = editor.buffer_view.clone();
                    window.open_dialog(crate::ExportFileDialog::new(&view.lock().get_buffer()));
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
                    self.open_dialog(crate::ExportFileDialog::new(&view.lock().get_buffer()));
                }
            }
            Message::ShowOutlineDialog => {
                self.open_dialog(SelectOutlineDialog::default());
            }
            Message::Undo => {
                if let Some(editor) = self.get_active_document() {
                    editor.lock().unwrap().undo();
                }
            }
            Message::Redo => {
                if let Some(editor) = self.get_active_document() {
                    editor.lock().unwrap().redo();
                }
            }

            Message::SelectAll => {
                if let Some(editor) = self
                    .get_active_document()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .get_ansi_editor()
                {
                    let buf = &mut editor.buffer_view.lock();
                    let w = buf.get_buffer().get_width();
                    let h = buf.get_buffer().get_line_count();

                    buf.set_selection(Selection::from_rectangle(0.0, 0.0, w as f32, h as f32));
                }
            }
            Message::Deselect => {
                if let Some(editor) = self
                    .get_active_document()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .get_ansi_editor()
                {
                    editor.buffer_view.lock().clear_selection();
                    editor.redraw_view();
                }
            }

            Message::DeleteSelection => {
                if let Some(editor) = self
                    .get_active_document()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .get_ansi_editor_mut()
                {
                    if editor.buffer_view.lock().get_selection().is_some() {
                        editor.delete_selection();
                        editor.redraw_view();
                    }
                }
            }

            Message::ShowCharacterSelectionDialog(ch) => {
                if let Some(editor) = self
                    .get_active_document()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .get_ansi_editor()
                {
                    let buf = editor.buffer_view.clone();
                    self.open_dialog(SelectCharacterDialog::new(buf, ch));
                }
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
                    self.open_dialog(crate::EditSauceDialog::new(&view.lock().get_buffer()));
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
                    self.open_dialog(crate::SetCanvasSizeDialog::new(&view.lock().get_buffer()));
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
                    self.open_dialog(crate::EditLayerDialog::new(&view.lock().get_buffer(), i));
                }
            }
            Message::AddLayer => {
                self.run_editor_command(0, |_, editor, _| {
                    let mut lock = editor.buffer_view.lock();
                    lock.get_edit_state_mut().add_layer();
                    None
                });
            }
            Message::MoveLayerUp(cur_layer) => {
                if let Some(editor) = self
                    .get_active_document()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .get_ansi_editor_mut()
                {
                    editor
                        .buffer_view
                        .lock()
                        .get_buffer_mut()
                        .layers
                        .swap(cur_layer, cur_layer - 1);
                    editor.set_cur_layer(editor.get_cur_layer() - 1);
                }
            }
            Message::MoveLayerDown(cur_layer) => {
                if let Some(editor) = self
                    .get_active_document()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .get_ansi_editor_mut()
                {
                    editor
                        .buffer_view
                        .lock()
                        .get_buffer_mut()
                        .layers
                        .swap(cur_layer, cur_layer + 1);
                    editor.set_cur_layer(editor.get_cur_layer() + 1);
                }
            }
            Message::RemoveLayer(cur_layer) => {

                self.run_editor_command(cur_layer, |_, editor: &mut crate::AnsiEditor, cur_layer| {
                    let mut lock = editor.buffer_view.lock();
                    lock.get_edit_state_mut().remove_layer(cur_layer);
                    None
                });

                
            }
            Message::DuplicateLayer(cur_layer) => {
                self.run_editor_command(cur_layer, |_, editor, cur_layer| {
                    let layer = editor.buffer_view.lock().get_buffer().layers[cur_layer].clone();
                    editor
                        .buffer_view
                        .lock()
                        .get_buffer_mut()
                        .layers
                        .insert(cur_layer + 1, layer);
                    editor.set_cur_layer(editor.get_cur_layer() + 1);
                    None
                });
            }
            Message::MergeLayer(cur_layer) => {
                self.run_editor_command(cur_layer, |_, editor, cur_layer| {
                    let layer = editor
                        .buffer_view
                        .lock()
                        .get_buffer_mut()
                        .layers
                        .remove(cur_layer);
                    if let Some(layer_below) = editor
                        .buffer_view
                        .lock()
                        .get_buffer_mut()
                        .layers
                        .get_mut(cur_layer)
                    {
                        for y in 0..layer.get_height() {
                            for x in 0..layer.get_width() {
                                println!("{} {}", x, y);
                                let ch = layer.get_char((x, y));
                                if ch.is_visible() {
                                    layer_below.set_char((x, y), ch);
                                }
                            }
                        }
                    }
                    None
                });
            }

            Message::ToggleVisibility(cur_layer) => {
                if let Some(editor) = self
                    .get_active_document()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .get_ansi_editor_mut()
                {
                    let is_visible =
                        editor.buffer_view.lock().get_buffer().layers[cur_layer].is_visible;
                    editor.buffer_view.lock().get_buffer_mut().layers[cur_layer].is_visible =
                        !is_visible;
                }
            }
            Message::SelectLayer(cur_layer) => {
                if let Some(editor) = self
                    .get_active_document()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .get_ansi_editor_mut()
                {
                    editor.set_cur_layer(cur_layer);
                }
            }

            Message::SetFontPage(page) => {
                if let Some(editor) = self
                    .get_active_document()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .get_ansi_editor_mut()
                {
                    editor
                        .buffer_view
                        .lock()
                        .get_caret_mut()
                        .set_font_page(page);

                    let lock = &mut editor.buffer_view.lock();
                    let buf = &mut lock.get_buffer_mut();
                    if buf.get_font(page).is_none() {
                        if let Some(font_name) =
                            icy_engine::parsers::ansi::constants::ANSI_FONT_NAMES.get(page)
                        {
                            match BitFont::from_name(font_name) {
                                Ok(font) => {
                                    buf.set_font(page, font);
                                }
                                Err(err) => {
                                    log::error!("Failed to load font: {err}");
                                }
                            }
                        }
                    }
                }
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

            Message::ShowAboutDialog => {
                self.open_dialog(crate::AboutDialog::default());
            }

            Message::ShowError(msg) => {
                log::error!("{msg}");
                self.toasts.error(msg);
            }
        }
    }
}
