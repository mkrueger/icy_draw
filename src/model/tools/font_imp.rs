use std::{
    fs,
    io::Read,
    path::Path,
    sync::{Arc, Mutex},
    thread,
};

use crate::{AnsiEditor, Message, SETTINGS};

use super::{Event, MKey, MModifiers, Position, Tool};
use directories::ProjectDirs;
use eframe::{
    egui::{self, RichText},
    epaint::{FontFamily, FontId},
};
use i18n_embed_fl::fl;
use icy_engine::{editor::OperationType, Size, TextPane, TheDrawFont};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
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

    pub fn install_watcher(&self) {
        if let Some(proj_dirs) = ProjectDirs::from("com", "GitHub", "icy_draw") {
            let tdf_dir = proj_dirs.config_dir().join("tdf");
            let fonts = self.fonts.clone();
            thread::spawn(move || loop {
                if watch(tdf_dir.as_path(), &fonts).is_err() {
                    return;
                }
            });
        }
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
            self.fonts = Arc::new(Mutex::new(load_fonts(tdf_dir.as_path())));
        }
    }
}

fn load_fonts(tdf_dir: &Path) -> Vec<TheDrawFont> {
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

        if extension == "zip" {
            match fs::File::open(path) {
                Ok(mut file) => {
                    let mut data = Vec::new();
                    file.read_to_end(&mut data).unwrap_or_default();
                    read_zip_archive(data, &mut fonts);
                }

                Err(err) => {
                    log::error!("Failed to open zip file: {}", err);
                }
            }
        }
    }
    fonts
}

