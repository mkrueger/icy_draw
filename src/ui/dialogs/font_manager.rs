use std::{path::Path, sync::Arc};

use eframe::{
    egui::{self, Button, Sense, SidePanel, TextStyle, WidgetText},
    epaint::{FontFamily, FontId, Rounding},
};
use egui_modal::Modal;
use i18n_embed_fl::fl;
use icy_engine::{BitFont, TextPane};
use icy_engine_egui::BufferView;

use crate::{AnsiEditor, Message, TerminalResult};

pub struct FontManager {
    selected: usize,
    replace_with: usize,
    do_select: bool,
    replace_font_path: String,
    buffer_view: Arc<eframe::epaint::mutex::Mutex<icy_engine_egui::BufferView>>,
}

impl FontManager {
    pub fn new(editor: &AnsiEditor) -> Self {
        Self {
            selected: 0,
            do_select: false,
            replace_with: 0,
            replace_font_path: String::new(),
            buffer_view: editor.buffer_view.clone(),
        }
    }
}

impl crate::ModalDialog for FontManager {
    fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut result = false;
        let modal = Modal::new(ctx, "manage_font_dialog");
        modal.show(|ui| {
            ui.set_height(420.);
            ui.set_width(800.);

            modal.title(ui, fl!(crate::LANGUAGE_LOADER, "manage-font-dialog-title"));
            modal.frame(ui, |ui| {
                SidePanel::left("new_file_side_panel")
                    .exact_width(280.0)
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        let row_height = 24.0;
                        ui.label("Used fonts");
                        egui::ScrollArea::vertical()
                            .id_source("bitfont_scroll_area")
                            .show(ui, |ui| {
                                for (i, font) in self.buffer_view.lock().get_buffer().font_iter() {
                                    let is_selected = *i == self.selected;

                                    let (id, rect) = ui
                                        .allocate_space([ui.available_width(), row_height].into());
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

                                    let font_id = FontId::new(12.0, FontFamily::Monospace);
                                    let text: WidgetText = format!("{i:-3}.").into();
                                    let galley = text.into_galley(
                                        ui,
                                        Some(false),
                                        f32::INFINITY,
                                        font_id.clone(),
                                    );
                                    let size = galley.size();
                                    let mut title_rect = rect;
                                    title_rect.set_left(title_rect.left() + 4.0);
                                    title_rect.set_top(title_rect.bottom() - size.y - 8.0);
                                    let text_color = if is_selected {
                                        ui.style().visuals.strong_text_color()
                                    } else {
                                        ui.style().visuals.text_color()
                                    };
                                    ui.painter().galley_with_color(
                                        egui::Align2::LEFT_TOP
                                            .align_size_within_rect(
                                                galley.size(),
                                                title_rect.shrink(4.0),
                                            )
                                            .min,
                                        galley.galley,
                                        text_color,
                                    );

                                    let font_id = TextStyle::Button.resolve(ui.style());
                                    let text: WidgetText = font.name.clone().into();
                                    let galley =
                                        text.into_galley(ui, Some(false), f32::INFINITY, font_id);
                                    let mut title_rect = rect;
                                    title_rect.set_left(title_rect.left() + 4.0 + size.x + 4.0);
                                    ui.painter().galley_with_color(
                                        egui::Align2::LEFT_TOP
                                            .align_size_within_rect(
                                                galley.size(),
                                                title_rect.shrink(4.0),
                                            )
                                            .min,
                                        galley.galley,
                                        text_color,
                                    );

                                    if response.clicked() {
                                        self.selected = *i;
                                    }
                                }
                            });
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    {
                        let lock = &self.buffer_view.lock();
                        if let Some(font) = lock.get_buffer().get_font(self.selected) {
                            ui.label(format!("Using slot {}", self.selected));
                            ui.label(format!("Size {}x{}", font.size.width, font.size.height));
                            let fonts = icy_engine::analyze_font_usage(lock.get_buffer());
                            if fonts.contains(&self.selected) {
                                ui.label("Font used.");
                            } else {
                                ui.label("No font usage.");
                            }
                        }
                    }
                    ui.horizontal(|ui| {
                        ui.label("Replace usage with slot:");
                        let mut tmp_str = self.replace_with.to_string();
                        ui.add(
                            egui::TextEdit::singleline(&mut tmp_str)
                                .desired_width(80.0)
                                .char_limit(4),
                        );
                        if let Ok(new_width) = tmp_str.parse::<usize>() {
                            self.replace_with = new_width;
                        } else {
                            self.replace_with = 0;
                        }
                        let enabled = self
                            .buffer_view
                            .lock()
                            .get_buffer()
                            .get_font(self.replace_with)
                            .is_some();
                        if ui.add_enabled(enabled, Button::new("Replace")).clicked() {
                            replace_font_usage(
                                &mut self.buffer_view.lock(),
                                self.selected,
                                self.replace_with,
                            );
                            self.selected = 0;
                            self.replace_with = 0;
                            self.do_select = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Load new font file for slot:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.replace_font_path)
                                .desired_width(200.0),
                        );
                        let enabled = Path::new(&self.replace_font_path).exists();
                        if ui.add_enabled(enabled, Button::new("Load")).clicked() {
                            match BitFont::load(Path::new(&self.replace_font_path)) {
                                Ok(font) => {
                                    self.buffer_view
                                        .lock()
                                        .get_buffer_mut()
                                        .set_font(self.selected, font);
                                }
                                Err(err) => {
                                    log::error!("{err}");
                                }
                            }
                        }
                    });

                    if ui
                        .add_enabled(self.selected > 0, Button::new("Remove"))
                        .clicked()
                    {
                        replace_font_usage(&mut self.buffer_view.lock(), self.selected, 0);
                        self.buffer_view
                            .lock()
                            .get_buffer_mut()
                            .remove_font(self.selected);
                        self.selected = 0;
                        self.replace_with = 0;
                        self.do_select = true;
                    }
                });
            });

            modal.buttons(ui, |ui| {
                if ui
                    .button(fl!(crate::LANGUAGE_LOADER, "new-file-ok"))
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

    fn commit(&self, _editor: &mut AnsiEditor) -> TerminalResult<Option<Message>> {
        Ok(None)
    }
}

fn replace_font_usage(buffer_view: &mut BufferView, from: usize, to: usize) {
    if buffer_view.get_caret().get_font_page() == from {
        buffer_view.get_caret_mut().set_font_page(to);
    }
    for layer in &mut buffer_view.get_buffer_mut().layers {
        for y in 0..layer.get_height() {
            for x in 0..layer.get_width() {
                let mut ch = layer.get_char((x, y));
                if ch.attribute.get_font_page() == from {
                    ch.attribute.set_font_page(to);
                    layer.set_char((x, y), ch);
                    assert!(layer.get_char((x, y)).get_font_page() == to);
                }
            }
        }
    }
}
