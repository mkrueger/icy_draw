use std::{cell::RefCell, fs, path::Path, rc::Rc, sync::Arc, time::Duration};

use crate::{
    add_child, model::Tool, AnsiEditor, DockingContainer, Document, DocumentOptions, FontEditor,
    ModalDialog, Tab, TabBehavior,
};
use eframe::{
    egui::{self, Response, SidePanel, TextStyle, Ui},
    epaint::pos2,
};
use glow::Context;
use hypex_ui::toasts;
use icy_engine::{BitFont, Buffer, Position};

pub struct MainWindow {
    pub hypex_ui: hypex_ui::HypexUi,
    pub toasts: toasts::Toasts,
    pub tree: egui_tiles::Tree<Tab>,

    pub tab_viewer: TabBehavior,
    pub gl: Arc<Context>,

    dialog_open: bool,
    modal_dialog: Option<Box<dyn ModalDialog>>,
    id: usize,

    pub left_panel: bool,
    pub right_panel: bool,
    pub bottom_panel: bool,
}

impl MainWindow {
    pub fn create_id(&mut self) -> usize {
        self.id += 1;
        self.id
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fnt = crate::model::font_imp::FontTool {
            selected_font: 0,
            fonts: Vec::new(),
            sizes: Vec::new(),
        };
        fnt.load_fonts();

        let tools: Vec<Box<dyn Tool>> = vec![
            Box::new(crate::model::click_imp::ClickTool {}),
            Box::new(crate::model::pencil_imp::PencilTool {
                use_back: true,
                use_fore: true,
                brush_type: crate::model::pencil_imp::PencilType::Shade,
                char_code: Rc::new(RefCell::new('\u{00B0}')),
                font_page: 0,
                last_pos: Position::default(),
            }),
            Box::new(crate::model::brush_imp::BrushTool {
                size: 3,
                use_back: true,
                use_fore: true,
                brush_type: crate::model::brush_imp::BrushType::Shade,
                char_code: Rc::new(RefCell::new('\u{00B0}')),
                font_page: 0,
            }),
            Box::new(crate::model::erase_imp::EraseTool {
                size: 3,
                brush_type: crate::model::erase_imp::EraseType::Shade,
            }),
            Box::new(crate::model::pipette_imp::PipetteTool {}),
            Box::new(crate::model::line_imp::LineTool {
                draw_mode: crate::model::DrawMode::Line,
                use_fore: true,
                use_back: true,
                attr: icy_engine::TextAttribute::default(),
                char_code: Rc::new(RefCell::new('\u{00B0}')),
                font_page: 0,
                old_pos: icy_engine::Position { x: 0, y: 0 },
            }),
            Box::new(crate::model::draw_rectangle_imp::DrawRectangleTool {
                draw_mode: crate::model::DrawMode::Line,
                use_fore: true,
                use_back: true,
                attr: icy_engine::TextAttribute::default(),
                char_code: Rc::new(RefCell::new('\u{00B0}')),
                font_page: 0,
            }),
            Box::new(
                crate::model::draw_rectangle_filled_imp::DrawRectangleFilledTool {
                    draw_mode: crate::model::DrawMode::Line,
                    use_fore: true,
                    use_back: true,
                    attr: icy_engine::TextAttribute::default(),
                    char_code: Rc::new(RefCell::new('\u{00B0}')),
                    font_page: 0,
                },
            ),
            Box::new(crate::model::draw_ellipse_imp::DrawEllipseTool {
                draw_mode: crate::model::DrawMode::Line,
                use_fore: true,
                use_back: true,
                attr: icy_engine::TextAttribute::default(),
                char_code: Rc::new(RefCell::new('\u{00B0}')),
                font_page: 0,
            }),
            Box::new(
                crate::model::draw_ellipse_filled_imp::DrawEllipseFilledTool {
                    draw_mode: crate::model::DrawMode::Line,
                    use_fore: true,
                    use_back: true,
                    attr: icy_engine::TextAttribute::default(),
                    char_code: Rc::new(RefCell::new('\u{00B0}')),
                    font_page: 0,
                },
            ),
            Box::new(crate::model::fill_imp::FillTool {
                use_fore: true,
                use_back: true,
                char_code: Rc::new(RefCell::new('\u{00B0}')),
                font_page: 0,
                fill_type: crate::model::fill_imp::FillType::Character,
                attr: icy_engine::TextAttribute::default(),
            }),
            Box::new(fnt),
            Box::new(crate::model::move_layer_imp::MoveLayer {
                pos: icy_engine::Position { x: 0, y: 0 },
            }),
        ];

        let hypex_ui = hypex_ui::HypexUi::load_and_apply(&cc.egui_ctx);

        MainWindow {
            hypex_ui,
            toasts: Default::default(),
            tab_viewer: TabBehavior {
                tools,
                selected_tool: 0,
                document_options: DocumentOptions {
                    scale: eframe::egui::Vec2::new(1.0, 1.0),
                },
            },
            tree: DockingContainer::default(),
            gl: cc.gl.clone().unwrap(),
            dialog_open: false,
            modal_dialog: None,
            id: 0,
            left_panel: true,
            right_panel: true,
            bottom_panel: false,
        }
    }

