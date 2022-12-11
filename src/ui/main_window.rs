use std::{path::PathBuf, fs, sync::Arc, time::Duration};

use eframe::{egui::{self, menu, TopBottomPanel, SidePanel, CentralPanel}};
use egui_dock::{DockArea, Style,  Tree, Node};
use glow::Context;
use i18n_embed_fl::fl;
use icy_engine::{BitFont, Buffer};
use rfd::FileDialog;
use crate::{Document, FontEditor, model::Tool};

use super::ansi_editor::AnsiEditor;

pub struct MainWindow {
    pub tools: Vec<Box<dyn Tool>>,
    pub selected_tool: usize,
    tree: Tree<(String, Box<dyn Document>)>,
    gl: Arc<Context>
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
            tools,
            selected_tool: 0,
            tree,
            gl: cc.gl.clone().unwrap()
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
                            self.tree.push_to_focused_leaf((full_path, Box::new(FontEditor::new(font))));
                            return;
                        }
                    }
                }
                _ => {}
            }
        }
        let buf = Buffer::load_buffer(&path).unwrap();
        
        let editor = AnsiEditor::new(&self.gl, buf);

        self.tree.push_to_focused_leaf((full_path, Box::new(editor)));

    }

}


struct TabViewer {

}

impl egui_dock::TabViewer for TabViewer {
    type Tab = (String, Box<dyn Document>);

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.1.show_ui(ui);
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
                        let files = FileDialog::new().pick_files();
                        if let Some(paths) = files {
                            for path in paths {
                                self.open_file(path);
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button(fl!(crate::LANGUAGE_LOADER, "menu-save")).clicked() {
                        if let Some(t) = self.tree.find_active_focused() {
                            let str = &t.1.0;
                            t.1.1.save(str);
                        }
                        ui.close_menu();
                    }
                });
            });
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

            if let Some(tool) = self.tools.get_mut(self.selected_tool) {
                tool.show_ui(ctx, ui, buffer_opt.clone());
            }
        });

        TopBottomPanel::bottom("bottom_panel").resizable(true).show(ctx, |ui| {
            let mut buffer_opt = None;
            if let Some((_, t)) = self.tree.find_active_focused() {
                buffer_opt = t.1.get_buffer_view();
            }
            ui.add(crate::show_char_table(ctx, buffer_opt.clone()));
        });

        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut TabViewer {});

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

