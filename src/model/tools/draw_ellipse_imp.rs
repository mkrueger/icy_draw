use eframe::egui;
use i18n_embed_fl::fl;
use icy_engine::{Rectangle, TextAttribute};

use crate::{AnsiEditor, Message};

use super::{
    brush_imp::draw_glyph, line_imp::set_half_block, DrawMode, Event, Plottable, Position,
    ScanLines, Tool,
};

pub struct DrawEllipseTool {
    pub draw_mode: DrawMode,

    pub use_fore: bool,
    pub use_back: bool,
    pub attr: TextAttribute,
    pub char_code: std::rc::Rc<std::cell::RefCell<char>>,
    pub font_page: usize,
}

impl Plottable for DrawEllipseTool {
    fn get_draw_mode(&self) -> DrawMode {
        self.draw_mode
    }
    fn get_use_fore(&self) -> bool {
        self.use_fore
    }
    fn get_use_back(&self) -> bool {
        self.use_back
    }
    fn get_char_code(&self) -> char {
        *self.char_code.borrow()
    }
}

impl Tool for DrawEllipseTool {
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage {
        &super::icons::ELLIPSE_OUTLINE_SVG
    }

    fn use_caret(&self) -> bool {
        false
    }
    fn use_selection(&self) -> bool {
        false
    }

    fn show_ui(
        &mut self,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        editor: &AnsiEditor,
    ) -> Option<Message> {
        let mut result = None;
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.use_fore, fl!(crate::LANGUAGE_LOADER, "tool-fg"))
                    .clicked()
                {
                    self.use_fore = !self.use_fore;
                }
                if ui
                    .selectable_label(self.use_back, fl!(crate::LANGUAGE_LOADER, "tool-bg"))
                    .clicked()
                {
                    self.use_back = !self.use_back;
                }
            });
        });

        ui.radio_value(
            &mut self.draw_mode,
            DrawMode::Line,
            fl!(crate::LANGUAGE_LOADER, "tool-line"),
        );
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.draw_mode,
                DrawMode::Char,
                fl!(crate::LANGUAGE_LOADER, "tool-character"),
            );

            result = draw_glyph(ui, editor, &self.char_code, self.font_page);
        });
        ui.radio_value(
            &mut self.draw_mode,
            DrawMode::Shade,
            fl!(crate::LANGUAGE_LOADER, "tool-shade"),
        );
        ui.radio_value(
            &mut self.draw_mode,
            DrawMode::Colorize,
            fl!(crate::LANGUAGE_LOADER, "tool-colorize"),
        );
        result
    }

    fn handle_drag(
        &mut self,
        editor: &mut AnsiEditor,
        mut start: Position,
        mut cur: Position,
    ) -> Event {
        if let Some(layer) = editor.buffer_view.lock().buf.get_overlay_layer() {
            layer.clear();
        }

        let mut lines = ScanLines::new(1);

        if self.draw_mode == DrawMode::Line {
            start.y *= 2;
            cur.y *= 2;
        }

        if start < cur {
            lines.add_ellipse(Rectangle::from_pt(start, cur));
        } else {
            lines.add_ellipse(Rectangle::from_pt(cur, start));
        }

        let col = editor
            .buffer_view
            .lock()
            .caret
            .get_attribute()
            .get_foreground();
        for rect in lines.outline() {
            for y in 0..(rect.size.height as i32) {
                for x in 0..(rect.size.width as i32) {
                    set_half_block(
                        editor,
                        Position::new(rect.start.x + x, rect.start.y + y),
                        col,
                    );
                }
            }
        }
        Event::None
    }

    fn handle_drag_end(
        &mut self,
        editor: &mut AnsiEditor,
        start: Position,
        cur: Position,
    ) -> Event {
        if start == cur {
            editor.buffer_view.lock().buf.remove_overlay();
        } else {
            editor.join_overlay();
        }
        Event::None
    }
}
