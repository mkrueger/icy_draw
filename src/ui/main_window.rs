use std::{path::PathBuf, fs, sync::Arc, time::Duration};

use eframe::{egui::{self, menu, TopBottomPanel, SidePanel}};
use egui_dock::{DockArea, Style,  Tree, Node};
use glow::Context;
use i18n_embed_fl::fl;
use icy_engine::{BitFont, Buffer};
use crate::{Document, FontEditor, model::Tool};

use super::ansi_editor::AnsiEditor;
use egui_file::FileDialog;
pub struct NewFileDialog {
    width: i32,
    height: i32,

    create: bool
}

impl NewFileDialog {
    pub fn new() -> Self {
        NewFileDialog {
            width: 80,
            height: 25,
            create: false
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut open = true;
        let mut create_file = true;
        egui::Window::new(fl!(crate::LANGUAGE_LOADER, "new-file-title"))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(fl!(crate::LANGUAGE_LOADER, "new-file-width"));
                let mut my_f32 = self.width as f32;
                ui.add(egui::DragValue::new(&mut my_f32).speed(1));
                self.width = my_f32 as i32;
            });
            ui.horizontal(|ui| {
                ui.label(fl!(crate::LANGUAGE_LOADER, "new-file-height"));
                let mut my_f32 = self.height as f32;
                ui.add(egui::DragValue::new(&mut my_f32).speed(1));
                self.height = my_f32 as i32;
            });
            if ui.button(fl!(crate::LANGUAGE_LOADER, "new-file-ok")).clicked() {
                self.create = true;
                create_file = false;
            }
        });

        !(open && create_file)
    }
}

pub struct MainWindow {
    pub tab_viewer: TabViewer,
    tree: Tree<(Option<String>, Box<dyn Document>)>,
    gl: Arc<Context>,

    opened_file: Option<PathBuf>,
    open_file_dialog: Option<FileDialog>,
    new_file_dialog: Option<NewFileDialog>,
}

impl MainWindow {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let tree = Tree::new(Vec::new());
  
        let mut tools: Vec<Box::<dyn Tool>> = Vec::new();
       
        tools.push(Box::new(crate::model::click_imp::ClickTool { }));
        tools.push(Box::new(crate::model::brush_imp::BrushTool {
            size: 3, 
            use_back: true,
            use_fore: true,
            brush_type: crate::model::brush_imp::BrushType::Shade,
            char_code: '\u{00B0}',
            font_page: 0
        }));
        tools.push(Box::new(crate::model::erase_imp::EraseTool { size: 3, brush_type: crate::model::erase_imp::EraseType::Shade }));
        tools.push(Box::new(crate::model::pipette_imp::PipetteTool { }));
        tools.push(Box::new(crate::model::line_imp::LineTool {
            draw_mode: crate::model::DrawMode::Line, 
            use_fore: true, 
            use_back: true, 
            attr: icy_engine::TextAttribute::default(), 
            char_code: '\u{00B0}',
            font_page: 0,
            old_pos: icy_engine::Position { x: 0, y: 0 }
        }));
        tools.push(Box::new(crate::model::draw_rectangle_imp::DrawRectangleTool { 
            draw_mode: crate::model::DrawMode::Line, 
            use_fore: true, 
            use_back: true, 
            attr: icy_engine::TextAttribute::default(), 
            char_code: '\u{00B0}',
            font_page: 0
        }));

        tools.push(Box::new(crate::model::draw_rectangle_filled_imp::DrawRectangleFilledTool { 
            draw_mode: crate::model::DrawMode::Line, 
            use_fore: true, 
            use_back: true, 
            attr: icy_engine::TextAttribute::default(), 
            char_code: '\u{00B0}',
            font_page: 0
        }));
        tools.push(Box::new(crate::model::draw_ellipse_imp::DrawEllipseTool {
            draw_mode: crate::model::DrawMode::Line, 
            use_fore: true, 
            use_back: true, 
            attr: icy_engine::TextAttribute::default(), 
            char_code: '\u{00B0}',
            font_page: 0
        }));

        tools.push(Box::new(crate::model::draw_ellipse_filled_imp::DrawEllipseFilledTool {
            draw_mode: crate::model::DrawMode::Line, 
            use_fore: true, 
            use_back: true, 
            attr: icy_engine::TextAttribute::default(), 
            char_code: '\u{00B0}',
            font_page: 0
        }));

        tools.push(Box::new(crate::model::fill_imp::FillTool {
            use_fore: true,
            use_back: true,
            char_code: '\u{00B0}',
            font_page: 0,
            fill_type: crate::model::fill_imp::FillType::Character,
            attr: icy_engine::TextAttribute::default()
        }));

        tools.push(Box::new(crate::model::move_layer_imp::MoveLayer { pos: icy_engine::Position { x: 0, y: 0 } }));

