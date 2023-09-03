use std::{fs, path::PathBuf, sync::Arc};

use eframe::egui::{self, RichText};
use icy_engine::{
    editor::UndoState, BitFont, Buffer, EngineResult, Layer, Size, TextAttribute, TheDrawFont,
    UPosition,
};

use crate::{
    model::Tool, AnsiEditor, BitFontEditor, ClipboardHandler, Document, DocumentOptions,
    DrawGlyphStyle, TerminalResult,
};

pub struct CharFontEditor {
    font: BitFont,
    selected_char_opt: Option<char>,
    file_name: Option<PathBuf>,
    ansi_editor: AnsiEditor,
    is_dirty: bool,
    selected_font: usize,
    fonts: Vec<TheDrawFont>,
}

impl ClipboardHandler for CharFontEditor {}

impl UndoState for CharFontEditor {
    fn undo_description(&self) -> Option<String> {
        todo!()
    }

    fn can_undo(&self) -> bool {
        false
    }

    fn undo(&mut self) -> EngineResult<()> {
        todo!()
    }

    fn redo_description(&self) -> Option<String> {
        todo!()
    }

    fn can_redo(&self) -> bool {
        false
    }

    fn redo(&mut self) -> EngineResult<()> {
        todo!()
    }
}

impl Document for CharFontEditor {
    fn get_title(&self) -> String {
        if let Some(file_name) = &self.file_name {
            file_name.file_name().unwrap().to_str().unwrap().to_string()
        } else {
            "Untitled".to_string()
        }
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn save(&mut self, file_name: &str) -> TerminalResult<()> {
        let file = PathBuf::from(file_name);
        self.file_name = Some(file);
        let bytes = TheDrawFont::create_font_bundle(&self.fonts)?;
        fs::write(file_name, bytes)?;
        self.is_dirty = false;
        Ok(())
    }

    fn show_ui(
        &mut self,
        ui: &mut egui::Ui,
        cur_tool: &mut Box<dyn Tool>,
        options: &DocumentOptions,
    ) {
        egui::ComboBox::from_id_source("combobox1")
            .selected_text(RichText::new(
                self.fonts[self.selected_font].name.to_string(),
            ))
            .show_ui(ui, |ui| {
                let mut changed = false;
                for (i, font) in self.fonts.iter().enumerate() {
                    if ui
                        .selectable_value(&mut self.selected_font, i, &font.name)
                        .clicked()
                    {
                        changed = true;
                        self.selected_font = i;
                    }
                }
                if changed {
                    self.show_selected_char();
                }
            });

        self.show_char_selector(ui);
        self.ansi_editor.show_ui(ui, cur_tool, options);
    }

    fn get_ansi_editor_mut(&mut self) -> Option<&mut AnsiEditor> {
        self.ansi_editor.get_ansi_editor_mut()
    }

    fn get_ansi_editor(&self) -> Option<&AnsiEditor> {
        self.ansi_editor.get_ansi_editor()
    }

    fn destroy(&self, gl: &glow::Context) {
        self.ansi_editor.destroy(gl);
    }
}

impl CharFontEditor {
    pub fn new(
        gl: &Arc<glow::Context>,
        file_name: Option<PathBuf>,
        id: usize,
        fonts: Vec<TheDrawFont>,
    ) -> Self {
        let mut buffer = Buffer::new(Size::new(30, 12));
        let mut new_layer = Layer::new("edit layer", Size::new(30, 12));
        new_layer.has_alpha_channel = true;
        buffer.layers.insert(0, new_layer);

        let ansi_editor = AnsiEditor::new(gl, id, buffer);

        Self {
            font: BitFont::default(),
            file_name,
            ansi_editor,
            is_dirty: false,
            selected_char_opt: None,
            fonts,
            selected_font: 0,
        }
    }

    pub fn show_char_selector(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for i in b'!'..=b'~' {
                    let ch = unsafe { char::from_u32_unchecked(i as u32) };
                    let mut style = DrawGlyphStyle::Normal;
                    if !self.fonts[self.selected_font].has_char(i) {
                        style = DrawGlyphStyle::GrayOut
                    }
                    if let Some(ch2) = self.selected_char_opt {
                        if ch == ch2 {
                            style = DrawGlyphStyle::Selected
                        }
                    }
                    let response = BitFontEditor::draw_glyph(ui, &self.font, style, ch);
                    if response.clicked() {
                        self.selected_char_opt = Some(ch);
                        self.show_selected_char();
                    }
                }
            });
        });
    }

    fn show_selected_char(&mut self) {
        let lock = &mut self.ansi_editor.buffer_view.lock();
        let buffer = &mut lock.get_buffer_mut();
        buffer.layers[0].clear();

        if let Some(ch) = self.selected_char_opt {
            let font = &self.fonts[self.selected_font];
            font.render(
                buffer,
                0,
                UPosition::default(),
                TextAttribute::default(),
                0,
                ch as u8,
            );
        }
    }
}
