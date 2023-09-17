use std::path::Path;

use eframe::{
    egui::{self, Layout, Sense, SidePanel, Ui, WidgetText},
    emath::Align,
    epaint::{Color32, FontId, Pos2, Rect, Rounding, Vec2},
};
use egui_extras::RetainedImage;
use egui_modal::Modal;
use i18n_embed_fl::fl;
use icy_engine::{BitFont, Buffer, FontType, TheDrawFont};

use crate::{add_child, AnsiEditor, MainWindow, Message};

trait Template {
    fn image(&self) -> &RetainedImage;
    fn title(&self) -> String;
    fn description(&self) -> String;
    fn create_file(&self, window: &mut MainWindow) -> crate::TerminalResult<Option<Message>>;

    fn show_ui(&mut self, ui: &mut Ui);
}

pub struct NewFileDialog {
    pub create: bool,

    selected: usize,

    templates: Vec<Box<dyn Template>>,
}

struct AnsiTemplate {
    pub width: i32,
    pub height: i32,

    pub file_id: bool,
}

impl Template for AnsiTemplate {
    fn image(&self) -> &RetainedImage {
        &crate::ANSI_TEMPLATE_IMG
    }

    fn title(&self) -> String {
        if self.file_id {
            fl!(crate::LANGUAGE_LOADER, "new-file-template-file_id-title")
        } else {
            fl!(crate::LANGUAGE_LOADER, "new-file-template-ansi-title")
        }
    }

    fn description(&self) -> String {
        if self.file_id {
            fl!(
                crate::LANGUAGE_LOADER,
                "new-file-template-file_id-description"
            )
        } else {
            fl!(crate::LANGUAGE_LOADER, "new-file-template-ansi-description")
        }
    }

    fn show_ui(&mut self, ui: &mut Ui) {
        egui::Grid::new("some_unique_id")
            .num_columns(2)
            .spacing([4.0, 8.0])
            .show(ui, |ui| {
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(fl!(crate::LANGUAGE_LOADER, "new-file-width"));
                });
                let mut tmp_str = self.width.to_string();
                ui.add(egui::TextEdit::singleline(&mut tmp_str).char_limit(35));
                if let Ok(new_width) = tmp_str.parse::<i32>() {
                    self.width = new_width;
                }
                ui.end_row();

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(fl!(crate::LANGUAGE_LOADER, "new-file-height"));
                });
                let mut tmp_str = self.height.to_string();
                ui.add(egui::TextEdit::singleline(&mut tmp_str).char_limit(35));
                if let Ok(new_height) = tmp_str.parse::<i32>() {
                    self.height = new_height;
                }
                ui.end_row();
            });
    }

    fn create_file(&self, window: &mut MainWindow) -> crate::TerminalResult<Option<Message>> {
        let buf = Buffer::create((self.width, self.height));
        let id = window.create_id();
        let editor = AnsiEditor::new(&window.gl, id, buf);
        add_child(&mut window.document_tree, None, Box::new(editor));
        Ok(None)
    }
}

struct AnsiMationTemplate {}

impl Template for AnsiMationTemplate {
    fn image(&self) -> &RetainedImage {
        &crate::ANSIMATION_TEMPLATE_IMG
    }

    fn title(&self) -> String {
        fl!(crate::LANGUAGE_LOADER, "new-file-template-ansimation-title")
    }

    fn description(&self) -> String {
        fl!(
            crate::LANGUAGE_LOADER,
            "new-file-template-ansimation-description"
        )
    }

    fn create_file(&self, window: &mut MainWindow) -> crate::TerminalResult<Option<Message>> {
        let id = window.create_id();
        let txt = r#"local buf = new_buffer(80, 25)

for i=0,9,1 do
    buf:clear()
    buf.x = 10 + i * 5
    buf.y = 10
    buf:print("Hello World " .. cur_frame)
    next_frame(buf)
end"#;
        let editor = crate::AnimationEditor::new(&window.gl, id, Path::new("."), txt.into());
        add_child(&mut window.document_tree, None, Box::new(editor));
        Ok(None)
    }

    fn show_ui(&mut self, ui: &mut Ui) {
        ui.label(fl!(
            crate::LANGUAGE_LOADER,
            "new-file-template-ansimation-ui-label"
        ));
        ui.hyperlink("https://github.com/mkrueger/icy_draw/blob/main/doc/lua_api.md");
    }
}

struct BitFontTemplate {}

impl Template for BitFontTemplate {
    fn image(&self) -> &RetainedImage {
        &crate::BITFONT_TEMPLATE_IMG
    }

    fn title(&self) -> String {
        fl!(crate::LANGUAGE_LOADER, "new-file-template-bit_font-title")
    }

    fn description(&self) -> String {
        fl!(
            crate::LANGUAGE_LOADER,
            "new-file-template-bit_font-description"
        )
    }

    fn create_file(&self, window: &mut MainWindow) -> crate::TerminalResult<Option<Message>> {
        let id = window.create_id();
        let editor = crate::BitFontEditor::new(&window.gl, id, BitFont::default());
        add_child(&mut window.document_tree, None, Box::new(editor));
        Ok(None)
    }
    fn show_ui(&mut self, ui: &mut Ui) {
        ui.label(fl!(
            crate::LANGUAGE_LOADER,
            "new-file-template-bitfont-ui-label"
        ));
    }
}

struct TdfFontTemplate {
    pub font_type: FontType,
}

impl Template for TdfFontTemplate {
    fn image(&self) -> &RetainedImage {
        match self.font_type {
            FontType::Outline => &crate::OUTLINEFONT_TEMPLATE_IMG,
            FontType::Block => &crate::BLOCKFONT_TEMPLATE_IMG,
            FontType::Color => &crate::COLORFONT_TEMPLATE_IMG,
        }
    }

