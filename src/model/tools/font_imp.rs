use std::{
    fs,
    sync::{Arc, Mutex},
};

use crate::{AnsiEditor, Message, SETTINGS};

use super::{Event, MKey, MModifiers, Position, Tool};
use directories::ProjectDirs;
use eframe::{
    egui::{self, RichText},
    epaint::{FontFamily, FontId},
};
use icy_engine::{Rectangle, Size, TextAttribute, TheDrawFont};
use walkdir::{DirEntry, WalkDir};
pub struct FontTool {
    pub selected_font: Arc<Mutex<i32>>,
    pub fonts: Arc<Mutex<Vec<TheDrawFont>>>,
    pub sizes: Vec<Size>,
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
                fs::create_dir_all(&tdf_dir).unwrap_or_else(|_| {
                    panic!(
                        "Can't create tdf font directory {:?}",
                        proj_dirs.config_dir()
                    )
                });
            }
            let mut fonts = Vec::new();
            let walker = WalkDir::new(tdf_dir).into_iter();
            for entry in walker.filter_entry(|e| !FontTool::is_hidden(e)) {
                if let Err(e) = entry {
                    eprintln!("Can't load tdf font library: {e}");
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
                    if let Ok(loaded_fonts) = TheDrawFont::load(path) {
                        fonts.extend(loaded_fonts);
                    }
                }
            }

            self.fonts = Arc::new(Mutex::new(fonts));
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
        _buffer_opt: &AnsiEditor,
    ) -> Option<Message> {
        let mut select = false;
        let font_count = self.fonts.lock().unwrap().len();
        let selected_font = *self.selected_font.lock().unwrap();

        ui.vertical_centered(|ui| {
            ui.label("Selected font");

            let mut selected_text = "<none>".to_string();

            if selected_font >= 0 && (selected_font as usize) < font_count {
                if let Some(font) = self.fonts.lock().unwrap().get(selected_font as usize) {
                    selected_text = font.name.clone();
                }
            }
            let selected_text =
                RichText::new(selected_text).font(FontId::new(18.0, FontFamily::Proportional));
            select = ui.button(selected_text).clicked();
        });

        ui.add_space(8.0);

        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                if let Some(font) = self.fonts.lock().unwrap().get(selected_font as usize) {
                    for ch in '!'..'@' {
                        ui.spacing_mut().item_spacing = eframe::epaint::Vec2::new(0.0, 0.0);
                        let color = if font.has_char(ch as u8) {
                            ui.style().visuals.strong_text_color()
                        } else {
                            ui.style().visuals.text_color()
                        };

                        ui.colored_label(
                            color,
                            RichText::new(ch.to_string())
                                .font(FontId::new(12.0, FontFamily::Monospace)),
                        );
                    }
                }
            });

            ui.horizontal(|ui| {
                if let Some(font) = self.fonts.lock().unwrap().get(selected_font as usize) {
                    for ch in '@'..'_' {
                        ui.spacing_mut().item_spacing = eframe::epaint::Vec2::new(0.0, 0.0);
                        let color = if font.has_char(ch as u8) {
                            ui.style().visuals.strong_text_color()
                        } else {
                            ui.style().visuals.text_color()
                        };

                        ui.colored_label(
                            color,
                            RichText::new(ch.to_string())
                                .font(FontId::new(12.0, FontFamily::Monospace)),
                        );
                    }
                }
            });

            ui.horizontal(|ui| {
                if let Some(font) = self.fonts.lock().unwrap().get(selected_font as usize) {
                    ui.spacing_mut().item_spacing = eframe::epaint::Vec2::new(0.0, 0.0);
                    for ch in '_'..'~' {
                        let color = if font.has_char(ch as u8) {
                            ui.style().visuals.strong_text_color()
                        } else {
                            ui.style().visuals.text_color()
                        };

                        ui.colored_label(
                            color,
                            RichText::new(ch.to_string())
                                .font(FontId::new(12.0, FontFamily::Monospace)),
                        );
                    }
                }
            });
            ui.horizontal(|ui| {
                if let Some(font) = self.fonts.lock().unwrap().get(selected_font as usize) {
                    ui.spacing_mut().item_spacing = eframe::epaint::Vec2::new(0.0, 0.0);
                    for ch in '~'..='~' {
                        let color = if font.has_char(ch as u8) {
                            ui.style().visuals.strong_text_color()
                        } else {
                            ui.style().visuals.text_color()
                        };

                        ui.colored_label(
                            color,
                            RichText::new(ch.to_string())
                                .font(FontId::new(12.0, FontFamily::Monospace)),
                        );
                    }
                }
            });
        });
        ui.add_space(32.0);
        if ui.button("Select Font Outline").clicked() {
            return Some(Message::ShowOutlineDialog);
        }

        ui.add_space(32.0);
        ui.label("Install new fonts in the font directory.");
        if ui.button("Open font directory").clicked() {
            if let Some(proj_dirs) = ProjectDirs::from("com", "GitHub", "icy_draw") {
                let tdf_dir = proj_dirs.config_dir().join("tdf");
                if let Err(err) = open::that(tdf_dir) {
                    return Some(Message::ShowError(format!(
                        "Can't open font directory: {err}"
                    )));
                }
            }
        }

        if select {
            Some(Message::SelectFontDialog(
                self.fonts.clone(),
                self.selected_font.clone(),
            ))
        } else {
            None
        }
    }

    fn handle_click(&mut self, editor: &mut AnsiEditor, button: i32, pos: Position) -> Event {
        if button == 1 {
            editor.set_caret_position(pos);
            editor.buffer_view.lock().clear_selection();
        }
        Event::None
    }

    fn handle_key(&mut self, editor: &mut AnsiEditor, key: MKey, modifier: MModifiers) -> Event {
        let selected_font = *self.selected_font.lock().unwrap();

        if selected_font < 0 || selected_font >= self.fonts.lock().unwrap().len() as i32 {
            return Event::None;
        }
        let font = &self.fonts.lock().unwrap()[selected_font as usize];
        let pos = editor.buffer_view.lock().caret.get_position();

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
                    let end = editor.buffer_view.lock().buf.get_width() as i32;
                    for i in 0..end {
                        if !editor
                            .get_char_from_cur_layer(pos.with_x(i))
                            .is_transparent()
                        {
                            editor.set_caret(i, pos.y);
                            return Event::None;
                        }
                    }
                }
                editor.set_caret(0, pos.y);
            }

            MKey::End => {
                if let MModifiers::Control = modifier {
                    let end = editor.buffer_view.lock().buf.get_width() as i32;
                    for i in (0..end).rev() {
                        if !editor
                            .get_char_from_cur_layer(pos.with_x(i))
                            .is_transparent()
                        {
                            editor.set_caret(i, pos.y);
                            return Event::None;
                        }
                    }
                }
                let w = editor.buffer_view.lock().buf.get_width() as i32;
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
                editor.buffer_view.lock().clear_selection();
                let pos = editor.get_caret_position();
                if pos.x > 0 {
                    editor.set_caret_position(pos + Position::new(-(letter_size.width as i32), 0));
                    if editor.buffer_view.lock().caret.insert_mode {
                        let end = (editor.buffer_view.lock().buf.get_width() - (letter_size.width))
                            as i32;
                        for i in pos.x..end {
                            let next = editor.get_char_from_cur_layer(Position::new(
                                i + letter_size.width as i32,
                                pos.y,
                            ));
                            editor.set_char(Position::new(i, pos.y), next);
                        }
                        let last_pos = Position::new(
                            (editor.buffer_view.lock().buf.get_width() - (letter_size.width))
                                as i32,
                            pos.y,
                        );
                        editor.fill(
                            Rectangle {
                                start: last_pos,
                                size: letter_size,
                            },
                            super::AttributedChar::new(' ', TextAttribute::default()),
                        );
                    } else {
                        let pos = editor.get_caret_position();
                        editor.fill(
                            Rectangle {
                                start: pos,
                                size: letter_size,
                            },
                            super::AttributedChar::new(' ', TextAttribute::default()),
                        );
                    }
                }
            }

            MKey::Character(ch) => {
                let c_pos = editor.get_caret_position();
                editor.begin_atomic_undo();
                let attr = editor.buffer_view.lock().caret.get_attribute();
                let opt_size: Option<Size> = font.render(
                    &mut editor.buffer_view.lock().buf,
                    0,
                    c_pos.as_uposition(),
                    attr,
                    unsafe { SETTINGS.font_outline_style },
                    ch as u8,
                );
                if let Some(size) = opt_size {
                    editor.set_caret(c_pos.x + size.width as i32 + font.spaces, c_pos.y);
                    let new_pos = editor.get_caret_position();
                    self.sizes.push(Size {
                        width: (new_pos.x - c_pos.x) as usize,
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