fn read_zip_archive(data: Vec<u8>, fonts: &mut Vec<TheDrawFont>) {
    let file = std::io::Cursor::new(data);
    match zip::ZipArchive::new(file) {
        Ok(mut archive) => {
            for i in 0..archive.len() {
                match archive.by_index(i) {
                    Ok(mut file) => {
                        if let Some(name) = file.enclosed_name() {
                            if name
                                .to_string_lossy()
                                .to_ascii_lowercase()
                                .ends_with(".tdf")
                            {
                                let mut data = Vec::new();
                                file.read_to_end(&mut data).unwrap_or_default();

                                if let Ok(loaded_fonts) = TheDrawFont::from_tdf_bytes(&data) {
                                    fonts.extend(loaded_fonts);
                                }
                            } else if name
                                .to_string_lossy()
                                .to_ascii_lowercase()
                                .ends_with(".zip")
                            {
                                let mut data = Vec::new();
                                file.read_to_end(&mut data).unwrap_or_default();
                                read_zip_archive(data, fonts);
                            }
                        }
                    }
                    Err(err) => {
                        log::error!("Error reading zip file: {}", err);
                    }
                }
            }
        }
        Err(err) => {
            log::error!("Error reading zip archive: {}", err);
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
                    for ch in '!'..'9' {
                        ui.spacing_mut().item_spacing = eframe::epaint::Vec2::new(0.0, 0.0);
                        let color = if font.has_char(ch as u8) {
                            ui.style().visuals.strong_text_color()
                        } else {
                            ui.style().visuals.text_color()
                        };

                        ui.colored_label(
                            color,
                            RichText::new(ch.to_string())
                                .font(FontId::new(14.0, FontFamily::Monospace)),
                        );
                    }
                }
            });

            ui.horizontal(|ui| {
                if let Some(font) = self.fonts.lock().unwrap().get(selected_font as usize) {
                    for ch in '9'..'Q' {
                        ui.spacing_mut().item_spacing = eframe::epaint::Vec2::new(0.0, 0.0);
                        let color = if font.has_char(ch as u8) {
                            ui.style().visuals.strong_text_color()
                        } else {
                            ui.style().visuals.text_color()
                        };

                        ui.colored_label(
                            color,
                            RichText::new(ch.to_string())
                                .font(FontId::new(14.0, FontFamily::Monospace)),
                        );
                    }
                }
            });

            ui.horizontal(|ui| {
                if let Some(font) = self.fonts.lock().unwrap().get(selected_font as usize) {
                    ui.spacing_mut().item_spacing = eframe::epaint::Vec2::new(0.0, 0.0);
                    for ch in 'Q'..'i' {
                        let color = if font.has_char(ch as u8) {
                            ui.style().visuals.strong_text_color()
                        } else {
                            ui.style().visuals.text_color()
                        };

                        ui.colored_label(
                            color,
                            RichText::new(ch.to_string())
                                .font(FontId::new(14.0, FontFamily::Monospace)),
                        );
                    }
                }
            });
            ui.horizontal(|ui| {
                if let Some(font) = self.fonts.lock().unwrap().get(selected_font as usize) {
                    ui.spacing_mut().item_spacing = eframe::epaint::Vec2::new(0.0, 0.0);
                    for ch in 'i'..='~' {
                        let color = if font.has_char(ch as u8) {
                            ui.style().visuals.strong_text_color()
                        } else {
                            ui.style().visuals.text_color()
                        };

                        ui.colored_label(
                            color,
                            RichText::new(ch.to_string())
                                .font(FontId::new(14.0, FontFamily::Monospace)),
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

    fn handle_click(
        &mut self,
        editor: &mut AnsiEditor,
        button: i32,
        pos: Position,
        _pos_abs: Position,
    ) -> Event {
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
        let pos = editor.buffer_view.lock().get_caret().get_position();

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
                    let end = editor.buffer_view.lock().get_buffer().get_width();
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
                    let end = editor.buffer_view.lock().get_buffer().get_width();
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
                let w = editor.buffer_view.lock().get_buffer().get_width();
                editor.set_caret(w - 1, pos.y);
            }

            MKey::Return => {
                editor.set_caret(0, pos.y + font.get_font_height());
                /*
                if let Some(size) = self.sizes.last() {
                    editor.set_caret(0,pos.y + size.height as i32);
                } else {
                    editor.set_caret(0,pos.y + 1);
                }*/
                self.sizes.clear();
            }

            MKey::Backspace => {
                let mut use_backspace = true;
                {
                    let mut render = false;
                    let mut reverse_count = 0;

                    let op = if let Ok(stack) = editor
                        .buffer_view
                        .lock()
                        .get_edit_state()
                        .get_undo_stack()
                        .lock()
                    {
                        for i in (0..stack.len()).rev() {
                            match stack[i].get_operation_type() {
                                OperationType::RenderCharacter => {
                                    if reverse_count == 0 {
                                        render = true;
                                        reverse_count = i;
                                        break;
                                    }
                                    reverse_count -= 1;
                                }
                                OperationType::ReversedRenderCharacter => {
                                    reverse_count += 1;
                                }
                                _ => {
                                    render = false;
                                }
                            }
                        }
                        stack[reverse_count].try_clone()
                    } else {
                        None
                    };

                    if render {
                        if let Some(op) = op {
                            let _ = editor
                                .buffer_view
                                .lock()
                                .get_edit_state_mut()
                                .push_reverse_undo(
                                    fl!("undo-delete_character"),
                                    op,
                                    OperationType::ReversedRenderCharacter,
                                );
                            use_backspace = false;
                        }
                    }
                }

                if use_backspace {
                    editor.backspace();
                }
            }
            MKey::Character(ch) => {
                let c_pos = editor.get_caret_position();
                let _undo = editor
                    .buffer_view
                    .lock()
                    .get_edit_state_mut()
                    .begin_typed_atomic_undo(
                        fl!("undo-render_character"),
                        OperationType::RenderCharacter,
                    );
                editor
                    .buffer_view
                    .lock()
                    .get_edit_state_mut()
                    .set_outline_style(unsafe { SETTINGS.font_outline_style });

                let _ = editor
                    .buffer_view
                    .lock()
                    .get_edit_state_mut()
                    .undo_caret_position();

                let opt_size: Option<Size> =
                    font.render(editor.buffer_view.lock().get_edit_state_mut(), ch as u8);
                if let Some(size) = opt_size {
                    editor.set_caret(c_pos.x + size.width + font.spaces, c_pos.y);
                    let new_pos = editor.get_caret_position();
                    self.sizes.push(Size {
                        width: (new_pos.x - c_pos.x),
                        height: size.height,
                    });
                } else {
                    editor.type_key(unsafe { char::from_u32_unchecked(ch as u32) });
                    self.sizes.push(Size::new(1, 1));
                }
            }
            _ => {}
        }
        Event::None
    }
}

fn watch(path: &Path, fonts: &Arc<Mutex<Vec<TheDrawFont>>>) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(_) => {
                fonts.lock().unwrap().clear();
                fonts.lock().unwrap().extend(load_fonts(path));

                break;
            }
            Err(e) => println!("watch error: {e:}"),
        }
    }

    Ok(())
}
