use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{model::Tool, util::autosave::store_auto_save, Document, DocumentOptions, Message};
use eframe::egui::{self, Response};
use egui_tiles::{TileId, Tiles};

pub struct DocumentTab {
    pub full_path: Option<PathBuf>,
    pub doc: Arc<Mutex<Box<dyn Document>>>,
    pub auto_save_status: usize,
}

pub struct DocumentBehavior {
    pub tools: Arc<Mutex<Vec<Box<dyn Tool>>>>,
    pub selected_tool: usize,
    pub document_options: DocumentOptions,

    pub request_close: Option<TileId>,
    pub message: Option<Message>,
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
        if let Ok(doc) = &mut pane.doc.lock() {
            self.message = doc.show_ui(
                ui,
                &mut self.tools.lock().unwrap()[self.selected_tool],
                self.selected_tool,
                &self.document_options,
            );
            let undo_stack_len = doc.undo_stack_len();
            if let Some(path) = &pane.full_path {
                if doc.is_dirty() && undo_stack_len != pane.auto_save_status {
                    pane.auto_save_status = undo_stack_len;
                    if let Ok(bytes) = doc.get_bytes(path) {
                        store_auto_save(path, &bytes);
                    }
                }
            }
        }
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
    full_path: Option<PathBuf>,
    doc: Box<dyn Document>,
) {
    let tile = DocumentTab {
        full_path,
        doc: Arc::new(Mutex::new(doc)),
        auto_save_status: 0,
    };
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
