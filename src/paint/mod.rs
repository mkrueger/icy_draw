use eframe::egui;
use i18n_embed_fl::fl;
use icy_engine::{AttributedChar, Position, TextAttribute, TextPane, TheDrawFont};
use icy_engine_egui::BufferView;

use crate::{model::brush_imp::draw_glyph, AnsiEditor, Message};

use self::half_block::get_half_block;

mod half_block;
mod rectangle;
pub use rectangle::*;
mod line;
pub use line::*;
mod ellipse;
pub use ellipse::*;

#[derive(Clone, Debug, PartialEq)]
pub enum BrushMode {
    Block,
    HalfBlock,
    Outline,
    Char(std::rc::Rc<std::cell::RefCell<char>>),
    Shade,
    Colorize,
}

impl BrushMode {
    pub fn show_ui(
        &mut self,
        ui: &mut egui::Ui,
        editor_opt: Option<&AnsiEditor>,
        char_code: std::rc::Rc<std::cell::RefCell<char>>,
        show_outline:bool
    ) -> Option<Message> {
        let mut msg = None;
        ui.radio_value(
            self,
            BrushMode::HalfBlock,
            fl!(crate::LANGUAGE_LOADER, "tool-half-block"),
        );
        ui.radio_value(
            self,
            BrushMode::Block,
            fl!(crate::LANGUAGE_LOADER, "tool-full-block"),
        );

        if show_outline { 
            
            ui.horizontal(|ui| { 
                ui.radio_value(
                    self,
                    BrushMode::Outline,
                    fl!(crate::LANGUAGE_LOADER, "tool-outline"),
                );
                if ui
                .button(fl!(
                    crate::LANGUAGE_LOADER,
                    "font_tool_select_outline_button"
                ))
                .clicked()
                {
                    msg = Some(Message::ShowOutlineDialog);
                }
            });
        }

        ui.horizontal(|ui| {
            ui.radio_value(
                self,
                BrushMode::Char(char_code.clone()),
                fl!(crate::LANGUAGE_LOADER, "tool-character"),
            );
            if let Some(editor) = editor_opt {
                draw_glyph(ui, editor, &char_code);
            }
        });
        ui.radio_value(
            self,
            BrushMode::Shade,
            fl!(crate::LANGUAGE_LOADER, "tool-shade"),
        );
        ui.radio_value(
            self,
            BrushMode::Colorize,
            fl!(crate::LANGUAGE_LOADER, "tool-colorize"),
        );

        msg
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorMode {
    None,
    UseFg,
    UseBg,
    Both,
}

impl ColorMode {
    pub fn use_fore(&self) -> bool {
        matches!(self, ColorMode::UseFg | ColorMode::Both)
    }

    pub fn use_back(&self) -> bool {
        matches!(self, ColorMode::UseBg | ColorMode::Both)
    }

    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                let mut use_fore = self.use_fore();
                let mut use_back = self.use_back();

                if ui
                    .selectable_label(use_fore, fl!(crate::LANGUAGE_LOADER, "tool-fg"))
                    .clicked()
                {
                    use_fore = !use_fore;
                }
                if ui
                    .selectable_label(use_back, fl!(crate::LANGUAGE_LOADER, "tool-bg"))
                    .clicked()
                {
                    use_back = !use_back;
                }

                if use_fore && use_back {
                    *self = ColorMode::Both;
                } else if use_fore {
                    *self = ColorMode::UseFg;
                } else if use_back {
                    *self = ColorMode::UseBg;
                } else {
                    *self = ColorMode::None;
                }
            });
        });
    }
}

pub fn plot_point(
    buffer_view: &mut BufferView,
    pos: impl Into<Position>,
    mut mode: BrushMode,
    color_mode: ColorMode,
    point_role: PointRole,
) {
    let pos = pos.into();
    let text_pos = Position::new(pos.x, pos.y / 2);
    let mut ch = if let Some(layer) = buffer_view.get_edit_state().get_cur_layer() {
        layer.get_char(text_pos)
    } else {
        return;
    };

    let editor_attr = buffer_view.get_caret().get_attribute();
    let mut attribute = ch.attribute;
    if !ch.is_visible() {
        attribute = TextAttribute::default();
    }
    attribute.set_font_page(editor_attr.get_font_page());
    if color_mode.use_fore() {
        attribute.set_foreground(editor_attr.get_foreground());
    }
    if color_mode.use_back() {
        attribute.set_background(editor_attr.get_background());
    }

    let Some(layer) = buffer_view.get_buffer_mut().get_overlay_layer() else {
        return;
    };
    let overlay_ch = layer.get_char(text_pos);
    if overlay_ch.is_visible() {
        ch = overlay_ch;
    }

    if matches!(mode, BrushMode::HalfBlock) && matches!(point_role, PointRole::Fill) {
        mode = BrushMode::Block;
    }

    match mode {
        BrushMode::HalfBlock => {
            layer.set_char(
                text_pos,
                get_half_block(ch, pos, attribute.get_foreground()),
            );
        }
        BrushMode::Block => {
            layer.set_char(text_pos, AttributedChar::new(219 as char, attribute));
        }
        BrushMode::Char(ch) => {
            layer.set_char(text_pos, AttributedChar::new(*ch.borrow(), attribute));
        }

        BrushMode::Outline => {
            if overlay_ch.is_visible() {
                return;
            }
            let ch = match point_role {
                PointRole::NWCorner => 'E',
                PointRole::NECorner => 'F',
                PointRole::SWCorner => 'K',
                PointRole::SECorner => 'L',
                PointRole::LeftSide => 'C',
                PointRole::RightSide => 'D',
                PointRole::TopSide => 'A',
                PointRole::BottomSide => 'B',
                _ => {
                    return;
                }
            };
            let outline_style = crate::Settings::get_font_outline_style();
            layer.set_char(text_pos, AttributedChar::new(TheDrawFont::transform_outline(outline_style, ch as u8) as char, attribute));
        }

        BrushMode::Shade => {
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
            layer.set_char(text_pos, AttributedChar::new(char_code, attribute));
        }
        BrushMode::Colorize => {
            layer.set_char(text_pos, AttributedChar::new(ch.ch, attribute));
        }
    }
}

pub static SHADE_GRADIENT: [char; 4] = ['\u{00B0}', '\u{00B1}', '\u{00B2}', '\u{00DB}'];
pub enum PointRole {
    NWCorner,
    NECorner,
    SWCorner,
    SECorner,
    LeftSide,
    RightSide,
    TopSide,
    BottomSide,
    Fill,
    Line,
}
