use std::path::PathBuf;

use eframe::egui;
use icy_engine::Selection;

use crate::{MainWindow, NewFileDialog, OpenFileDialog, SaveFileDialog, SelectOutlineDialog};

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
    Undo,
    Redo,
    EditSauce,
    SetCanvasSize,
    SelectAll,
    Deselect,

    ShowAboutDialog,
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
                                doc.doc.save(str).unwrap();
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
                if self.get_active_document_mut().is_some() {
                    self.open_dialog(SaveFileDialog::default());
                }
            }
            Message::ExportFile => {
                let mut buffer_opt = None;
                if let Some(doc) = self.get_active_document_mut() {
                    buffer_opt = doc.get_ansi_editor_mut();
                }
                let view = buffer_opt.unwrap().buffer_view.clone();
                self.open_dialog(crate::ExportFileDialog::new(&view.lock().buf));
            }
            Message::ShowOutlineDialog => {
                self.open_dialog(SelectOutlineDialog::default());
            }
            Message::Undo => {
                if let Some(doc) = self.get_active_document_mut() {
                    let doc = doc.get_ansi_editor_mut();
                    if let Some(editor) = doc {
                        editor.undo();
                        editor.buffer_view.lock().redraw_view();
                    }
                }
            }
            Message::Redo => {
                if let Some(doc) = self.get_active_document_mut() {
                    let doc = doc.get_ansi_editor_mut();
                    if let Some(editor) = doc {
                        editor.redo();
                        editor.buffer_view.lock().redraw_view();
                    }
                }
            }

            Message::SelectAll => {
                if let Some(doc) = self.get_active_document_mut() {
                    let doc = doc.get_ansi_editor_mut();
                    if let Some(ansi_editor) = doc {
                        let buf = &mut ansi_editor.buffer_view.lock();
                        let w = buf.buf.get_width();
                        let h = buf.buf.get_line_count();

                        buf.set_selection(Selection::from_rectangle(0.0, 0.0, w as f32, h as f32));
                    }
                }
            }
            Message::Deselect => {
                if let Some(doc) = self.get_active_document_mut() {
                    let doc = doc.get_ansi_editor_mut();
                    if let Some(editor) = doc {
                        editor.cur_selection = None;
                        editor.redraw_view();
                    }
                }
            }

            Message::EditSauce => {
                let mut buffer_opt = None;
                if let Some(doc) = self.get_active_document_mut() {
                    buffer_opt = doc.get_ansi_editor_mut();
                }
                let view = buffer_opt.unwrap().buffer_view.clone();
                self.open_dialog(crate::EditSauceDialog::new(&view.lock().buf));
            }
            Message::SetCanvasSize => {
                let mut buffer_opt = None;
                if let Some(doc) = self.get_active_document_mut() {
                    buffer_opt = doc.get_ansi_editor_mut();
                }

                let view = buffer_opt.unwrap().buffer_view.clone();
                self.open_dialog(crate::SetCanvasSizeDialog::new(&view.lock().buf));
            }

            Message::EditLayer(i) => {
                let editor = self
                    .get_active_document_mut()
                    .unwrap()
                    .get_ansi_editor_mut()
                    .unwrap();
                let buffer_view = editor.buffer_view.clone();
                self.open_dialog(crate::EditLayerDialog::new(&buffer_view.lock().buf, i));
            }
            Message::NewLayer => {
                let editor = self
                    .get_active_document_mut()
                    .unwrap()
                    .get_ansi_editor_mut()
                    .unwrap();
                let buf = &mut editor.buffer_view.lock().buf;
                let size = buf.get_buffer_size();
                let mut new_layer = icy_engine::Layer::new("New Layer", size);
                if buf.layers.is_empty() {
                    new_layer.has_alpha_channel = false;
                }

                buf.layers.insert(0, new_layer);
            }
            Message::MoveLayerUp(cur_layer) => {
                let editor = self
                    .get_active_document_mut()
                    .unwrap()
                    .get_ansi_editor_mut()
                    .unwrap();

                editor
                    .buffer_view
                    .lock()
                    .buf
                    .layers
                    .swap(cur_layer, cur_layer - 1);
                editor.cur_layer -= 1;
            }
            Message::MoveLayerDown(cur_layer) => {
                let editor = self
                    .get_active_document_mut()
                    .unwrap()
                    .get_ansi_editor_mut()
                    .unwrap();

                editor
                    .buffer_view
                    .lock()
                    .buf
                    .layers
                    .swap(cur_layer, cur_layer + 1);
                editor.cur_layer += 1;
            }
            Message::DeleteLayer(cur_layer) => {
                let editor = self
                    .get_active_document_mut()
                    .unwrap()
                    .get_ansi_editor_mut()
                    .unwrap();
                editor.buffer_view.lock().buf.layers.remove(cur_layer);
                editor.cur_layer = editor.cur_layer.clamp(
                    0,
                    editor.buffer_view.lock().buf.layers.len().saturating_sub(1),
                );
            }
            Message::ToggleVisibility(cur_layer) => {
                let editor = self
                    .get_active_document_mut()
                    .unwrap()
                    .get_ansi_editor_mut()
                    .unwrap();
                let is_visible = editor.buffer_view.lock().buf.layers[cur_layer].is_visible;
                editor.buffer_view.lock().buf.layers[cur_layer].is_visible = !is_visible;
            }
            Message::SelectLayer(cur_layer) => {
                let editor = self
                    .get_active_document_mut()
                    .unwrap()
                    .get_ansi_editor_mut()
                    .unwrap();
                editor.cur_layer = cur_layer;
            }

            Message::ShowAboutDialog => {
                self.open_dialog(crate::AboutDialog::default());
            }
        }
    }
}