    fn title(&self) -> String {
        match self.font_type {
            FontType::Outline => fl!(
                crate::LANGUAGE_LOADER,
                "new-file-template-outline_font-title"
            ),
            FontType::Block => fl!(crate::LANGUAGE_LOADER, "new-file-template-block_font-title"),
            FontType::Color => fl!(crate::LANGUAGE_LOADER, "new-file-template-color_font-title"),
        }
    }

    fn description(&self) -> String {
        match self.font_type {
            FontType::Outline => fl!(
                crate::LANGUAGE_LOADER,
                "new-file-template-outline_font-description"
            ),
            FontType::Block => fl!(
                crate::LANGUAGE_LOADER,
                "new-file-template-block_font-description"
            ),
            FontType::Color => fl!(
                crate::LANGUAGE_LOADER,
                "new-file-template-color_font-description"
            ),
        }
    }

    fn create_file(&self, window: &mut MainWindow) -> crate::TerminalResult<Option<Message>> {
        let id = window.create_id();
        let fonts = vec![TheDrawFont::new(self.title(), self.font_type, 1)];
        let editor = crate::CharFontEditor::new(&window.gl, id, fonts);
        add_child(&mut window.document_tree, None, Box::new(editor));
        Ok(None)
    }

    fn show_ui(&mut self, ui: &mut Ui) {
        ui.label(fl!(
            crate::LANGUAGE_LOADER,
            "new-file-template-thedraw-ui-label"
        ));
        ui.hyperlink("http://www.roysac.com/thedrawfonts-tdf.html");
    }
}

impl Default for NewFileDialog {
    fn default() -> Self {
        let templates: Vec<Box<dyn Template>> = vec![
            Box::new(AnsiTemplate {
                width: 80,
                height: 25,
                file_id: false,
            }),
            Box::new(AnsiTemplate {
                width: 44,
                height: 25,
                file_id: true,
            }),
            Box::new(AnsiMationTemplate {}),
            Box::new(BitFontTemplate {}),
            Box::new(TdfFontTemplate {
                font_type: FontType::Color,
            }),
            Box::new(TdfFontTemplate {
                font_type: FontType::Block,
            }),
            Box::new(TdfFontTemplate {
                font_type: FontType::Outline,
            }),
        ];

        Self {
            create: false,
            templates,
            selected: 0,
        }
    }
}

impl crate::ModalDialog for NewFileDialog {
    fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut result = false;
        let modal = Modal::new(ctx, "new_file_dialog");

        modal.show(|ui| {
            ui.set_height(420.);
            ui.set_width(800.);

            modal.title(ui, fl!(crate::LANGUAGE_LOADER, "new-file-title"));

            modal.frame(ui, |ui| {
                SidePanel::left("new_file_side_panel")
                    .exact_width(280.0)
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        let row_height = 58.0;
                        egui::ScrollArea::vertical()
                            .id_source("bitfont_scroll_area")
                            .show(ui, |ui| {
                                for (i, template) in self.templates.iter().enumerate() {
                                    let is_selected = i == self.selected;

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
                                    let image = template.image();

                                    let r = Rect::from_min_size(
                                        Pos2::new(
                                            (rect.left() + 4.0).floor(),
                                            (rect.top() + 4.0).floor(),
                                        ),
                                        Vec2::new(32.0, 32.0),
                                    );

                                    ui.painter().image(
                                        image.texture_id(ui.ctx()),
                                        r,
                                        Rect::from_min_max(
                                            Pos2::new(0.0, 0.0),
                                            Pos2::new(1.0, 1.0),
                                        ),
                                        Color32::WHITE,
                                    );

                                    let font_id =
                                        FontId::new(20.0, eframe::epaint::FontFamily::Proportional);
                                    let text: WidgetText = template.title().into();
                                    let galley =
                                        text.into_galley(ui, Some(false), f32::INFINITY, font_id);
                                    let mut title_rect = rect;
                                    title_rect.set_left(title_rect.left() + 40.0);
                                    ui.painter().galley_with_color(
                                        egui::Align2::LEFT_TOP
                                            .align_size_within_rect(
                                                galley.size(),
                                                title_rect.shrink(4.0),
                                            )
                                            .min,
                                        galley.galley,
                                        ui.style().visuals.strong_text_color(),
                                    );

                                    let font_id =
                                        FontId::new(14.0, eframe::epaint::FontFamily::Proportional);
                                    let text: WidgetText = template.description().into();
                                    let galley =
                                        text.into_galley(ui, Some(false), f32::INFINITY, font_id);
                                    let mut descr_rect = rect;
                                    descr_rect.set_top(descr_rect.top() + 34.0);
                                    ui.painter().galley_with_color(
                                        egui::Align2::LEFT_TOP
                                            .align_size_within_rect(
                                                galley.size(),
                                                descr_rect.shrink(4.0),
                                            )
                                            .min,
                                        galley.galley,
                                        ui.style().visuals.text_color(),
                                    );

                                    if response.clicked() {
                                        self.selected = i;
                                    }
                                    if response.double_clicked() {
                                        self.selected = i;
                                        self.create = true;
                                        result = true;
                                    }
                                }
                            });
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.label(self.templates[self.selected].description());
                    ui.separator();
                    self.templates[self.selected].show_ui(ui);
                });
            });
            ui.separator();
            ui.add_space(4.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .button(fl!(crate::LANGUAGE_LOADER, "new-file-create"))
                    .clicked()
                {
                    self.create = true;
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
        self.create
    }

    fn commit_self(&self, window: &mut MainWindow) -> crate::TerminalResult<Option<Message>> {
        self.templates[self.selected].create_file(window)
    }
}
