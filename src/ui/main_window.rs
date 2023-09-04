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
    CharTableToolWindow, Document, DocumentBehavior, DocumentOptions, DocumentTab, LayerToolWindow,
    Message, ModalDialog, ToolBehavior, ToolTab, TopBar,
};
use eframe::{
    egui::{self, Response, SidePanel, TextStyle, Ui},
    epaint::{pos2, FontId},
};
use glow::Context;
use i18n_embed_fl::fl;
use icy_engine::{BitFont, Buffer, EngineResult, Position, TheDrawFont};

pub struct MainWindow {
    pub document_tree: egui_tiles::Tree<DocumentTab>,
    pub tool_tree: egui_tiles::Tree<ToolTab>,
    pub toasts: egui_notify::Toasts,

    pub document_behavior: DocumentBehavior,
    pub tool_behavior: ToolBehavior,
    pub gl: Arc<Context>,

    dialog_open: bool,
    modal_dialog: Option<Box<dyn ModalDialog>>,
    id: usize,
    palette_mode: usize,
    pub top_bar: TopBar,
    pub left_panel: bool,
    pub right_panel: bool,
    pub bottom_panel: bool,
}

pub const PASTE_TOOL: usize = 0;
pub const FIRST_TOOL: usize = 1;

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
            Box::new(crate::model::paste_tool::PasteTool::default()),
            Box::new(crate::model::click_imp::ClickTool {}),
            Box::new(crate::model::pencil_imp::PencilTool {
                use_back: true,
                use_fore: true,
                undo_op: None,
                brush_type: crate::model::pencil_imp::PencilType::Shade,
                char_code: Rc::new(RefCell::new('\u{00B0}')),
                last_pos: Position::default(),
            }),
            Box::new(crate::model::brush_imp::BrushTool {
                size: 3,
                use_back: true,
                use_fore: true,
                undo_op: None,
                brush_type: crate::model::brush_imp::BrushType::Shade,
                char_code: Rc::new(RefCell::new('\u{00B0}')),
            }),
            Box::new(crate::model::erase_imp::EraseTool {
                size: 3,
                brush_type: crate::model::erase_imp::EraseType::Shade,
                undo_op: None,
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
            Box::new(crate::model::move_layer_imp::MoveLayer::default()),
        ];

        let ctx: &egui::Context = &cc.egui_ctx;

        // try to detect dark vs light mode from the host system; default to dark
        ctx.set_visuals(if dark_light::detect() == dark_light::Mode::Light {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        });

        let mut style: egui::Style = (*ctx.style()).clone();
        style.spacing.window_margin = egui::Margin::same(8.0);
        use egui::FontFamily::Proportional;
        use egui::TextStyle::{Body, Button, Heading, Monospace, Small};
        style.text_styles = [
            (Heading, FontId::new(24.0, Proportional)),
            (Body, FontId::new(18.0, Proportional)),
            (Monospace, FontId::new(18.0, egui::FontFamily::Monospace)),
            (Button, FontId::new(18.0, Proportional)),
            (Small, FontId::new(14.0, Proportional)),
        ]
        .into();
        ctx.set_style(style);

        let mut tool_tree = egui_tiles::Tree::<ToolTab>::empty("tool_tree");
        let layers = tool_tree
            .tiles
            .insert_pane(ToolTab::new(LayerToolWindow::default()));
        let char_table = tool_tree
            .tiles
            .insert_pane(ToolTab::new(CharTableToolWindow::default()));
        let bitfont_selector = tool_tree
            .tiles
            .insert_pane(ToolTab::new(BitFontSelector::default()));

        let tab = tool_tree
            .tiles
            .insert_tab_tile(vec![char_table, bitfont_selector]);
        let v = tool_tree.tiles.insert_vertical_tile(vec![tab, layers]);

        tool_tree.root = Some(v);

        MainWindow {
            document_behavior: DocumentBehavior {
                tools: Arc::new(Mutex::new(tools)),
                selected_tool: FIRST_TOOL,
                document_options: DocumentOptions {
                    scale: eframe::egui::Vec2::new(1.0, 1.0),
                },
                request_close: None,
            },
            tool_behavior: ToolBehavior::default(),
            toasts: egui_notify::Toasts::default(),
            document_tree: egui_tiles::Tree::<DocumentTab>::empty("document_tree"),
            tool_tree,
            gl: cc.gl.clone().unwrap(),
            dialog_open: false,
            modal_dialog: None,
            id: 0,
            left_panel: true,
            right_panel: true,
            bottom_panel: false,
            palette_mode: 0,
            top_bar: TopBar::new(&cc.egui_ctx),
        }
    }

    pub fn open_file(&mut self, path: &Path) {
        let full_path = path.to_str().unwrap().to_string();

        if let Some(ext) = path.extension() {
            let ext = ext.to_str().unwrap().to_ascii_lowercase();
            if "psf" == ext || "f16" == ext || "f14" == ext || "f8" == ext || "fon" == ext {
                if let Ok(data) = fs::read(path) {
                    let file_name = path.file_name();
                    if file_name.is_none() {
                        return;
                    }
                    let file_name_str = file_name.unwrap().to_str().unwrap().to_string();
                    if let Ok(font) = BitFont::from_bytes(&file_name_str, &data) {
                        add_child(
                            &mut self.document_tree,
                            Some(full_path),
                            Box::new(BitFontEditor::new(font)),
                        );
                        return;
                    }
                }
            }

            if "tdf" == ext {
                if let Ok(data) = fs::read(path) {
                    let file_name = path.file_name();
                    if file_name.is_none() {
                        return;
                    }
                    let file_name = PathBuf::from(file_name.unwrap());
                    if let Ok(fonts) = TheDrawFont::from_tdf_bytes(&data) {
                        let id = self.create_id();
                        add_child(
                            &mut self.document_tree,
                            Some(full_path),
                            Box::new(CharFontEditor::new(&self.gl, Some(file_name), id, fonts)),
                        );
                        return;
                    }
                }
            }
        }
        match Buffer::load_buffer(path, true) {
            Ok(mut buf) => {
                let id = self.create_id();
                buf.is_terminal_buffer = false;
                buf.set_buffer_height(buf.get_line_count());
                let editor = AnsiEditor::new(&self.gl, id, buf);
                add_child(&mut self.document_tree, Some(full_path), Box::new(editor));
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

    pub fn get_active_pane(&mut self) -> Option<&mut DocumentTab> {
        let mut stack = vec![];

        if let Some(root) = self.document_tree.root {
            stack.push(root);
        }
        while let Some(id) = stack.pop() {
            match self.document_tree.tiles.get(id) {
                Some(egui_tiles::Tile::Pane(_)) => {
                    if let Some(egui_tiles::Tile::Pane(p)) = self.document_tree.tiles.get_mut(id) {
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

        if let Some(root) = self.document_tree.root {
            stack.push(root);
        }
        while let Some(id) = stack.pop() {
            match self.document_tree.tiles.get(id) {
                Some(egui_tiles::Tile::Pane(_)) => {
                    if let Some(egui_tiles::Tile::Pane(p)) = self.document_tree.tiles.get_mut(id) {
                        callback(&mut p.doc.lock().unwrap());
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

    pub fn get_active_document(&mut self) -> Option<Arc<Mutex<Box<dyn Document>>>> {
        if let Some(pane) = self.get_active_pane() {
            return Some(pane.doc.clone());
        }
        None
    }

    pub(crate) fn open_dialog<T: ModalDialog + 'static>(&mut self, dialog: T) {
        self.modal_dialog = Some(Box::new(dialog));
    }

    pub(crate) fn run_editor_command<T>(
        &mut self,
        param: T,
        func: fn(&mut MainWindow, &mut AnsiEditor, T) -> Option<Message>,
    ) {
        if let Some(doc) = self.get_active_document() {
            if let Ok(mut doc) = doc.lock() {
                if let Some(editor) = doc.get_ansi_editor_mut() {
                    let msg = func(self, editor, param);
                    self.handle_message(msg);
                }
            }
        }
    }

    pub(crate) fn handle_result<T>(&mut self, result: EngineResult<T>) {
        if let Err(err) = result {
            log::error!("Error: {}", err);
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
            .exact_width(240.0)
            .resizable(false)
            .frame(egui::Frame {
                fill: ctx.style().visuals.panel_fill,
                ..Default::default()
            })
            .show_animated(ctx, self.left_panel, |ui| {
                ui.add_space(8.0);

                let mut palette: usize = self.palette_mode;
                if let Some(doc) = self.get_active_document() {
                    if let Some(editor) = doc.lock().unwrap().get_ansi_editor() {
                        ui.vertical_centered(|ui| {
                            ui.add(crate::palette_switcher(ctx, editor));
                        });
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);

                            if ui.selectable_label(palette == 0, "DOS").clicked() {
                                palette = 0;
                            }
                            if ui.selectable_label(palette == 1, "Extended").clicked() {
                                palette = 1;
                            }
                            if ui.selectable_label(palette == 2, "Custom").clicked() {
                                palette = 2;
                            }
                        });
                        ui.separator();
                        match palette {
                            0 => {
                                crate::palette_editor_16(ui, editor);
                            }
                            1 => {
                                crate::show_extended_palette(ui, editor);
                            }
                            _ => {
                                ui.label("TODO");
                            }
                        }
                        ui.separator();
                    }
                }
                self.palette_mode = palette;

                crate::add_tool_switcher(ctx, ui, self);
                if let Some(tool) = self
                    .document_behavior
                    .tools
                    .clone()
                    .lock()
                    .unwrap()
                    .get_mut(self.document_behavior.selected_tool)
                {
                    ui.horizontal(|ui| {
                        ui.add_space(4.0);
                        ui.vertical(|ui| {
                            if let Some(doc) = self.get_active_document() {
                                if let Some(editor) = doc.lock().unwrap().get_ansi_editor() {
                                    let tool_result = tool.show_ui(ctx, ui, editor);
                                    self.handle_message(tool_result);
                                }
                            }
                        });
                    });
                }
            });

        let panel_frame = egui::Frame {
            fill: ctx.style().visuals.panel_fill,
            ..Default::default()
        };

        egui::SidePanel::right("right_panel")
            .frame(panel_frame)
            .exact_width(200.0)
            .resizable(false)
            .show_animated(ctx, self.right_panel, |ui| {
                self.tool_behavior.active_document = self.get_active_document();
                self.tool_tree.ui(&mut self.tool_behavior, ui);
                self.tool_behavior.active_document = None;
                let msg = self.tool_behavior.message.take();
                self.handle_message(msg);

                /*
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

                self.bitfont_selector = Some(sel);

                // ui.add(crate::show_char_table(buffer_opt.clone()));
                */
            });

        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: ctx.style().visuals.panel_fill,
                ..Default::default()
            })
            .show(ctx, |ui| {
                self.document_tree.ui(&mut self.document_behavior, ui);

                if let Some(doc) = self.get_active_document() {
                    if let Some(editor) = doc.lock().unwrap().get_ansi_editor() {
                        let lock = &mut editor.buffer_view.lock();
                        let last = lock.get_buffer().layers.len().saturating_sub(1);
                        if let Some(layer) = lock.get_buffer().layers.last() {
                            if matches!(layer.role, icy_engine::Role::PastePreview)
                                && self.document_behavior.selected_tool != PASTE_TOOL
                            {
                                self.document_behavior.tools.lock().unwrap()[PASTE_TOOL] =
                                    Box::new(crate::model::paste_tool::PasteTool::new(
                                        self.document_behavior.selected_tool,
                                    ));
                                self.document_behavior.selected_tool = PASTE_TOOL;

                                lock.get_edit_state_mut().set_current_layer(last);
                            }
                        }
                    }
                }
            });

        self.dialog_open = false;

        if self.modal_dialog.is_some() {
            self.dialog_open = true;
            if self.modal_dialog.as_mut().unwrap().show(ctx) {
                let modal_dialog = self.modal_dialog.take().unwrap();
                if modal_dialog.should_commit() {
                    if let Some(doc) = self.get_active_document() {
                        if let Some(editor) = doc.lock().unwrap().get_ansi_editor_mut() {
                            modal_dialog.commit(editor).unwrap();
                        }
                    }
                    modal_dialog.commit_self(self).unwrap();
                }
            }
        }
        self.toasts.show(ctx);
        if let Some(close) = self.document_behavior.request_close {
            self.document_tree.tiles.remove(close);
            self.document_behavior.request_close = None;
        }

        ctx.request_repaint_after(Duration::from_millis(150));
    }

    fn on_exit(&mut self, _gl: Option<&glow::Context>) {
        /* TODO

        self.enumerate_documents( move |doc| {
            doc.destroy(gl);
        });*/
    }
}
