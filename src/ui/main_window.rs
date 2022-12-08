use std::{path::PathBuf, fs, sync::Arc, time::Duration};

use eframe::egui::{self, menu, TopBottomPanel};
use egui_dock::{DockArea, Style,  Tree, Node};
use glow::Context;
use icy_engine::{BitFont, Buffer};
use rfd::FileDialog;

use crate::{Document, FontEditor};

use super::ansi_editor::AnsiEditor;

pub struct MainWindow {
    tree: Tree<(String, Box<dyn Document>)>,
    gl: Arc<Context>
}

impl MainWindow {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let tree = Tree::new(Vec::new());

        let view = MainWindow {
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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        let files = FileDialog::new().pick_files();
                        if let Some(paths) = files {
                            for path in paths {
                                self.open_file(path);
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        if let Some(t) = self.tree.find_active_focused() {
                            let str = &t.1.0;
                            t.1.1.save(str);
                        }

                        ui.close_menu();
                    }
                });
            });
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
                    for (s, t) in tabs {
                        t.destroy(gl);
                    }
                }
            }
        }
    }
}
