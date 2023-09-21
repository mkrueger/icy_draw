use std::{fs, io::Read, path::Path};

use eframe::{
    egui::{self, Button, Response, Sense, TextEdit, WidgetText},
    epaint::{ahash::HashMap, Color32, FontFamily, FontId, Pos2, Rect, Rounding, Stroke, Vec2},
};
use egui_extras::RetainedImage;
use egui_modal::Modal;
use i18n_embed_fl::fl;
use icy_engine::{AttributedChar, BitFont, Buffer, TextAttribute, ANSI_FONTS, SAUCE_FONT_NAMES};
use walkdir::WalkDir;

use crate::{
    create_retained_image, is_font_extensions, AnsiEditor, Message, Settings, TerminalResult,
};

enum BitfontSource {
    AnsiSlot(usize),
    Sauce(String),
    File(usize),
    Library,
    LibraryAndFile(usize),
}

pub struct FontSelector {
    fonts: Vec<(BitFont, BitfontSource)>,
    selected_font: i32,

    filter: String,
    show_builtin: bool,
    show_library: bool,
    show_file: bool,
    show_sauce: bool,
    should_add: bool,

    image_cache: HashMap<usize, RetainedImage>,
    do_select: bool,
    edit_selected_font: bool,
}

impl FontSelector {
    pub fn new(editor: &AnsiEditor, should_add: bool) -> Self {
        let mut fonts = Vec::new();
        for f in SAUCE_FONT_NAMES {
            fonts.push((
                BitFont::from_sauce_name(f).unwrap(),
                BitfontSource::Sauce(f.to_string()),
            ));
        }

        let only_sauce_fonts = matches!(
            editor.buffer_view.lock().get_buffer().font_mode,
            icy_engine::FontMode::Sauce
        );

        if !only_sauce_fonts {
            for f in 0..ANSI_FONTS {
                fonts.push((
                    BitFont::from_ansi_font_page(f).unwrap(),
                    BitfontSource::AnsiSlot(f),
                ));
            }

            if let Ok(font_dir) = Settings::get_font_diretory() {
                for font in FontSelector::load_fonts(font_dir.as_path()) {
                    fonts.push((font, BitfontSource::Library));
                }
            }
        }

        let mut selected_font = 0;
        let cur_font = editor.buffer_view.lock().get_caret().get_font_page();

        for (id, font) in editor.buffer_view.lock().get_buffer().font_iter() {
            let mut index = -1;
            (0..fonts.len()).for_each(|i| {
                if fonts[i].0.glyphs == font.glyphs {
                    index = i as i32;
                }
            });
            if !only_sauce_fonts {
                if index < 0 {
                    index = fonts.len() as i32;
                    fonts.push((font.clone(), BitfontSource::File(*id)));
                } else {
                    fonts[index as usize].1 = BitfontSource::LibraryAndFile(*id);
                }
            }

            if *id == cur_font {
                selected_font = index;
            }
        }

        Self {
            do_select: false,
            fonts,
            image_cache: HashMap::default(),
            selected_font,
            filter: String::new(),
            show_builtin: true,
            show_library: true,
            show_file: true,
            show_sauce: true,
            edit_selected_font: false,
            should_add,
        }
    }

