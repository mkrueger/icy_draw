pub mod brush_imp;
pub mod click_imp;
pub mod draw_ellipse_filled_imp;
pub mod draw_ellipse_imp;
pub mod draw_rectangle_filled_imp;
pub mod draw_rectangle_imp;
pub mod erase_imp;
pub mod fill_imp;
pub mod flip_imp;
pub mod font_imp;
pub mod line_imp;
pub mod move_layer_imp;
pub mod paste_tool;
pub mod pencil_imp;
pub mod pipette_imp;
pub mod select_imp;

mod icons;

use eframe::egui::{self, Response};
use egui_extras::RetainedImage;
use icy_engine::{AttributedChar, Position, TextAttribute};
use icy_engine_egui::TerminalCalc;
pub use scan_lines::*;

use crate::{AnsiEditor, Event, Message, Settings};
pub mod scan_lines;

#[derive(Copy, Clone, Debug)]
pub enum MKey {
    Character(u16),
    Down,
    Up,
    Left,
    Right,
    PageDown,
    PageUp,
    Home,
    End,
    Return,
    Delete,
    Insert,
    Backspace,
    Tab,
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

#[derive(Copy, Clone, Debug)]
pub enum MModifiers {
    None,
    Shift,
    Alt,
    Control,
}

impl MModifiers {
    pub fn is_shift(self) -> bool {
        matches!(self, MModifiers::Shift)
    }

    pub fn is_alt(self) -> bool {
        matches!(self, MModifiers::Alt)
    }

    pub fn is_control(self) -> bool {
        matches!(self, MModifiers::Control)
    }
}

#[derive(Default, Clone, Copy)]
pub struct DragPos {
    pub start_abs: Position,
    pub cur_abs: Position,
    pub start: Position,
    pub cur: Position,
}

pub trait Tool {
    fn get_icon_name(&self) -> &'static RetainedImage;

    fn use_caret(&self) -> bool {
        true
    }

    fn is_visible(&self) -> bool {
        true
    }

    fn is_exclusive(&self) -> bool {
        false
    }

    fn use_selection(&self) -> bool {
        true
    }

    fn show_ui(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        editor_opt: Option<&AnsiEditor>,
    ) -> Option<Message>;

    fn handle_key(&mut self, _editor: &mut AnsiEditor, _key: MKey, _modifier: MModifiers) -> Event {
        Event::None
    }

    fn handle_click(
        &mut self,
        _editor: &mut AnsiEditor,
        _button: i32,
        _pos: Position,
        _pos_abs: Position,
        _response: &Response,
    ) -> Event {
        Event::None
    }

    fn handle_drag_begin(&mut self, _editor: &mut AnsiEditor, _response: &egui::Response) -> Event {
        Event::None
    }

    fn handle_drag(
        &mut self,
        _ui: &egui::Ui,
        response: Response,
        _editor: &mut AnsiEditor,
        _calc: &TerminalCalc,
    ) -> Response {
        response
    }

    fn handle_hover(
        &mut self,
        _ui: &egui::Ui,
        response: Response,
        _editor: &mut AnsiEditor,
        _cur: Position,
        _cur_abs: Position,
    ) -> Response {
        response
    }

    fn handle_drag_end(&mut self, _editor: &mut AnsiEditor) -> Event {
        Event::None
    }
}

fn handle_outline_insertion(editor: &mut AnsiEditor, modifier: MModifiers, outline: usize) {
    if let MModifiers::Control = modifier {
        Settings::set_character_set(outline);
        return;
    }

    if outline < 5 {
        if let MModifiers::Shift = modifier {
            Settings::set_character_set(10 + outline);
            return;
        }
    }
    editor.buffer_view.lock().clear_selection();
    let ch = editor.get_outline_char_code(outline);
    if let Ok(ch) = ch {
        editor.type_key(unsafe { char::from_u32_unchecked(ch as u32) });
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrawMode {
    Line,
    Char,
    Shade,
    Colorize,
}

trait Plottable {
    fn get_draw_mode(&self) -> DrawMode;

    fn get_use_fore(&self) -> bool;
    fn get_use_back(&self) -> bool;
    fn get_char_code(&self) -> char;
}

fn plot_point(editor: &AnsiEditor, tool: &dyn Plottable, pos: Position) {
    let ch = editor.get_char_from_cur_layer(pos);
    let editor_attr = editor.buffer_view.lock().get_caret().get_attribute();
    let mut attribute = ch.attribute;
    if !ch.is_visible() {
        attribute = TextAttribute::default();
    }
    if tool.get_use_back() {
        attribute.set_background(editor_attr.get_background());
    }
    if tool.get_use_fore() {
        attribute.set_is_bold(false);
        attribute.set_foreground(editor_attr.get_foreground());
    }

    attribute.set_font_page(editor_attr.get_font_page());

    match tool.get_draw_mode() {
        DrawMode::Line => {
            if let Some(layer) = editor
                .buffer_view
                .lock()
                .get_buffer_mut()
                .get_overlay_layer()
            {
                layer.set_char(
                    pos,
                    AttributedChar::new(unsafe { char::from_u32_unchecked(219) }, attribute),
                );
            }
        }
        DrawMode::Char => {
            if let Some(layer) = editor
                .buffer_view
                .lock()
                .get_buffer_mut()
                .get_overlay_layer()
            {
                layer.set_char(pos, AttributedChar::new(tool.get_char_code(), attribute));
            }
        }
        DrawMode::Shade => {
            let mut char_code = SHADE_GRADIENT[0];
            if ch.ch == SHADE_GRADIENT[SHADE_GRADIENT.len() - 1] {
                char_code = SHADE_GRADIENT[SHADE_GRADIENT.len() - 1];
            } else {
                for i in 0..SHADE_GRADIENT.len() - 1 {
                    if ch.ch == SHADE_GRADIENT[i] {
                        char_code = SHADE_GRADIENT[i + 1];
                        break;
                    }
                }
            }
            if let Some(layer) = editor
                .buffer_view
                .lock()
                .get_buffer_mut()
                .get_overlay_layer()
            {
                layer.set_char(pos, AttributedChar::new(char_code, attribute));
            }
        }
        DrawMode::Colorize => {
            if let Some(layer) = editor
                .buffer_view
                .lock()
                .get_buffer_mut()
                .get_overlay_layer()
            {
                layer.set_char(pos, AttributedChar::new(ch.ch, attribute));
            }
        }
    }
}

pub static SHADE_GRADIENT: [char; 4] = ['\u{00B0}', '\u{00B1}', '\u{00B2}', '\u{00DB}'];
