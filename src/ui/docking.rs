use crate::{model::Tool, Document, DocumentOptions};
use eframe::egui;
pub type DockingContainer = egui_tiles::Tree<Tab>;

pub struct Tab {
    pub full_path: Option<String>,
    pub doc: Box<dyn Document>,
}

pub struct TabBehavior {
    pub tools: Vec<Box<dyn Tool>>,
    pub selected_tool: usize,
    pub document_options: DocumentOptions,
}

impl egui_tiles::Behavior<Tab> for TabBehavior {
    fn tab_title_for_pane(&mut self, pane: &Tab) -> egui::WidgetText {
        println!("get title for pane {}", pane.doc.get_title());
        let mut title = pane.doc.get_title();
        if pane.doc.is_dirty() {
            title.push('*');
        }
        title.into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Tab,
    ) -> egui_tiles::UiResponse {
        pane.doc.show_ui(
            ui,
            &mut self.tools[self.selected_tool],
            &self.document_options,
        );
        // You can make your pane draggable like so:
        if ui
            .add(egui::Button::new("Drag me!").sense(egui::Sense::drag()))
            .drag_started()
        {
            egui_tiles::UiResponse::DragStarted
        } else {
            egui_tiles::UiResponse::None
        }
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        egui_tiles::SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }
}

pub fn add_child(
    tree: &mut egui_tiles::Tree<Tab>,
    full_path: Option<String>,
    doc: Box<dyn Document>,
) {
    let tile = Tab { full_path, doc };
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
