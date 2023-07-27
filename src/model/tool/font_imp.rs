use std::{
    fs,
    sync::{Arc, Mutex},
};

use crate::{ansi_editor::BufferView, SETTINGS};

use super::{Event, MKey,  MModifiers, Position, Tool, ToolUiResult};
use directories::ProjectDirs;
use eframe::{
    egui::{self, ComboBox},
    epaint::{
        text::LayoutJob,
        Color32, FontId,
    },
};
use icy_engine::{Rectangle, Size, TextAttribute, TheDrawFont};
use walkdir::{DirEntry, WalkDir};
pub struct FontTool {
    pub selected_font: i32,
    pub fonts: Vec<TheDrawFont>,
    pub sizes: Vec<Size<i32>>,
}

impl FontTool {
    /*pub fn get_selected_font(&self) -> Option<&TheDrawFont> {
        self.fonts.get(self.selected_font as usize)
    }*/

    fn is_hidden(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map_or(false, |s| s.starts_with('.'))
    }

    pub fn load_fonts(&mut self) {
        if let Some(proj_dirs) = ProjectDirs::from("com", "GitHub", "icy_draw") {
            let tdf_dir = proj_dirs.config_dir().join("tdf");
            if !tdf_dir.exists() {
                fs::create_dir_all(&tdf_dir).expect(&format!(
                    "Can't create tdf font directory {:?}",
                    proj_dirs.config_dir()
                ));
            }
            self.fonts.clear();
            let walker = WalkDir::new(tdf_dir).into_iter();
            for entry in walker.filter_entry(|e| !FontTool::is_hidden(e)) {
                if let Err(e) = entry {
                    eprintln!("Can't load tdf font library: {}", e);
                    break;
                }
                let entry = entry.unwrap();
                let path = entry.path();

                if path.is_dir() {
                    continue;
                }
                let extension = path.extension();
                if extension.is_none() {
                    continue;
                }
                let extension = extension.unwrap().to_str();
                if extension.is_none() {
                    continue;
                }
                let extension = extension.unwrap().to_lowercase();

                if extension == "tdf" {
                    if let Some(font) = TheDrawFont::load(path) {
                        self.fonts.push(font);
                    }
                }
            }
        }
    }
}

impl Tool for FontTool {
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage {
        &super::icons::FONT_SVG
    }
    fn use_selection(&self) -> bool {
        false
    }

