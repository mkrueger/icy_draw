use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::{
    create_retained_image, model::Tool, util::autosave::store_auto_save, Document, DocumentOptions,
    Message, Settings, DEFAULT_OUTLINE_TABLE, FIRST_TOOL,
};
use eframe::egui::{self, Response, Ui};
use egui_extras::RetainedImage;
use egui_tiles::{Tabs, TileId, Tiles};
use i18n_embed_fl::fl;
use icy_engine::{AttributedChar, Buffer, Position, TextAttribute, TextPane};

pub struct DocumentTab {
    pub full_path: Option<PathBuf>,
    pub doc: Arc<Mutex<Box<dyn Document>>>,
    pub auto_save_status: usize,

    pub instant: Instant,
    pub last_change: usize,
}

pub struct DocumentBehavior {
    pub tools: Arc<Mutex<Vec<Box<dyn Tool>>>>,
    pub selected_tool: usize,
    pub document_options: DocumentOptions,

    char_set_img: Option<RetainedImage>,
    cur_char_set: usize,

    pos_img: Option<RetainedImage>,
    cur_pos: Position,

    pub request_close: Option<TileId>,
    pub message: Option<Message>,
}

impl DocumentBehavior {
    pub fn new(tools: Arc<Mutex<Vec<Box<dyn Tool>>>>) -> Self {
        Self {
            tools,
            selected_tool: FIRST_TOOL,
            document_options: DocumentOptions::default(),
            char_set_img: None,
            cur_char_set: usize::MAX,
            request_close: None,
            message: None,
            pos_img: None,
            cur_pos: Position::new(i32::MAX, i32::MAX),
        }
    }
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
                    if pane.last_change != undo_stack_len {
                        pane.instant = Instant::now();
                    }
                    pane.last_change = undo_stack_len;

                    if pane.instant.elapsed().as_secs() > 5 {
                        pane.auto_save_status = undo_stack_len;
                        if let Ok(bytes) = doc.get_bytes(path) {
                            store_auto_save(path, &bytes);
                        }
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

    fn top_bar_rtl_ui(
        &mut self,
        tiles: &Tiles<DocumentTab>,
        ui: &mut Ui,
        _tile_id: TileId,
        _tabs: &Tabs,
    ) {
        ui.add_space(4.0);
        let mut buffer = Buffer::new((48, 1));
        let char_set = Settings::get_character_set();
        if self.cur_char_set != char_set {
            self.cur_char_set = char_set;
            let mut attr: TextAttribute = TextAttribute::default();
            attr.set_foreground(9);
            let s = format!("Set {:2} ", char_set + 1);
            let mut i = 0;
            for c in s.chars() {
                buffer.layers[0].set_char((i, 0), AttributedChar::new(c, attr));
                i += 1;
            }
            attr.set_foreground(15);
            attr.set_background(4);

            for j in i..buffer.get_width() {
                buffer.layers[0].set_char((j, 0), AttributedChar::new(' ', attr));
            }

            for j in 0..10 {
                if j == 9 {
                    i += 1;
                }
                let s = format!("{:-2}=", j + 1);
                attr.set_foreground(0);
                for c in s.chars() {
                    buffer.layers[0].set_char((i, 0), AttributedChar::new(c, attr));
                    i += 1;
                }
                attr.set_foreground(15);
                buffer.layers[0].set_char(
                    (i, 0),
                    AttributedChar::new(
                        unsafe {
                            char::from_u32_unchecked(
                                DEFAULT_OUTLINE_TABLE[char_set][j as usize] as u32,
                            )
                        },
                        attr,
                    ),
                );
                i += 1;
            }

            self.char_set_img = Some(create_retained_image(&buffer));
        }

        if let Some(img) = &self.char_set_img {
            img.show(ui);
        }

        if let Some(id) = _tabs.active {
            if let Some(egui_tiles::Tile::Pane(pane)) = tiles.get(id) {
                if let Ok(doc) = &mut pane.doc.lock() {
                    if let Some(editor) = doc.get_ansi_editor() {
                        let pos = editor.get_caret_position();

                        if pos != self.cur_pos {
                            self.cur_pos = pos;
                            let txt = fl!(
                                crate::LANGUAGE_LOADER,
                                "toolbar-position",
                                line = pos.y,
                                column = pos.x
                            );

                            let mut buffer = Buffer::new((txt.chars().count(), 1));
                            buffer.is_terminal_buffer = true;
                            let mut attr: TextAttribute = TextAttribute::default();
                            attr.set_foreground(15);

                            for (i, mut c) in txt.chars().enumerate() {
                                if c as u32 > 255 {
                                    c = ' ';
                                }
                                buffer.layers[0].set_char((i, 0), AttributedChar::new(c, attr));
                            }
                            self.pos_img = Some(create_retained_image(&buffer));
                        }

                        if let Some(img) = &self.pos_img {
                            img.show(ui);
                        }
                    }
                }
            }
        }
    }

    /*


    */
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
        instant: Instant::now(),
        last_change: 0,
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