    fn load_fonts(tdf_dir: &Path) -> Vec<BitFont> {
        let mut fonts = Vec::new();
        let walker = WalkDir::new(tdf_dir).into_iter();
        for entry in walker.filter_entry(|e| !crate::model::font_imp::FontTool::is_hidden(e)) {
            if let Err(e) = entry {
                log::error!("Can't load font library: {e}");
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
            let ext = extension.unwrap().to_lowercase().to_string();

            if is_font_extensions(&ext) {
                if let Ok(font) = BitFont::load(path) {
                    fonts.push(font);
                }
            }

            if ext == "zip" {
                match fs::File::open(path) {
                    Ok(mut file) => {
                        let mut data = Vec::new();
                        file.read_to_end(&mut data).unwrap_or_default();
                        FontSelector::read_zip_archive(data, &mut fonts);
                    }

                    Err(err) => {
                        log::error!("Failed to open zip file: {}", err);
                    }
                }
            }
        }
        fonts
    }

    fn read_zip_archive(data: Vec<u8>, fonts: &mut Vec<BitFont>) {
        let file = std::io::Cursor::new(data);
        match zip::ZipArchive::new(file) {
            Ok(mut archive) => {
                for i in 0..archive.len() {
                    match archive.by_index(i) {
                        Ok(mut file) => {
                            if let Some(path) = file.enclosed_name() {
                                let file_name = path.to_string_lossy().to_string();
                                let ext = path.extension().unwrap().to_str().unwrap();
                                if is_font_extensions(&ext.to_ascii_lowercase()) {
                                    let mut data = Vec::new();
                                    file.read_to_end(&mut data).unwrap_or_default();
                                    if let Ok(font) = BitFont::from_bytes(file_name, &data) {
                                        fonts.push(font)
                                    }
                                } else if ext == "zip" {
                                    let mut data = Vec::new();
                                    file.read_to_end(&mut data).unwrap_or_default();
                                    FontSelector::read_zip_archive(data, fonts);
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

    pub fn draw_font_row(
        &mut self,
        ui: &mut egui::Ui,
        cur_font: usize,
        row_height: f32,
        is_selected: bool,
    ) -> Response {
        let font = &self.fonts[cur_font];
        let (id, rect) = ui.allocate_space([ui.available_width(), row_height].into());
        let response = ui.interact(rect, id, Sense::click());

        if response.hovered() {
            ui.painter().rect_filled(
                rect.expand(1.0),
                Rounding::same(4.0),
                ui.style().visuals.widgets.active.bg_fill,
            );
        } else if is_selected {
            ui.painter().rect_filled(
                rect.expand(1.0),
                Rounding::same(4.0),
                ui.style().visuals.extreme_bg_color,
            );
        }

        let text_color = if is_selected {
            ui.style().visuals.strong_text_color()
        } else {
            ui.style().visuals.text_color()
        };

        let font_id = FontId::new(14.0, FontFamily::Proportional);
        let text: WidgetText = font.0.name.clone().into();
        let galley = text.into_galley(ui, Some(false), f32::INFINITY, font_id);
        ui.painter().galley_with_color(
            egui::Align2::LEFT_TOP
                .align_size_within_rect(galley.size(), rect.shrink(4.0))
                .min,
            galley.galley,
            text_color,
        );

        #[allow(clippy::map_entry)]
        if !self.image_cache.contains_key(&cur_font) {
            let mut buffer = Buffer::new((64, 4));
            buffer.set_font(0, font.0.clone());
            for ch in 0..256 {
                buffer.layers[0].set_char(
                    (ch % 64, ch / 64),
                    AttributedChar::new(
                        unsafe { char::from_u32_unchecked(ch as u32) },
                        TextAttribute::default(),
                    ),
                );
            }
            let img = create_retained_image(&buffer);
            self.image_cache.insert(cur_font, img);
        }

        if let Some(image) = self.image_cache.get(&cur_font) {
            let w = (image.width() as f32).floor();
            let h = (image.height() as f32).floor();
            let r = Rect::from_min_size(
                Pos2::new((rect.left() + 4.0).floor(), (rect.top() + 24.0).floor()),
                Vec2::new(w, h),
            );
            ui.painter().image(
                image.texture_id(ui.ctx()),
                r,
                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );

            let font_type = match font.1 {
                BitfontSource::AnsiSlot(_) => {
                    fl!(crate::LANGUAGE_LOADER, "font_selector-ansi_font")
                }
                BitfontSource::Library => {
                    fl!(crate::LANGUAGE_LOADER, "font_selector-library_font")
                }
                BitfontSource::LibraryAndFile(_) | BitfontSource::File(_) => {
                    fl!(crate::LANGUAGE_LOADER, "font_selector-file_font")
                }
                BitfontSource::Sauce(_) => {
                    fl!(crate::LANGUAGE_LOADER, "font_selector-sauce_font")
                }
            };

            let font_id = FontId::new(12.0, FontFamily::Proportional);
            let text: WidgetText = font_type.into();
            let galley = text.into_galley(ui, Some(false), f32::INFINITY, font_id);

            let rect = Rect::from_min_size(
                Pos2::new(
                    (rect.right() - galley.size().x - 10.0).floor(),
                    (rect.top() + 8.0).floor(),
                ),
                galley.size(),
            );

            ui.painter().rect_filled(
                rect.expand(2.0),
                Rounding::same(4.0),
                ui.style().visuals.widgets.active.bg_fill,
            );

            ui.painter()
                .rect_stroke(rect.expand(2.0), 4.0, Stroke::new(1.0, text_color));

            ui.painter().galley_with_color(
                egui::Align2::CENTER_CENTER
                    .align_size_within_rect(galley.size(), rect)
                    .min,
                galley.galley,
                text_color,
            );

            let secondary_font_type = match font.1 {
                BitfontSource::LibraryAndFile(_) => {
                    fl!(crate::LANGUAGE_LOADER, "font_selector-library_font")
                }
                _ => String::new(),
            };

            if !secondary_font_type.is_empty() {
                let font_id = FontId::new(12.0, FontFamily::Proportional);
                let text: WidgetText = secondary_font_type.into();
                let galley = text.into_galley(ui, Some(false), f32::INFINITY, font_id);

                let rect = Rect::from_min_size(
                    Pos2::new(
                        (rect.right() - galley.size().x - 30.0).floor(),
                        rect.top().floor(),
                    ),
                    galley.size(),
                );

                ui.painter().rect_filled(
                    rect.expand(2.0),
                    Rounding::same(4.0),
                    ui.style().visuals.widgets.active.bg_fill,
                );

                ui.painter()
                    .rect_stroke(rect.expand(2.0), 4.0, Stroke::new(1.0, text_color));

                ui.painter().galley_with_color(
                    egui::Align2::CENTER_CENTER
                        .align_size_within_rect(galley.size(), rect)
                        .min,
                    galley.galley,
                    text_color,
                );
            }
        }
        response
    }
}

impl crate::ModalDialog for FontSelector {
    fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut result = false;
        let modal = Modal::new(ctx, "select_font_dialog2");
        let font_count = self.fonts.len();
        modal.show(|ui| {
            modal.title(
                ui,
                if self.should_add {
                    fl!(
                        crate::LANGUAGE_LOADER,
                        "add-font-dialog-title",
                        fontcount = font_count
                    )
                } else {
                    fl!(
                        crate::LANGUAGE_LOADER,
                        "select-font-dialog-title",
                        fontcount = font_count
                    )
                },
            );
            modal.frame(ui, |ui| {
                let row_height = 200.0 / 2.0;
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.add_sized(
                        [250.0, 20.0],
                        TextEdit::singleline(&mut self.filter).hint_text(fl!(
                            crate::LANGUAGE_LOADER,
                            "select-font-dialog-filter-text"
                        )),
                    );
                    let response = ui.button("🗙");
                    if response.clicked() {
                        self.filter.clear();
                    }

                    let response = ui.selectable_label(
                        self.show_library,
                        fl!(crate::LANGUAGE_LOADER, "font_selector-library_font"),
                    );
                    if response.clicked() {
                        self.show_library = !self.show_library;
                    }

                    let response = ui.selectable_label(
                        self.show_file,
                        fl!(crate::LANGUAGE_LOADER, "font_selector-file_font"),
                    );
                    if response.clicked() {
                        self.show_file = !self.show_file;
                    }

                    let response = ui.selectable_label(
                        self.show_builtin,
                        fl!(crate::LANGUAGE_LOADER, "font_selector-ansi_font"),
                    );
                    if response.clicked() {
                        self.show_builtin = !self.show_builtin;
                    }
                    let response = ui.selectable_label(
                        self.show_sauce,
                        fl!(crate::LANGUAGE_LOADER, "font_selector-sauce_font"),
                    );
                    if response.clicked() {
                        self.show_sauce = !self.show_sauce;
                    }
                });
                ui.add_space(4.0);

                let mut filtered_fonts = Vec::new();

                for i in 0..font_count {
                    let font = &self.fonts[i];
                    let match_filter = match font.1 {
                        BitfontSource::AnsiSlot(_) => self.show_builtin,
                        BitfontSource::File(_) => self.show_file,
                        BitfontSource::Library => self.show_library,
                        BitfontSource::Sauce(_) => self.show_sauce,
                        BitfontSource::LibraryAndFile(_) => self.show_file || self.show_library,
                    };

                    if font
                        .0
                        .name
                        .to_lowercase()
                        .contains(&self.filter.to_lowercase())
                        && match_filter
                    {
                        filtered_fonts.push(i);
                    }
                }
                if filtered_fonts.is_empty() {
                    if font_count == 0 {
                        ui.label(fl!(
                            crate::LANGUAGE_LOADER,
                            "select-font-dialog-no-fonts-installed"
                        ));
                    } else {
                        ui.label(fl!(crate::LANGUAGE_LOADER, "select-font-dialog-no-fonts"));
                    }
                } else {
                    egui::ScrollArea::vertical().max_height(300.).show_rows(
                        ui,
                        row_height,
                        filtered_fonts.len(),
                        |ui, range| {
                            for row in range {
                                let is_selected = self.selected_font == filtered_fonts[row] as i32;
                                let response = self.draw_font_row(
                                    ui,
                                    filtered_fonts[row],
                                    row_height,
                                    is_selected,
                                );

                                if response.clicked() {
                                    self.selected_font = filtered_fonts[row] as i32;
                                }
                                if response.double_clicked() {
                                    self.selected_font = filtered_fonts[row] as i32;
                                    self.do_select = true;
                                    result = true;
                                }
                            }
                        },
                    );
                }
            });

            modal.buttons(ui, |ui| {
                let text = if self.should_add {
                    fl!(crate::LANGUAGE_LOADER, "add-font-dialog-select")
                } else {
                    fl!(crate::LANGUAGE_LOADER, "select-font-dialog-select")
                };

                if ui.button(text).clicked() {
                    self.do_select = true;
                    result = true;
                }
                if ui
                    .button(fl!(crate::LANGUAGE_LOADER, "new-file-cancel"))
                    .clicked()
                {
                    result = true;
                }

                let enabled = self.fonts[self.selected_font as usize].0.path_opt.is_some();
                if ui
                    .add_enabled(
                        enabled,
                        Button::new(fl!(
                            crate::LANGUAGE_LOADER,
                            "select-font-dialog-edit-button"
                        )),
                    )
                    .clicked()
                {
                    self.edit_selected_font = true;
                    result = true;
                }
            });
        });
        modal.open();
        result
    }

    fn should_commit(&self) -> bool {
        self.do_select || self.edit_selected_font
    }

    fn commit(&self, editor: &mut AnsiEditor) -> TerminalResult<Option<Message>> {
        if self.edit_selected_font {
            let font = &self.fonts[self.selected_font as usize];
            if let Some(path) = &font.0.path_opt {
                return Ok(Some(Message::TryLoadFile(path.clone())));
            }
            return Ok(Some(Message::ShowError("Invalid font.".to_string())));
        }

        if let Some((font, src)) = self.fonts.get(self.selected_font as usize) {
            match src {
                BitfontSource::AnsiSlot(id) => {
                    if self.should_add {
                        return Ok(Some(Message::AddAnsiFont(*id)));
                    } else {
                        return Ok(Some(Message::SetAnsiFont(*id)));
                    }
                }
                BitfontSource::LibraryAndFile(id) | BitfontSource::File(id) => {
                    return Ok(Some(Message::SwitchToFontPage(*id)));
                }
                BitfontSource::Sauce(name) => {
                    if self.should_add {
                        return Ok(Some(Message::AddFont(Box::new(font.clone()))));
                    } else {
                        return Ok(Some(Message::SetSauceFont(name.clone())));
                    }
                }

                BitfontSource::Library => {
                    let mut font_set = false;
                    let mut font_slot = 0;
                    editor
                        .buffer_view
                        .lock()
                        .get_buffer()
                        .font_iter()
                        .for_each(|(id, f)| {
                            if f == font {
                                font_slot = *id;
                                font_set = true;
                            }
                        });
                    if font_set {
                        return Ok(Some(Message::SwitchToFontPage(font_slot)));
                    }

                    if !font_set {
                        if self.should_add {
                            return Ok(Some(Message::AddFont(Box::new(font.clone()))));
                        } else {
                            return Ok(Some(Message::SetFont(Box::new(font.clone()))));
                        }
                    }
                }
            }
        }

        Ok(None)
    }
}