        let view = MainWindow {
            tab_viewer: TabViewer {
                tools,
                selected_tool: 0
            },
            tree,
            gl: cc.gl.clone().unwrap(),
            opened_file: None,
            open_file_dialog: None,        
            new_file_dialog: None
        };
        view
    }

    pub fn open_file(&mut self, path: PathBuf) {
        let full_path = path.to_str().unwrap().to_string();

        if let Some(ext) = path.extension()  {
            match ext.to_str().unwrap() { 
                "psf" => {
                    if let Ok(data) = fs::read(&path) {
                        let file_name = path.file_name();
                        if file_name.is_none() {
                            return;
                        }
                        let file_name_str = file_name.unwrap().to_str().unwrap().to_string();
                        if let Ok(font) = BitFont::from_bytes(&file_name_str, &data) {
                            self.tree.push_to_focused_leaf((Some(full_path), Box::new(FontEditor::new(font))));
                            return;
                        }
                    }
                }
                _ => {}
            }
        }
        let buf = Buffer::load_buffer(&path).unwrap();
        let editor = AnsiEditor::new(&self.gl, buf);
        self.tree.push_to_focused_leaf((Some(full_path), Box::new(editor)));

    }

}


pub struct TabViewer {
    pub tools: Vec<Box<dyn Tool>>,
    pub selected_tool: usize,
}

impl egui_dock::TabViewer for TabViewer {
    type Tab = (Option<String>, Box<dyn Document>);

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.1.show_ui(ui, &mut self.tools[self.selected_tool]);
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        let mut title = tab.1.get_title();
        if tab.1.is_dirty() {
            title.push('*');
        }
        title.into()
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use egui::FontFamily::Proportional;
        use egui::FontId;
        use egui::TextStyle::*;

        let mut style: egui::Style = (*ctx.style()).clone();
        style.text_styles = [
            (Heading, FontId::new(30.0, Proportional)),
            (Body, FontId::new(20.0, Proportional)),
            (Monospace, FontId::new(20.0, Proportional)),
            (Button, FontId::new(20.0, Proportional)),
            (Small, FontId::new(16.0, Proportional)),
        ].into();
        ctx.set_style(style);
        
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-file"), |ui| {
                    if ui.button(fl!(crate::LANGUAGE_LOADER, "menu-open")).clicked() {
                        let mut dialog = FileDialog::open_file(self.opened_file.clone());
                        dialog.open();
                        self.open_file_dialog = Some(dialog);

                        ui.close_menu();
                    }
                    if ui.button(fl!(crate::LANGUAGE_LOADER, "menu-save")).clicked() {
                        if let Some(t) = self.tree.find_active_focused() {
                            if let Some(str) = &t.1.0 {
                                t.1.1.save(str);
                            }
                        }
                        ui.close_menu();
                    }
                });
            });

            if ui.button(fl!(crate::LANGUAGE_LOADER, "toolbar-new")).clicked() {
                self.new_file_dialog = Some(NewFileDialog::new());
            }
        });
        SidePanel::left("left_panel").show(ctx, |ui| {
            let mut buffer_opt = None;
            if let Some((_, t)) = self.tree.find_active_focused() {
                buffer_opt = t.1.get_buffer_view();
            }
            ui.vertical_centered(|ui| {
                ui.add(crate::palette_switcher(ctx, buffer_opt.clone()));
            });
            ui.add(crate::palette_editor_16(buffer_opt.clone()));
            crate::add_tool_switcher(ctx, ui, self);

            if let Some(tool) = self.tab_viewer.tools.get_mut(self.tab_viewer.selected_tool) {
                tool.show_ui(ctx, ui, buffer_opt.clone());
            }
        });

        TopBottomPanel::bottom("bottom_panel").resizable(true).show(ctx, |ui| {
            let mut buffer_opt = None;
            if let Some((_, t)) = self.tree.find_active_focused() {
                buffer_opt = t.1.get_buffer_view();
            }
            ui.add(crate::show_char_table(buffer_opt.clone()));
        });

        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut self.tab_viewer);

        if let Some(dialog) = &mut self.open_file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    self.open_file(file);
                }
            }
        }
        
        if let Some(dialog) = &mut self.new_file_dialog {
            if dialog.show(ctx) {
                if dialog.create {
                    let buf = Buffer::create(dialog.width, dialog.height);
                    let editor = AnsiEditor::new(&self.gl, buf);
                    self.tree.push_to_focused_leaf((None, Box::new(editor)));
                }
                self.new_file_dialog = None;
            }
        }
            
        ctx.request_repaint_after(Duration::from_millis(150));
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            for t in self.tree.iter() {
                if let Node::Leaf { tabs, .. } = t  {
                    for (_, t) in tabs {
                        t.destroy(gl);
                    }
                }
            }
        }
    }
}

