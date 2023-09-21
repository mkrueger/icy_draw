use eframe::egui;
use i18n_embed_fl::fl;
use icy_engine::{Rectangle, TextAttribute};
use icy_engine_egui::TerminalCalc;

use crate::{AnsiEditor, Message};

use super::{
    brush_imp::draw_glyph, line_imp::set_half_block, plot_point, DrawMode, Plottable, Position,
    ScanLines, Tool,
};

pub struct DrawEllipseTool {
    pub draw_mode: DrawMode,

    pub use_fore: bool,
    pub use_back: bool,
    pub attr: TextAttribute,
    pub char_code: std::rc::Rc<std::cell::RefCell<char>>,
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
        editor_opt: Option<&AnsiEditor>,
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
            if let Some(editor) = editor_opt {
                result = draw_glyph(ui, editor, &self.char_code);
            }
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

    fn handle_hover(
        &mut self,
        _ui: &egui::Ui,
        response: egui::Response,
        _editor: &mut AnsiEditor,
        _cur: Position,
        _cur_abs: Position,
    ) -> egui::Response {
        response.on_hover_cursor(egui::CursorIcon::Crosshair)
    }

    fn handle_drag(
        &mut self,
        _ui: &egui::Ui,
        response: egui::Response,
        editor: &mut AnsiEditor,
        _calc: &TerminalCalc,
    ) -> egui::Response {
        editor.clear_overlay_layer();

        let mut lines = ScanLines::new(1);
        let mut start = editor.drag_pos.start;
        let mut cur = editor.drag_pos.cur;

        if self.draw_mode == DrawMode::Line {
            start = editor.drag_pos.start_half_block;
            cur = editor.half_block_click_pos;
        }

        if start < cur {
            lines.add_ellipse(Rectangle::from_pt(start, cur));
        } else {
            lines.add_ellipse(Rectangle::from_pt(cur, start));
        }

        let col = editor
            .buffer_view
            .lock()
            .get_caret()
            .get_attribute()
            .get_foreground();
        for rect in lines.outline() {
            for y in 0..rect.size.height {
                for x in 0..rect.size.width {
                    let pos = Position::new(rect.start.x + x, rect.start.y + y);
                    match self.draw_mode {
                        DrawMode::Line => {
                            set_half_block(editor, pos, col);
                        }
                        DrawMode::Char
                        | DrawMode::Shade
                        | DrawMode::Colorize
                        /*| DrawMode::Outline*/ => {
                            plot_point(editor, self, pos);
                        }
                    }
                }
            }
        }
        response
    }

    fn handle_drag_end(&mut self, editor: &mut AnsiEditor) -> Option<Message> {
        if editor.drag_pos.start == editor.drag_pos.cur {
            editor.buffer_view.lock().get_buffer_mut().remove_overlay();
        } else {
            editor.join_overlay(fl!(crate::LANGUAGE_LOADER, "undo-draw-ellipse"));
        }
        None
    }
}
