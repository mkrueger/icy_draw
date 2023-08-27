use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::Duration,
};

use crate::{
    model::Tool, AnsiEditor, Document, DocumentOptions, EditSauceDialog, FontEditor, ModalDialog,
    NewFileDialog, SelectOutlineDialog,
};
use eframe::{
    egui::{self, menu, Modifiers, Response, SidePanel, TextStyle, TopBottomPanel, Ui, Id},
    epaint::{pos2, Vec2},
};
use egui_dock::{DockArea, Node, Style, Tree};
use glow::Context;
use i18n_embed_fl::fl;
use icy_engine::{BitFont, Buffer, Position, SaveOptions, Selection};

use super::SetCanvasSizeDialog;
use egui_file::FileDialog;

pub struct MainWindow {
    pub tab_viewer: TabViewer,
    tree: Tree<(Option<String>, Box<dyn Document>)>,
    gl: Arc<Context>,

    opened_file: Option<PathBuf>,

    dialog_open: bool,
    open_file_dialog: Option<FileDialog>,
    save_file_dialog: Option<FileDialog>,
    new_file_dialog: Option<NewFileDialog>,

    modal_dialog: Option<Box<dyn ModalDialog>>,
    id: usize,
}

impl MainWindow {
    fn create_id(&mut self) -> usize {
        self.id += 1;
        self.id
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let tree = Tree::new(Vec::new());

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

        MainWindow {
            tab_viewer: TabViewer {
                tools,
                selected_tool: 0,
                document_options: DocumentOptions {
                    scale: eframe::egui::Vec2::new(1.0, 1.0),
                },
            },
            tree,
            gl: cc.gl.clone().unwrap(),
            opened_file: None,
            dialog_open: false,
            open_file_dialog: None,
            save_file_dialog: None,
            new_file_dialog: None,
            modal_dialog: None,
            id: 0
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
                        self.tree.push_to_focused_leaf((
                            Some(full_path),
                            Box::new(FontEditor::new(font, id)),
                        ));
                        return;
                    }
                }
            }
        }
        let buf = Buffer::load_buffer(path, true).unwrap();
        let id = self.create_id();
        let editor = AnsiEditor::new(&self.gl, id, buf);
        self.tree
            .push_to_focused_leaf((Some(full_path), Box::new(editor)));
    }

    fn main_menu(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        menu::bar(ui, |ui| {
            let mut buffer_opt = None;
            if let Some((_, t)) = self.tree.find_active_focused() {
                buffer_opt = t.1.get_buffer_view();
            }

            let has_buffer = buffer_opt.is_some();

            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-file"), |ui| {
                if ui
                    .add(egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-new")).wrap(false))
                    .clicked()
                {
                    self.new_file_dialog = Some(NewFileDialog::default());
                    ui.close_menu();
                }

                if ui
                    .add(egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-open")).wrap(false))
                    .clicked()
                {
                    let mut dialog = FileDialog::open_file(self.opened_file.clone());
                    dialog.open();
                    self.open_file_dialog = Some(dialog);

                    ui.close_menu();
                }
                ui.separator();
                if ui
                    .add_enabled(
                        has_buffer,
                        egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-save")).wrap(false),
                    )
                    .clicked()
                {
                    if let Some(t) = self.tree.find_active_focused() {
                        if let Some(str) = &t.1 .0 {
                            t.1 .1.save(str).unwrap();
                        }
                    }
                    ui.close_menu();
                }
                if ui
                    .add_enabled(
                        has_buffer,
                        egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-save-as")).wrap(false),
                    )
                    .clicked()
                {
                    if let Some(t) = self.tree.find_active_focused() {
                        if let Some(str) = &t.1 .0 {
                            let mut dialog = FileDialog::save_file(Some(PathBuf::from(str)));
                            dialog.open();
                            self.save_file_dialog = Some(dialog);
                            ui.close_menu();
                        }
                    }
                }

                if ui
                    .add_enabled(
                        has_buffer,
                        egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-export")).wrap(false),
                    )
                    .clicked()
                {
                    let mut buffer_opt = None;
                    if let Some((_, t)) = self.tree.find_active_focused() {
                        buffer_opt = t.1.get_buffer_view();
                    }

                    self.modal_dialog = Some(Box::new(crate::ExportFileDialog::new(
                        &buffer_opt.unwrap().buffer_view.lock().buf,
                    )));
                    ui.close_menu();
                }
                ui.separator();
                if ui
                    .add(
                        egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-edit-font-outline"))
                            .wrap(false),
                    )
                    .clicked()
                {
                    self.modal_dialog = Some(Box::<SelectOutlineDialog>::default());
                    ui.close_menu();
                }

                ui.separator();
                let button: Response = button_with_shortcut(
                    ui,
                    true,
                    fl!(crate::LANGUAGE_LOADER, "menu-close"),
                    "Ctrl+Q",
                );
                if button.clicked() {
                    _frame.close();
                    ui.close_menu();
                }
            });

            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-edit"), |ui| {
                let button: Response = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-undo"),
                    "Ctrl+Z",
                );
                if button.clicked() {
                    self.undo_command();
                    ui.close_menu();
                }

                let button: Response = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-redo"),
                    "Ctrl+Shift+Z",
                );
                if button.clicked() {
                    self.redo_command();
                    ui.close_menu();
                }
                ui.separator();
                if ui
                    .add_enabled(
                        has_buffer,
                        egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-edit-sauce"))
                            .wrap(false),
                    )
                    .clicked()
                {
                    let mut buffer_opt = None;
                    if let Some((_, t)) = self.tree.find_active_focused() {
                        buffer_opt = t.1.get_buffer_view();
                    }

                    self.modal_dialog = Some(Box::new(EditSauceDialog::new(
                        &buffer_opt.unwrap().buffer_view.lock().buf,
                    )));
                    ui.close_menu();
                }

                if ui
                    .add_enabled(
                        has_buffer,
                        egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-set-canvas-size"))
                            .wrap(false),
                    )
                    .clicked()
                {
                    let mut buffer_opt = None;
                    if let Some((_, t)) = self.tree.find_active_focused() {
                        buffer_opt = t.1.get_buffer_view();
                    }
                    self.modal_dialog = Some(Box::new(SetCanvasSizeDialog::new(
                        &buffer_opt.unwrap().buffer_view.lock().buf,
                    )));
                    ui.close_menu();
                }
            });

            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-selection"), |ui| {
                let button: Response = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-select-all"),
                    "Ctrl+A",
                );
                if button.clicked() {
                    self.select_all_command();
                    ui.close_menu();
                }

                let button: Response = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-deselect"),
                    "Esc",
                );
                if button.clicked() {
                    if let Some(t) = self.tree.find_active_focused() {
                        let doc = t.1 .1.get_buffer_view();
                        if let Some(editor) = doc {
                            editor.cur_selection = None;
                            editor.redraw_view();
                        }
                    }
                    ui.close_menu();
                }
                ui.separator();

                let button: Response = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-erase"),
                    "Del",
                );
                if button.clicked() {
                    if let Some(t) = self.tree.find_active_focused() {
                        let doc = t.1 .1.get_buffer_view();
                        if let Some(editor) = doc {
                            editor.delete_selection();
                            editor.redraw_view();
                        }
                    }
                    ui.close_menu();
                }

                let button: Response = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-flipx"),
                    "X",
                );
                if button.clicked() {
                    if let Some(t) = self.tree.find_active_focused() {
                        let doc: Option<&mut AnsiEditor> = t.1 .1.get_buffer_view();
                        if let Some(editor) = doc {
                            editor.flip_x();
                            editor.redraw_view();
                        }
                    }
                    ui.close_menu();
                }

                let button: Response = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-flipy"),
                    "Y",
                );
                if button.clicked() {
                    if let Some(t) = self.tree.find_active_focused() {
                        let doc = t.1 .1.get_buffer_view();
                        if let Some(editor) = doc {
                            editor.flip_y();
                            editor.redraw_view();
                        }
                    }
                    ui.close_menu();
                }

                let button: Response = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-justifycenter"),
                    "Y",
                );
                if button.clicked() {
                    if let Some(t) = self.tree.find_active_focused() {
                        let doc = t.1 .1.get_buffer_view();
                        if let Some(editor) = doc {
                            editor.justify_center();
                            editor.redraw_view();
                        }
                    }
                    ui.close_menu();
                }

                let button: Response = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-justifyleft"),
                    "L",
                );
                if button.clicked() {
                    if let Some(t) = self.tree.find_active_focused() {
                        let doc = t.1 .1.get_buffer_view();
                        if let Some(editor) = doc {
                            editor.justify_left();
                            editor.redraw_view();
                        }
                    }
                    ui.close_menu();
                }

                let button: Response = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-justifyright"),
                    "R",
                );
                if button.clicked() {
                    if let Some(t) = self.tree.find_active_focused() {
                        let doc = t.1 .1.get_buffer_view();
                        if let Some(editor) = doc {
                            editor.justify_right();
                            editor.redraw_view();
                        }
                    }
                    ui.close_menu();
                }
                ui.separator();

                let button: Response = button_with_shortcut(
                    ui,
                    has_buffer,
                    fl!(crate::LANGUAGE_LOADER, "menu-crop"),
                    "",
                );
                if button.clicked() {
                    if let Some(t) = self.tree.find_active_focused() {
                        let doc = t.1 .1.get_buffer_view();
                        if let Some(editor) = doc {
                            editor.crop();
                            editor.redraw_view();
                        }
                    }
                    ui.close_menu();
                }
            });

            ui.menu_button("View", |ui| {
                if ui.button("100%").clicked() {
                    self.tab_viewer.document_options.scale = Vec2::new(1.0, 1.0);
                    ui.close_menu();
                }
                if ui.button("200%").clicked() {
                    self.tab_viewer.document_options.scale = Vec2::new(2.0, 2.0);
                    ui.close_menu();
                }
                if ui.button("300%").clicked() {
                    self.tab_viewer.document_options.scale = Vec2::new(3.0, 3.0);
                    ui.close_menu();
                }
            });

            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-help"), |ui| {
                let r = ui.hyperlink_to(
                    fl!(crate::LANGUAGE_LOADER, "menu-discuss"),
                    "https://github.com/mkrueger/icy_draw/discussions",
                );
                if r.clicked() {
                    ui.close_menu();
                }
                let r = ui.hyperlink_to(
                    fl!(crate::LANGUAGE_LOADER, "menu-report-bug"),
                    "https://github.com/mkrueger/icy_draw/issues/new",
                );
                if r.clicked() {
                    ui.close_menu();
                }
                ui.separator();
                if ui
                    .button(fl!(crate::LANGUAGE_LOADER, "menu-about"))
                    .clicked()
                {
                    self.modal_dialog = Some(Box::<crate::AboutDialog>::default());
                    ui.close_menu();
                }
            });
        });

        if ui.input(|i| i.key_pressed(egui::Key::Q) && i.modifiers.ctrl) {
            _frame.close();
        }

        if ui.input(|i| i.key_pressed(egui::Key::A) && i.modifiers.ctrl) {
            ui.input_mut(|i| i.consume_key(Modifiers::CTRL, egui::Key::A));
            self.select_all_command();
        }

        if ui.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.ctrl && !i.modifiers.shift) {
            ui.input_mut(|i| i.consume_key(Modifiers::CTRL, egui::Key::Z));
            self.undo_command();
        }

        if ui.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.shift && i.modifiers.ctrl) {
            ui.input_mut(|i| i.consume_key(CTRL_SHIFT, egui::Key::Z));
            self.redo_command();
        }
    }

    fn redo_command(&mut self) {
        if let Some(t) = self.tree.find_active_focused() {
            let doc = t.1 .1.get_buffer_view();
            if let Some(editor) = doc {
                editor.redo();
                editor.buffer_view.lock().redraw_view();
            }
        }
    }

    fn undo_command(&mut self) {
        if let Some(t) = self.tree.find_active_focused() {
            let doc = t.1 .1.get_buffer_view();
            if let Some(editor) = doc {
                editor.undo();
                editor.buffer_view.lock().redraw_view();
            }
        }
    }

    fn select_all_command(&mut self) {
        if let Some(t) = self.tree.find_active_focused() {
            let doc = t.1 .1.get_buffer_view();
            if let Some(ansi_editor) = doc {
                let buf = &mut ansi_editor.buffer_view.lock();
                let w = buf.buf.get_buffer_width();
                let h = buf.buf.get_real_buffer_height();

                buf.set_selection(Selection::from_rectangle(0.0, 0.0, w as f32, h as f32));
            }
        }
    }
}