    fn show_ui(
        &mut self,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        _buffer_opt: Option<std::sync::Arc<std::sync::Mutex<crate::ui::ansi_editor::BufferView>>>,
    ) -> ToolUiResult {
        ui.vertical_centered(|ui| {
            let mut selected_text = "<none>".to_string();

            if self.selected_font >= 0 && (self.selected_font as usize) < self.fonts.len() {
                if let Some(font) = self.fonts.get(self.selected_font as usize) {
                    selected_text = font.name.clone();
                }
            }

            ComboBox::from_label("Font")
                .wrap(false)
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for i in 0..self.fonts.len() {
                        let text = LayoutJob::simple_singleline(
                            self.fonts[i].name.clone(),
                            FontId::default(),
                            Color32::WHITE,
                        );
                        ui.selectable_value(&mut self.selected_font, i as i32, text);
                    }
                });
        });
        ToolUiResult::new()
    }

    fn handle_click(
        &mut self,
        buffer_view: Arc<Mutex<BufferView>>,
        button: i32,
        pos: Position,
    ) -> Event {
        if button == 1 {
            let editor = &mut buffer_view.lock().unwrap().editor;
            editor.set_caret_position(pos);
            editor.cur_selection = None;
        }
        Event::None
    }

    fn handle_key(
        &mut self,
        buffer_view: Arc<Mutex<BufferView>>,
        key: MKey,
        modifier: MModifiers,
    ) -> Event {
        if self.selected_font < 0 || self.selected_font >= self.fonts.len() as i32 {
            return Event::None;
        }
        let font = &self.fonts[self.selected_font as usize];
        let editor = &mut buffer_view.lock().unwrap().editor;
        let pos = editor.caret.get_position();

        match key {
            MKey::Down => {
                editor.set_caret(pos.x, pos.y + 1);
            }
            MKey::Up => {
                editor.set_caret(pos.x, pos.y - 1);
            }
            MKey::Left => {
                editor.set_caret(pos.x - 1, pos.y);
            }
            MKey::Right => {
                editor.set_caret(pos.x + 1, pos.y);
            }

            MKey::Home => {
                if let MModifiers::Control = modifier {
                    for i in 0..editor.buf.get_buffer_width() {
                        if !editor
                            .get_char_from_cur_layer(pos.with_x(i as i32))
                            .unwrap_or_default()
                            .is_transparent()
                        {
                            editor.set_caret(i as i32, pos.y);
                            return Event::None;
                        }
                    }
                }
                editor.set_caret(0, pos.y);
            }

            MKey::End => {
                if let MModifiers::Control = modifier {
                    for i in (0..editor.buf.get_buffer_width()).rev() {
                        if !editor
                            .get_char_from_cur_layer(pos.with_x(i as i32))
                            .unwrap_or_default()
                            .is_transparent()
                        {
                            editor.set_caret(i as i32, pos.y);
                            return Event::None;
                        }
                    }
                }
                let w = editor.buf.get_buffer_width() as i32;
                editor.set_caret(w - 1, pos.y);
            }

            MKey::Return => {
                editor.set_caret(0, pos.y + font.get_font_height() as i32);
                /*
                if let Some(size) = self.sizes.last() {
                    editor.set_caret(0,pos.y + size.height as i32);
                } else {
                    editor.set_caret(0,pos.y + 1);
                }*/
                self.sizes.clear();
            }

            MKey::Backspace => {
                let letter_size = self.sizes.pop().unwrap_or_else(|| Size::new(1, 1));
                editor.cur_selection = None;
                let pos = editor.get_caret_position();
                if pos.x > 0 {
                    editor.set_caret_position(pos + Position::new(-(letter_size.width as i32), 0));
                    if editor.caret.insert_mode {
                        for i in pos.x
                            ..(editor.buf.get_buffer_width() as i32 - (letter_size.width as i32))
                        {
                            let next = editor.get_char_from_cur_layer(Position::new(
                                i + (letter_size.width as i32),
                                pos.y,
                            ));
                            editor.set_char(Position::new(i, pos.y), next);
                        }
                        let last_pos = Position::new(
                            editor.buf.get_buffer_width() as i32 - (letter_size.width as i32),
                            pos.y,
                        );
                        editor.fill(
                            Rectangle {
                                start: last_pos,
                                size: letter_size,
                            },
                            Some(super::AttributedChar::new(' ', TextAttribute::default())),
                        );
                    } else {
                        let pos = editor.get_caret_position();
                        editor.fill(
                            Rectangle {
                                start: pos,
                                size: letter_size,
                            },
                            Some(super::AttributedChar::new(' ', TextAttribute::default())),
                        );
                    }
                }
            }

            MKey::Character(ch) => {
                let c_pos = editor.get_caret_position();
                editor.begin_atomic_undo();
                let attr = editor.caret.get_attribute();
                let opt_size = font.render(&mut editor.buf, 0, c_pos, attr, unsafe { SETTINGS.font_outline_style }, ch as u8);
                if let Some(size) = opt_size {
                    editor.set_caret(c_pos.x + size.width as i32 + font.spaces, c_pos.y);
                    let new_pos = editor.get_caret_position();
                    self.sizes.push(Size {
                        width: (new_pos.x - c_pos.x),
                        height: size.height,
                    });
                } else {
                    editor.type_key(unsafe { char::from_u32_unchecked(ch as u32) });
                    self.sizes.push(Size::new(1, 1));
                }
                editor.end_atomic_undo();
            }
            _ => {}
        }
        Event::None
    }
}