    pub fn open_file(&mut self, path: &Path) {
        let full_path = path.to_str().unwrap().to_string();

        if let Some(ext) = path.extension() {
            if let "psf" = ext.to_str().unwrap() {
                if let Ok(data) = fs::read(path) {
                    let file_name = path.file_name();
                    if file_name.is_none() {
                        return;
                    }
                    let file_name_str = file_name.unwrap().to_str().unwrap().to_string();
                    if let Ok(font) = BitFont::from_bytes(&file_name_str, &data) {
                        let id = self.create_id();
                        add_child(
                            &mut self.tree,
                            Some(full_path),
                            Box::new(FontEditor::new(font, id)),
                        );
                        return;
                    }
                }
            }
        }
        let buf = Buffer::load_buffer(path, true).unwrap();
        let id = self.create_id();
        let editor = AnsiEditor::new(&self.gl, id, buf);
        add_child(&mut self.tree, Some(full_path), Box::new(editor));
    }

    pub fn get_active_pane(&mut self) -> Option<&mut Tab> {
        let mut stack = vec![];

        if let Some(root) = self.tree.root {
            stack.push(root);
        }
        while !stack.is_empty() {
            let id = stack.pop().unwrap();

            match self.tree.tiles.get(id) {
                Some(egui_tiles::Tile::Pane(p)) => {
                    if let Some(egui_tiles::Tile::Pane(p)) = self.tree.tiles.get_mut(id) {
                        return Some(p);
                    } else {
                        return None;
                    }
                }
                Some(egui_tiles::Tile::Container(container)) => match container {
                    egui_tiles::Container::Tabs(tabs) => {
                        if let Some(active) = tabs.active {
                            stack.push(active);
                        }
                    }
                    egui_tiles::Container::Linear(l) => {
                        for child in l.children.iter() {
                            stack.push(*child);
                        }
                    }
                    egui_tiles::Container::Grid(g) => {
                        for child in g.children() {
                            stack.push(*child);
                        }
                    }
                },
                None => {}
            }
        }

        None
    }

    pub fn get_active_document_mut(&mut self) -> Option<&mut Box<dyn Document>> {
        if let Some(pane) = self.get_active_pane() {
            return Some(&mut pane.doc);
        }
        None
    }

    pub(crate) fn open_dialog<T: ModalDialog + 'static>(&mut self, dialog: T) {
        self.modal_dialog = Some(Box::new(dialog));
    }
}

