use std::sync::{Arc, Mutex};

use crate::{model::Tool, Document, DocumentOptions};
use eframe::egui::{self, Response};
use egui_tiles::{TileId, Tiles};

pub struct DocumentTab {
    pub full_path: Option<String>,
    pub doc: Arc<Mutex<Box<dyn Document>>>,
}

pub struct DocumentBehavior {
    pub tools: Arc<Mutex<Vec<Box<dyn Tool>>>>,
    pub selected_tool: usize,
    pub document_options: DocumentOptions,

    pub request_close: Option<TileId>,
}

impl egui_tiles::Behavior<DocumentTab> for DocumentBehavior {
    fn tab_title_for_pane(&mut self, pane: &DocumentTab) -> egui::WidgetText {
        let doc = pane.doc.lock().unwrap();
        let mut title = doc.get_title();
        if doc.is_dirty() {
            title.push('*');
        }
        title.into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut DocumentTab,
    ) -> egui_tiles::UiResponse {

        pane.doc.lock().unwrap().show_ui(
            ui,
            &mut self.tools.lock().unwrap()[self.selected_tool],
            &self.document_options,
        );
        egui_tiles::UiResponse::None
    }

    fn on_tab_button(
        &mut self,
        tiles: &Tiles<DocumentTab>,
        tile_id: TileId,
        button_response: eframe::egui::Response,
    ) -> Response {
        button_response.context_menu(|ui| {
            if ui.button("Close").clicked() {
                self.on_close_requested(tiles, tile_id);
                ui.close_menu();
            }
        })
    }

    fn on_close_requested(&mut self, _tiles: &Tiles<DocumentTab>, tile_id: TileId) {
        self.request_close = Some(tile_id);
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        egui_tiles::SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }

    fn has_close_buttons(&self) -> bool {
        true
    }
}

pub fn add_child(
    tree: &mut egui_tiles::Tree<DocumentTab>,
    full_path: Option<String>,
    doc: Box<dyn Document>,
) {
    let tile = DocumentTab { full_path, doc: Arc::new(Mutex::new(doc)) };
    let new_child = tree.tiles.insert_pane(tile);

    if tree.root.is_none() {
        tree.root = Some(new_child);
    } else if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Tabs(tabs))) =
        tree.tiles.get_mut(tree.root.unwrap())
    {
        tabs.add_child(new_child);
        tabs.set_active(new_child);
    } else if let Some(egui_tiles::Tile::Pane(_)) = tree.tiles.get(tree.root.unwrap()) {
        let new_id = tree
            .tiles
            .insert_tab_tile(vec![new_child, tree.root.unwrap()]);
        tree.root = Some(new_id);
    } else {
        tree.root = Some(new_child);
    }
}
