use eframe::egui;
use i18n_embed_fl::fl;
use icy_engine::{Position, Rectangle, TextAttribute};

use crate::AnsiEditor;

use super::{
    brush_imp::draw_glyph, plot_point, DrawMode, Event, Plottable, ScanLines, Tool, ToolUiResult,
};

pub struct DrawRectangleFilledTool {
    pub draw_mode: DrawMode,

    pub use_fore: bool,
    pub use_back: bool,
    pub attr: TextAttribute,
    pub char_code: std::rc::Rc<std::cell::RefCell<char>>,
    pub font_page: usize,
}

impl Plottable for DrawRectangleFilledTool {
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

impl Tool for DrawRectangleFilledTool {
    fn get_icon_name(&self) -> &'static egui_extras::RetainedImage {
        &super::icons::RECTANGLE_FILLED_SVG
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
        editor: &mut AnsiEditor,
    ) -> ToolUiResult {
        let mut result = ToolUiResult::default();
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
            fl!(crate::LANGUAGE_LOADER, "tool-solid"),
        );
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.draw_mode,
                DrawMode::Char,
                fl!(crate::LANGUAGE_LOADER, "tool-character"),
            );

            draw_glyph(ui, editor, &mut result, &self.char_code, self.font_page);
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

    fn handle_drag(&mut self, editor: &mut AnsiEditor, start: Position, cur: Position) -> Event {
        if let Some(layer) = editor.buffer_view.lock().buf.get_overlay_layer() {
            layer.clear();
        }

        let mut lines = ScanLines::new(1);
        lines.add_rectangle(Rectangle::from_pt(start, cur));

        let draw = move |rect: Rectangle| {
            println!("draw rect: {:?}", rect);
            for y in 0..rect.size.height {
                for x in 0..rect.size.width {
                    plot_point(
                        editor,
                        self,
                        Position::new(rect.start.x + x, rect.start.y + y),
                    );
                }
            }
        };
        lines.fill(draw);

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