pub fn button_with_shortcut(
    ui: &mut Ui,
    enabled: bool,
    label: impl Into<String>,
    shortcut: impl Into<String>,
) -> Response {
    let btn_re = ui.add_enabled(enabled, egui::Button::new(label.into()));
    let font_id = TextStyle::Body.resolve(ui.style());
    let color = ui.style().visuals.noninteractive().fg_stroke.color;

    let galley = ui.fonts(|f| {
        f.layout_job(egui::text::LayoutJob::simple_singleline(
            shortcut.into(),
            font_id,
            color,
        ))
    });
    ui.painter().galley(
        pos2(
            btn_re.rect.right() - galley.size().x - 2.0,
            btn_re.rect.top() + 2.0,
        ),
        galley,
    );
    btn_re
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        use egui::FontFamily::Proportional;
        use egui::FontId;
        use egui::TextStyle::*;

        let mut style: egui::Style = (*ctx.style()).clone();
        style.text_styles = [
            (Heading, FontId::new(30.0, Proportional)),
            (Body, FontId::new(14.0, Proportional)),
            (Monospace, FontId::new(20.0, Proportional)),
            (Button, FontId::new(20.0, Proportional)),
            (Small, FontId::new(16.0, Proportional)),
        ]
        .into();
        ctx.set_style(style);

        let msg = self.show_top_bar(ctx, frame);
        self.handle_message(msg);

        SidePanel::left("left_panel")
            .default_width(500.0)
            .frame(egui::Frame {
                fill: ctx.style().visuals.panel_fill,
                ..Default::default()
            })
            .show_animated(ctx, self.left_panel, |ui| {
                if let Some(doc) = self.get_active_document_mut() {
                    let doc = doc.get_ansi_editor_mut();
                    if let Some(editor) = doc {
                        ui.vertical_centered(|ui| {
                            ui.add(crate::palette_switcher(ctx, editor));
                        });
                        ui.add(crate::palette_editor_16(editor));
                    }
                }
                crate::add_tool_switcher(ctx, ui, self);
                /* TODO: Tool UI is not working yet
                let modal = if let Some(tool) = self.tab_viewer.tools.get_mut(self.tab_viewer.selected_tool) {
                    if let Some(doc) = self.get_active_document() {
                       let doc = doc.get_ansi_editor();
                        if let Some(editor) = doc {
                            let tool_result = tool.show_ui(ctx, ui, editor);
                            tool_result.modal_dialog
                        } else { None }
                    } else { None }
                } else { None };

                if modal.is_some() {
                    self.modal_dialog = modal;
                }*/
            });

        let panel_frame = egui::Frame {
            fill: ctx.style().visuals.panel_fill,
            inner_margin: hypex_ui::HypexUi::view_padding().into(),
            ..Default::default()
        };

        egui::SidePanel::right("right_panel")
            .frame(panel_frame)
            .show_animated(ctx, self.right_panel, |ui| {
                let message = if let Some(doc) = self.get_active_document_mut() {
                    let doc = doc.get_ansi_editor_mut();
                    if let Some(editor) = doc {
                        crate::ui::layer_view::show_layer_view(ctx, ui, editor)
                    } else {
                        None
                    }
                } else {
                    None
                };
                self.handle_message(message);

                // ui.add(crate::show_char_table(buffer_opt.clone()));
            });

        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: ctx.style().visuals.panel_fill,
                ..Default::default()
            })
            .show(ctx, |ui| {
                self.tree.ui(&mut self.tab_viewer, ui);
            });

        self.dialog_open = false;

        if self.modal_dialog.is_some() {
            self.dialog_open = true;
            if self.modal_dialog.as_mut().unwrap().show(ctx) {
                let modal_dialog = self.modal_dialog.take().unwrap();
                if modal_dialog.should_commit() {
                    if let Some(doc) = self.get_active_document_mut() {
                        let doc = doc.get_ansi_editor_mut();
                        if let Some(editor) = doc {
                            modal_dialog.commit(editor).unwrap();
                        }
                    }
                    modal_dialog.commit_self(self).unwrap();
                }
            }

            for (_, tile) in self.tree.tiles.iter_mut() {
                match tile {
                    egui_tiles::Tile::Pane(Tab { doc, .. }) => {
                        doc.set_enabled(!self.dialog_open);
                    }
                    _ => {}
                }
            }
        }
        ctx.request_repaint_after(Duration::from_millis(150));
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            for (_, tile) in self.tree.tiles.iter() {
                match tile {
                    egui_tiles::Tile::Pane(Tab { doc, .. }) => {
                        doc.destroy(gl);
                    }
                    _ => {}
                }
            }
        }
    }
}
