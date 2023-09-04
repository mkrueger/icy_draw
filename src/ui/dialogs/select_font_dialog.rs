use std::sync::{Arc, Mutex};

use eframe::{
    egui::{self, RichText, TextEdit},
    epaint::{ahash::HashMap, ColorImage, FontFamily, FontId, Stroke},
};
use egui_extras::RetainedImage;
use egui_modal::Modal;
use i18n_embed_fl::fl;
use icy_engine::{Buffer, Position, Rectangle, Size, TextAttribute, TheDrawFont, editor::EditState};

use crate::MainWindow;

pub struct SelectFontDialog {
    fonts: Arc<Mutex<Vec<TheDrawFont>>>,
    selected_font_arc: Arc<Mutex<i32>>,
    selected_font: i32,
    pub do_select: bool,
    filter: String,
    show_outline: bool,
    show_color: bool,
    show_block: bool,

    image_cache: HashMap<usize, RetainedImage>,
}

impl SelectFontDialog {
    pub fn new(fonts: Arc<Mutex<Vec<TheDrawFont>>>, selected_font_arc: Arc<Mutex<i32>>) -> Self {
        let selected_font = *selected_font_arc.lock().unwrap();

        Self {
            do_select: false,
            fonts,
            selected_font_arc,
            selected_font,
            filter: String::new(),
            show_outline: true,
            show_color: true,
            show_block: true,
            image_cache: HashMap::default(),
        }
    }
}

impl crate::ModalDialog for SelectFontDialog {
    fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut result = false;
        let modal = Modal::new(ctx, "select_font_dialog");
        let font_count = self.fonts.lock().unwrap().len();
        modal.show(|ui| {
            modal.title(
                ui,
                fl!(
                    crate::LANGUAGE_LOADER,
                    "select-font-dialog-title",
                    fontcount = font_count
                ),
            );
            modal.frame(ui, |ui| {
                let row_height = 176.0 / 2.0;
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

                    let response = ui.selectable_label(self.show_color, "COLOR");
                    if response.clicked() {
                        self.show_color = !self.show_color;
                    }

                    let response = ui.selectable_label(self.show_block, "BLOCK");
                    if response.clicked() {
                        self.show_block = !self.show_block;
                    }

                    let response = ui.selectable_label(self.show_outline, "OUTLINE");
                    if response.clicked() {
                        self.show_outline = !self.show_outline;
                    }
                });
                ui.add_space(4.0);

                let mut filtered_fonts = Vec::new();

                for i in 0..font_count {
                    let font = &self.fonts.lock().unwrap()[i];
                    if font
                        .name
                        .to_lowercase()
                        .contains(&self.filter.to_lowercase())
                        && (self.show_outline
                            && matches!(font.font_type, icy_engine::FontType::Outline)
                            || self.show_block
                                && matches!(font.font_type, icy_engine::FontType::Block)
                            || self.show_color
                                && matches!(font.font_type, icy_engine::FontType::Color))
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
                                let font = &self.fonts.lock().unwrap()[filtered_fonts[row]];
                                ui.horizontal(|ui: &mut egui::Ui| {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            let font_type = match font.font_type {
                                                icy_engine::FontType::Outline => "OUTLINE",
                                                icy_engine::FontType::Block => "BLOCK",
                                                icy_engine::FontType::Color => "COLOR",
                                            };
                                            let response = ui.label(font_type);
                                            ui.painter().rect_stroke(
                                                response.rect.expand(2.0),
                                                4.0,
                                                Stroke::new(1.0, ctx.style().visuals.text_color()),
                                            );

                                            let sel = ui.selectable_label(
                                                self.selected_font == filtered_fonts[row] as i32,
                                                font.name.clone(),
                                            );
                                            if sel.clicked() {
                                                self.selected_font = filtered_fonts[row] as i32;
                                            }
                                            if sel.double_clicked() {
                                                self.selected_font = filtered_fonts[row] as i32;
                                                self.do_select = true;
                                                result = true;
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            for ch in '!'..'P' {
                                                ui.spacing_mut().item_spacing =
                                                    eframe::epaint::Vec2::new(0.0, 0.0);
                                                let color = if font.has_char(ch as u8) {
                                                    ui.style().visuals.strong_text_color()
                                                } else {
                                                    ui.style().visuals.text_color()
                                                };

                                                ui.colored_label(
                                                    color,
                                                    RichText::new(ch.to_string()).font(
                                                        FontId::new(12.0, FontFamily::Monospace),
                                                    ),
                                                );
                                            }
                                        });

                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing =
                                                eframe::epaint::Vec2::new(0.0, 0.0);
                                            for ch in 'P'..='~' {
                                                let color = if font.has_char(ch as u8) {
                                                    ui.style().visuals.strong_text_color()
                                                } else {
                                                    ui.style().visuals.text_color()
                                                };

                                                ui.colored_label(
                                                    color,
                                                    RichText::new(ch.to_string()).font(
                                                        FontId::new(12.0, FontFamily::Monospace),
                                                    ),
                                                );
                                            }
                                        });
                                    });

                                    if let Some(img) = self.image_cache.get(&filtered_fonts[row]) {
                                        img.show_scaled(ui, 0.5);
                                    } else {
                                        let buffer = Buffer::new((100, 12));
                                        let mut state = EditState::from_buffer(buffer);

                                        for ch in "HELLO".bytes() {
                                            let opt_size: Option<Size> = font.render(
                                                &mut state,
                                                ch,
                                            );
                                            if let Some(size) = opt_size {
                                                let mut pos = state.get_caret().get_position();
                                                pos.x += size.width + font.spaces;
                                                state.get_caret_mut().set_position(pos);
                                            }
                                        }
                                        let img = create_retained_image(state.get_buffer());
                                        img.show_scaled(ui, 0.5);
                                        self.image_cache.insert(filtered_fonts[row], img);
                                    }
                                });
                            }
                        },
                    );
                }
            });

            modal.buttons(ui, |ui| {
                if ui
                    .button(fl!(crate::LANGUAGE_LOADER, "select-font-dialog-select"))
                    .clicked()
                {
                    self.do_select = true;
                    result = true;
                }
                if ui
                    .button(fl!(crate::LANGUAGE_LOADER, "new-file-cancel"))
                    .clicked()
                {
                    result = true;
                }
            });
        });
        modal.open();
        result
    }

    fn should_commit(&self) -> bool {
        self.do_select
    }

    fn commit_self(&self, _window: &mut MainWindow) -> crate::TerminalResult<bool> {
        *self.selected_font_arc.lock().unwrap() = self.selected_font;
        Ok(true)
    }
}

pub fn create_retained_image(buf: &Buffer) -> RetainedImage {
    let (size, pixels) =
        buf.render_to_rgba(Rectangle::from(0, 0, buf.get_width(), buf.get_height()));
    RetainedImage::from_color_image(
        "buf_img",
        ColorImage::from_rgba_premultiplied([size.width as usize, size.height as usize], &pixels),
    )
}
