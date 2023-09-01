use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    add_child, model::Tool, AnsiEditor, BitFontEditor, BitFontSelector, CharFontEditor,
    DockingContainer, Document, DocumentOptions, ModalDialog, Tab, TabBehavior,
};
use eframe::{
    egui::{self, Response, SidePanel, TextStyle, Ui},
    epaint::pos2,
};
use glow::Context;
use i18n_embed_fl::fl;
use icy_engine::{BitFont, Buffer, Position, TheDrawFont};

pub struct MainWindow {
    pub tree: egui_tiles::Tree<Tab>,
    pub toasts: egui_notify::Toasts,

    pub tab_viewer: TabBehavior,
    pub gl: Arc<Context>,

    dialog_open: bool,
    modal_dialog: Option<Box<dyn ModalDialog>>,
    bitfont_selector: Option<BitFontSelector>,
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
            selected_font: Arc::new(Mutex::new(0)),
            fonts: Arc::new(Mutex::new(Vec::new())),
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
                last_pos: Position::default(),
            }),
            Box::new(crate::model::brush_imp::BrushTool {
                size: 3,
                use_back: true,
                use_fore: true,
                brush_type: crate::model::brush_imp::BrushType::Shade,
                char_code: Rc::new(RefCell::new('\u{00B0}')),
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
                old_pos: icy_engine::Position { x: 0, y: 0 },
            }),
            Box::new(crate::model::draw_rectangle_imp::DrawRectangleTool {
                draw_mode: crate::model::DrawMode::Line,
                use_fore: true,
                use_back: true,
                attr: icy_engine::TextAttribute::default(),
                char_code: Rc::new(RefCell::new('\u{00B0}')),
            }),
            Box::new(
                crate::model::draw_rectangle_filled_imp::DrawRectangleFilledTool {
                    draw_mode: crate::model::DrawMode::Line,
                    use_fore: true,
                    use_back: true,
                    attr: icy_engine::TextAttribute::default(),
                    char_code: Rc::new(RefCell::new('\u{00B0}')),
                },
            ),
            Box::new(crate::model::draw_ellipse_imp::DrawEllipseTool {
                draw_mode: crate::model::DrawMode::Line,
                use_fore: true,
                use_back: true,
                attr: icy_engine::TextAttribute::default(),
                char_code: Rc::new(RefCell::new('\u{00B0}')),
            }),
            Box::new(
                crate::model::draw_ellipse_filled_imp::DrawEllipseFilledTool {
                    draw_mode: crate::model::DrawMode::Line,
                    use_fore: true,
                    use_back: true,
                    attr: icy_engine::TextAttribute::default(),
                    char_code: Rc::new(RefCell::new('\u{00B0}')),
                },
            ),
            Box::new(crate::model::fill_imp::FillTool {
                use_fore: true,
                use_back: true,
                char_code: Rc::new(RefCell::new('\u{00B0}')),
                fill_type: crate::model::fill_imp::FillType::Character,
                attr: icy_engine::TextAttribute::default(),
            }),
            Box::new(fnt),
            Box::new(crate::model::move_layer_imp::MoveLayer {
                pos: icy_engine::Position { x: 0, y: 0 },
            }),
        ];

        MainWindow {
            tab_viewer: TabBehavior {
                tools: Arc::new(Mutex::new(tools)),
                selected_tool: 0,
                document_options: DocumentOptions {
                    scale: eframe::egui::Vec2::new(1.0, 1.0),
                },
            },
            toasts: egui_notify::Toasts::default(),
            tree: DockingContainer::default(),
            gl: cc.gl.clone().unwrap(),
            dialog_open: false,
            modal_dialog: None,
            id: 0,
            left_panel: true,
            right_panel: true,
            bottom_panel: false,
            bitfont_selector: Some(BitFontSelector::default()),
        }
    }

    pub fn open_file(&mut self, path: &Path) {
        let full_path = path.to_str().unwrap().to_string();

        if let Some(ext) = path.extension() {
            if "psf" == ext.to_str().unwrap().to_ascii_lowercase() {
                if let Ok(data) = fs::read(path) {
                    let file_name = path.file_name();
                    if file_name.is_none() {
                        return;
                    }
                    let file_name_str = file_name.unwrap().to_str().unwrap().to_string();
                    if let Ok(font) = BitFont::from_bytes(&file_name_str, &data) {
                        add_child(
                            &mut self.tree,
                            Some(full_path),
                            Box::new(BitFontEditor::new(font)),
                        );
                        return;
                    }
                }
            }

            if "tdf" == ext.to_str().unwrap().to_ascii_lowercase() {
                if let Ok(data) = fs::read(path) {
                    let file_name = path.file_name();
                    if file_name.is_none() {
                        return;
                    }
                    let file_name = PathBuf::from(file_name.unwrap());
                    if let Ok(fonts) = TheDrawFont::from_tdf_bytes(&data) {
                        let id = self.create_id();
                        add_child(
                            &mut self.tree,
                            Some(full_path),
                            Box::new(CharFontEditor::new(&self.gl, Some(file_name), id, fonts)),
                        );
                        return;
                    }
                }
            }
        }
        match Buffer::load_buffer(path, true) {
            Ok(buf) => {
                let id = self.create_id();
                let editor = AnsiEditor::new(&self.gl, id, buf);
                add_child(&mut self.tree, Some(full_path), Box::new(editor));
            }
            Err(err) => {
                log::error!("Error loading file: {}", err);
                self.toasts
                    .error(fl!(
                        crate::LANGUAGE_LOADER,
                        "error-load-file",
                        error = err.to_string()
                    ))
                    .set_duration(Some(Duration::from_secs(5)));
            }
        }
    }

    pub fn get_active_pane(&mut self) -> Option<&mut Tab> {
        let mut stack = vec![];

        if let Some(root) = self.tree.root {
            stack.push(root);
        }
        while let Some(id) = stack.pop() {
            match self.tree.tiles.get(id) {
                Some(egui_tiles::Tile::Pane(_)) => {
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

    pub fn enumerate_documents(&mut self, callback: fn(&mut Box<dyn Document>)) {
        let mut stack = vec![];

        if let Some(root) = self.tree.root {
            stack.push(root);
        }
        while let Some(id) = stack.pop() {
            match self.tree.tiles.get(id) {
                Some(egui_tiles::Tile::Pane(_)) => {
                    if let Some(egui_tiles::Tile::Pane(p)) = self.tree.tiles.get_mut(id) {
                        callback(&mut p.doc);
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
    }

    pub fn get_active_document_mut(&mut self) -> Option<&mut Box<dyn Document>> {
        if let Some(pane) = self.get_active_pane() {
            return Some(&mut pane.doc);
        }
        None
    }

    pub fn get_active_document(&mut self) -> Option<&Box<dyn Document>> {
        if let Some(pane) = self.get_active_pane() {
            return Some(&pane.doc);
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
                if let Some(tool) = self
                    .tab_viewer
                    .tools
                    .clone()
                    .lock()
                    .unwrap()
                    .get_mut(self.tab_viewer.selected_tool)
                {
                    if let Some(doc) = self.get_active_document_mut() {
                        let doc = doc.get_ansi_editor();
                        if let Some(editor) = doc {
                            let tool_result = tool.show_ui(ctx, ui, editor);
                            self.handle_message(tool_result);
                        }
                    }
                }
            });

        let panel_frame = egui::Frame {
            fill: ctx.style().visuals.panel_fill,
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
                let sel = self.bitfont_selector.take().unwrap();
                let message = if let Some(doc) = self.get_active_document_mut() {
                    let doc = doc.get_ansi_editor_mut();
                    if let Some(editor) = doc {
                        sel.show_ui(ctx, ui, editor)
                    } else {
                        None
                    }
                } else {
                    None
                };
                self.handle_message(message);

                self.bitfont_selector = Some(sel);

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
        }
        self.toasts.show(ctx);

        ctx.request_repaint_after(Duration::from_millis(150));
    }

    fn on_exit(&mut self, _gl: Option<&glow::Context>) {
        /* TODO

        self.enumerate_documents( move |doc| {
            doc.destroy(gl);
        });*/
    }
}
