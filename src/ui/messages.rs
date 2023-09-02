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

    NewLayer,
    EditLayer(usize),
    DeleteLayer(usize),
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
                    window.open_dialog(crate::ExportFileDialog::new(&view.lock().buf));
                });
                if let Some(editor) = self.get_active_document().unwrap().lock().unwrap().get_ansi_editor() {
                    let view = editor.buffer_view.clone();
                    self.open_dialog(crate::ExportFileDialog::new(&view.lock().buf));
                }
            }
            Message::ShowOutlineDialog => {
                self.open_dialog(SelectOutlineDialog::default());
            }
            Message::Undo => {
                if let Some(editor) = self.get_active_document().unwrap().lock().unwrap().get_ansi_editor_mut() {
                    editor.undo();
                    editor.buffer_view.lock().redraw_view();
                }
            }
            Message::Redo => {
                if let Some(editor) = self.get_active_document().unwrap().lock().unwrap().get_ansi_editor_mut() {
                editor.redo();
                    editor.buffer_view.lock().redraw_view();
                }
            
            }

            Message::SelectAll => {
                if let Some(editor) = self.get_active_document().unwrap().lock().unwrap().get_ansi_editor() {
                    let buf = &mut editor.buffer_view.lock();
                        let w = buf.buf.get_width();
                        let h = buf.buf.get_line_count();

                        buf.set_selection(Selection::from_rectangle(0.0, 0.0, w as f32, h as f32));
                    }
            }
            Message::Deselect => {
                if let Some(editor) = self.get_active_document().unwrap().lock().unwrap().get_ansi_editor() {
                    editor.buffer_view.lock().clear_selection();
                        editor.redraw_view();
                    }
            }

            Message::DeleteSelection => {
                if let Some(editor) = self.get_active_document().unwrap().lock().unwrap().get_ansi_editor_mut() {
                    if editor.buffer_view.lock().get_selection().is_some() {
                            editor.delete_selection();
                            editor.redraw_view();
                        }
                }
            }

            Message::ShowCharacterSelectionDialog(ch) => {
                if let Some(editor) = self.get_active_document().unwrap().lock().unwrap().get_ansi_editor() {
                    let buf = editor.buffer_view.clone();
                        self.open_dialog(SelectCharacterDialog::new(buf, ch));
                    }
            }
            Message::SelectFontDialog(fonts, selected_font) => {
                self.open_dialog(crate::SelectFontDialog::new(fonts, selected_font));
            }

            Message::EditSauce => {
                if let Some(editor) = self.get_active_document().unwrap().lock().unwrap().get_ansi_editor()  {
                    let view = editor.buffer_view.clone();
                    self.open_dialog(crate::EditSauceDialog::new(&view.lock().buf));
                }
            }
            Message::SetCanvasSize => {
                if let Some(editor) = self.get_active_document().unwrap().lock().unwrap().get_ansi_editor()  {
                    let view = editor.buffer_view.clone();
                    self.open_dialog(crate::SetCanvasSizeDialog::new(&view.lock().buf));
                }
            }

            Message::EditLayer(i) => {
                if let Some(editor) = self.get_active_document().unwrap().lock().unwrap().get_ansi_editor()  {
                    let view = editor.buffer_view.clone();
                    self.open_dialog(crate::EditLayerDialog::new(&view.lock().buf, i));
                }
            }
            Message::NewLayer => {
                if let Some(editor) = self.get_active_document().unwrap().lock().unwrap().get_ansi_editor_mut()  {
                    let buf = &mut editor.buffer_view.lock().buf;
                let size = buf.get_buffer_size();
                let mut new_layer = icy_engine::Layer::new("New Layer", size);
                new_layer.has_alpha_channel = true;
                if buf.layers.is_empty() {
                    new_layer.has_alpha_channel = false;
                }

                buf.layers.insert(0, new_layer);
            }
            }
            Message::MoveLayerUp(cur_layer) => {
                if let Some(editor) = self.get_active_document().unwrap().lock().unwrap().get_ansi_editor_mut()  {

                editor
                    .buffer_view
                    .lock()
                    .buf
                    .layers
                    .swap(cur_layer, cur_layer - 1);
                editor.cur_layer -= 1;
                }
            }
            Message::MoveLayerDown(cur_layer) => {
                if let Some(editor) = self.get_active_document().unwrap().lock().unwrap().get_ansi_editor_mut()  {
                    editor
                    .buffer_view
                    .lock()
                    .buf
                    .layers
                    .swap(cur_layer, cur_layer + 1);
                editor.cur_layer += 1;
                }
            }
            Message::DeleteLayer(cur_layer) => {
                self.run_editor_command(cur_layer, |_, editor, cur_layer| {
                    editor.buffer_view.lock().buf.layers.remove(cur_layer);
                    editor.cur_layer = editor.cur_layer.clamp(
                    0,
                    editor.buffer_view.lock().buf.layers.len().saturating_sub(1),
                    );
                });
            }
            Message::DuplicateLayer(cur_layer) => {
                self.run_editor_command(cur_layer, |_, editor, cur_layer| {
                    let layer = editor.buffer_view.lock().buf.layers[cur_layer].clone();
                    editor.buffer_view.lock().buf.layers.insert(cur_layer + 1, layer);
                    editor.cur_layer += 1;
                });
            }
            Message::MergeLayer(cur_layer) => {
                self.run_editor_command(cur_layer, |_, editor, cur_layer| {
                    let layer = editor.buffer_view.lock().buf.layers.remove(cur_layer);
                    if let Some(layer_below) = editor.buffer_view.lock().buf.layers.get_mut(cur_layer) {
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
                });
            }

            Message::ToggleVisibility(cur_layer) => {
                if let Some(editor) = self.get_active_document().unwrap().lock().unwrap().get_ansi_editor_mut()  {
                    let is_visible = editor.buffer_view.lock().buf.layers[cur_layer].is_visible;
                    editor.buffer_view.lock().buf.layers[cur_layer].is_visible = !is_visible;
                }
            }
            Message::SelectLayer(cur_layer) => {
                if let Some(editor) = self.get_active_document().unwrap().lock().unwrap().get_ansi_editor_mut()  {
                    editor.cur_layer = cur_layer;
                }
            }

            Message::SetFontPage(page) => {
                if let Some(editor) = self.get_active_document().unwrap().lock().unwrap().get_ansi_editor_mut()  {
                    editor.buffer_view.lock().caret.set_font_page(page);

                    let buf = &mut editor.buffer_view.lock().buf;
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
                let ch  =ch as u8;
                self.run_editor_command(ch,|_, editor, ch| {
                    editor.print_char(ch);
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