const CTRL_SHIFT: egui::Modifiers = egui::Modifiers {
    alt: false,
    ctrl: true,
    shift: true,
    mac_cmd: false,
    command: false,
};

pub struct TabViewer {
    pub tools: Vec<Box<dyn Tool>>,
    pub selected_tool: usize,
    pub document_options: DocumentOptions,

}

impl egui_dock::TabViewer for TabViewer {
    type Tab = (Option<String>, Box<dyn Document>);

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, tab: &mut Self::Tab) {
       tab.1.show_ui(
            ui,
            &mut self.tools[self.selected_tool],
            &self.document_options,
        );
    }

    fn id(&mut self, tab: &mut Self::Tab) -> Id {
        Id::new(tab.1.get_id())
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        let mut title = tab.1.get_title();
        if tab.1.is_dirty() {
            title.push('*');
        }
        title.into()
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use egui::FontFamily::Proportional;
        use egui::FontId;
        use egui::TextStyle::*;

        if let Some(file) = &self.opened_file.clone() {
            self.opened_file = None;
            self.open_file(file);
        }
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

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.main_menu(ui, _frame);
        });
        SidePanel::left("left_panel").show(ctx, |ui| {
            if let Some((_, t)) = self.tree.find_active_focused() {
                if let Some(editor) = t.1.get_buffer_view() {
                    ui.vertical_centered(|ui| {
                        ui.add(crate::palette_switcher(ctx, editor));
                    });
                    ui.add(crate::palette_editor_16(editor));
                }
            }
            crate::add_tool_switcher(ctx, ui, self);

            if let Some(tool) = self.tab_viewer.tools.get_mut(self.tab_viewer.selected_tool) {
                if let Some((_, t)) = self.tree.find_active_focused() {
                    if let Some(editor) = t.1.get_buffer_view() {
                        let tool_result = tool.show_ui(ctx, ui, editor);
                        if tool_result.modal_dialog.is_some() {
                            self.modal_dialog = tool_result.modal_dialog;
                        }
                    }
                }
            }
        });
        SidePanel::right("right_panel").show(ctx, |ui| {
            if let Some((_, t)) = self.tree.find_active_focused() {
                if let Some(editor) = t.1.get_buffer_view() {
                    crate::show_layer_view(ctx, ui, editor);
                }
            }
            // ui.add(crate::show_char_table(buffer_opt.clone()));
        });

        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut self.tab_viewer);

        self.dialog_open = false;
        if let Some(dialog) = &mut self.open_file_dialog {
            self.dialog_open = true;
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    self.opened_file = Some(file.to_path_buf());
                }
                self.open_file_dialog = None;
            }
        }
        if let Some(dialog) = &mut self.save_file_dialog {
            self.dialog_open = true;
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    if let Some((_, t)) = self.tree.find_active_focused() {
                        if let Some(editor) = t.1.get_buffer_view() {
                            let options = SaveOptions::new();
                            editor
                                .save_content(file.to_path_buf().as_path(), &options)
                                .unwrap();
                        }
                    }
                }
                self.save_file_dialog = None;
            }
        }

        if let Some(dialog) = &mut self.new_file_dialog {
            self.dialog_open = true;
            if dialog.show(ctx) {
                if dialog.create {
                    let buf = Buffer::create(dialog.width, dialog.height);
                    let id = self.create_id();
                    let editor = AnsiEditor::new(&self.gl, id, buf);
                    self.tree.push_to_focused_leaf((None, Box::new(editor)));
                }
                self.new_file_dialog = None;
            }
        }

        if let Some(dialog) = &mut self.modal_dialog {
            self.dialog_open = true;
            if dialog.show(ctx) {
                if dialog.should_commit() {
                    if let Some((_, t)) = self.tree.find_active_focused() {
                        if let Some(editor) = t.1.get_buffer_view() {
                            dialog.commit(editor).unwrap();
                        }
                    }
                }
                self.modal_dialog = None;
            }
        }

        for t in self.tree.iter_mut() {
            if let Node::Leaf { tabs, .. } = t {
                for (_, t) in tabs {
                    t.set_enabled(!self.dialog_open);
                }
            }
        }

        ctx.request_repaint_after(Duration::from_millis(150));
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            for t in self.tree.iter() {
                if let Node::Leaf { tabs, .. } = t {
                    for (_, t) in tabs {
                        t.destroy(gl);
                    }
                }
            }
        }
    }
}
