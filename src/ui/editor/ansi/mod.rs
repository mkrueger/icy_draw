use std::{
    cmp::{max, min},
    ffi::OsStr,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use eframe::{
    egui::{self, Id, Key, Response, RichText},
    epaint::{mutex::Mutex, FontId, Vec2},
};
use i18n_embed_fl::fl;
use icy_engine::{
    editor::{AtomicUndoGuard, UndoState},
    util::{pop_data, pop_sixel_image, push_data, BUFFER_DATA},
    AttributedChar, Buffer, EngineResult, Line, Position, Rectangle, SaveOptions, TextAttribute,
    TextPane,
};

use icy_engine_egui::{
    show_terminal_area, BackgroundEffect, BufferView, MonitorSettings, TerminalCalc,
};

use crate::{
    model::{MKey, MModifiers, Tool, DragPos},
    ClipboardHandler, Document, DocumentOptions, Message, TerminalResult, FIRST_TOOL,
};

pub enum Event {
    None,
    CursorPositionChange(Position, Position),
}

pub struct AnsiEditor {
    pub id: usize,
    dirty_pos: usize,
    pub drag_pos: DragPos,

    pub buffer_view: Arc<eframe::epaint::mutex::Mutex<BufferView>>,
    pub cur_outline: usize,
    pub is_inactive: bool,

    pub reference_image: Option<PathBuf>,
    // pub outline_changed: std::boxed::Box<dyn Fn(&Editor)>,
    //pub request_refresh: Box<dyn Fn ()>,
    pub egui_id: Id,
    //pub pos_changed: std::boxed::Box<dyn Fn(&Editor, Position)>,
    //pub attr_changed: std::boxed::Box<dyn Fn(TextAttribute)>
}

impl UndoState for AnsiEditor {
    fn undo_description(&self) -> Option<String> {
        self.buffer_view.lock().get_edit_state().undo_description()
    }

    fn can_undo(&self) -> bool {
        self.buffer_view.lock().get_edit_state().can_undo()
    }

    fn undo(&mut self) -> icy_engine::EngineResult<()> {
        self.buffer_view.lock().get_edit_state_mut().undo()
    }

    fn redo_description(&self) -> Option<String> {
        self.buffer_view.lock().get_edit_state().redo_description()
    }

    fn can_redo(&self) -> bool {
        self.buffer_view.lock().get_edit_state().can_redo()
    }

    fn redo(&mut self) -> icy_engine::EngineResult<()> {
        self.buffer_view.lock().get_edit_state_mut().redo()
    }
}

impl ClipboardHandler for AnsiEditor {
    fn can_cut(&self) -> bool {
        self.buffer_view.lock().get_selection().is_some()
    }
    fn cut(&mut self) -> EngineResult<()> {
        let _cut = self.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-cut"));
        self.copy()?;
        self.buffer_view
            .lock()
            .get_edit_state_mut()
            .delete_selection();
        Ok(())
    }

    fn can_copy(&self) -> bool {
        self.buffer_view.lock().get_selection().is_some()
    }

    fn copy(&mut self) -> EngineResult<()> {
        if let Some(data) = self
            .buffer_view
            .lock()
            .get_edit_state_mut()
            .get_clipboard_data()
        {
            push_data(BUFFER_DATA, &data)?;
        }
        Ok(())
    }

    fn can_paste(&self) -> bool {
        pop_data(BUFFER_DATA).is_some() || pop_sixel_image().is_some()
    }

    fn paste(&mut self) -> EngineResult<()> {
        if let Some(data) = pop_data(BUFFER_DATA) {
            self.buffer_view
                .lock()
                .get_edit_state_mut()
                .paste_clipboard_data(&data)?;
        } else if let Some(sixel) = pop_sixel_image() {
            self.buffer_view
                .lock()
                .get_edit_state_mut()
                .paste_sixel(sixel)?;
        }
        Ok(())
    }
}

impl Document for AnsiEditor {
    fn get_title(&self) -> String {
        if let Some(file_name) = &self.buffer_view.lock().get_buffer().file_name {
            file_name.file_name().unwrap().to_str().unwrap().to_string()
        } else {
            "Untitled".to_string()
        }
    }

    fn is_dirty(&self) -> bool {
        self.dirty_pos != self.buffer_view.lock().get_edit_state().undo_stack_len()
    }

    fn save(&mut self, file_name: &str) -> TerminalResult<()> {
        let file = PathBuf::from(file_name);
        let options = SaveOptions::new();
        let bytes = self
            .buffer_view
            .lock()
            .get_buffer()
            .to_bytes(file.extension().unwrap().to_str().unwrap(), &options)?;
        fs::write(file_name, bytes)?;
        self.dirty_pos = self.buffer_view.lock().get_edit_state().undo_stack_len();
        Ok(())
    }

    fn show_ui(
        &mut self,
        ui: &mut egui::Ui,
        cur_tool: &mut Box<dyn Tool>,
        selected_tool: usize,
        options: &DocumentOptions,
    ) -> Option<Message> {
        let mut message = None;

        let mut scale = options.scale;
        if self.buffer_view.lock().get_buffer().use_aspect_ratio() {
            scale.y *= 1.35;
        }
        if selected_tool != FIRST_TOOL {
            self.buffer_view.lock().clear_selection();
        }

        ui.allocate_ui(
            Vec2::new(ui.available_width(), ui.available_height() - 35.0),
            |ui| {
                let opt = icy_engine_egui::TerminalOptions {
                    focus_lock: false,
                    stick_to_bottom: false,
                    scale: Some(scale),
                    settings: MonitorSettings {
                        background_effect: BackgroundEffect::Checkers,
                        ..Default::default()
                    },
                    id: Some(Id::new(self.id + 10000)),
                    ..Default::default()
                };
                let (response, calc) = show_terminal_area(ui, self.buffer_view.clone(), opt);

                let response = response.context_menu(|ui| {
                    message = terminal_context_menu(self, ui);
                });
                self.handle_response(ui, response, calc, cur_tool)
            },
        );
        self.show_toolbar(ui);
        message
    }

    fn get_ansi_editor_mut(&mut self) -> Option<&mut AnsiEditor> {
        Some(self)
    }

    fn get_ansi_editor(&self) -> Option<&AnsiEditor> {
        Some(self)
    }

    fn destroy(&self, gl: &glow::Context) {
        self.buffer_view.lock().destroy(gl);
    }
}

impl AnsiEditor {
    pub fn new(gl: &Arc<glow::Context>, id: usize, buf: Buffer) -> Self {
        let buffer_view = Arc::new(Mutex::new(BufferView::from_buffer(
            gl,
            buf,
            glow::NEAREST as i32,
        )));
        // let buffer_parser = ansi::Parser::default();
        AnsiEditor {
            id,
            buffer_view,
            cur_outline: 0,
            is_inactive: false,
            reference_image: None,

            dirty_pos: 0,
            drag_pos: DragPos::default(),
            egui_id: Id::new(id),
        }
    }

    pub fn get_cur_layer_index(&self) -> usize {
        self.buffer_view
            .lock()
            .get_edit_state_mut()
            .get_current_layer()
    }

    pub fn set_cur_layer_index(&self, layer: usize) {
        self.buffer_view
            .lock()
            .get_edit_state_mut()
            .set_current_layer(layer);
    }

    pub fn output_string(&mut self, str: &str) {
        for ch in str.chars() {
            let translated_char = self
                .buffer_view
                .lock()
                .get_edit_state()
                .get_parser()
                .convert_from_unicode(ch, 0);
            if let Err(err) = self.print_char(translated_char as u8) {
                eprintln!("{}", err);
            }
        }
    }

    pub fn print_char(&mut self, c: u8) -> Result<(), Box<dyn std::error::Error>> {
        self.buffer_view
            .lock()
            .print_char(unsafe { char::from_u32_unchecked(c as u32) })?;
        self.buffer_view.lock().redraw_view();
        Ok(())
    }
    pub fn get_caret_position(&self) -> Position {
        self.buffer_view.lock().get_caret().get_position()
    }

    pub fn set_caret_position(&mut self, pos: Position) {
        let buffer_view = &mut self.buffer_view.lock();
        let pos = Position::new(
            min(buffer_view.get_width() - 1, max(0, pos.x)),
            min(buffer_view.get_buffer().get_line_count() - 1, max(0, pos.y)),
        );
        buffer_view.get_caret_mut().set_position(pos);
        //(self.pos_changed)(self, pos);
    }

    pub fn set_caret_attribute(&mut self, attr: TextAttribute) {
        if attr == self.buffer_view.lock().get_caret().get_attribute() {
            return;
        }

        self.buffer_view.lock().get_caret_mut().set_attr(attr);
        // (self.attr_changed)(attr);
    }

    pub fn join_overlay(&mut self, description: impl Into<String>) {
        let _undo = self.begin_atomic_undo(description.into());
        let opt_layer = self.buffer_view.lock().get_buffer_mut().remove_overlay();

        if let Some(layer) = &opt_layer {
            for y in 0..layer.lines.len() {
                let line = &layer.lines[y];
                for x in 0..line.chars.len() {
                    let ch = line.chars[x];
                    if ch.is_visible() {
                        self.set_char(Position::new(x as i32, y as i32), ch);
                    }
                }
            }
        }
    }

    pub fn delete_line(&mut self, line: i32) {
        // TODO: Undo
        let mut lock = self.buffer_view.lock();
        let cur_layer = self.get_cur_layer_index();

        let layer = &mut lock.get_buffer_mut().layers[cur_layer];
        layer.remove_line(line);
    }

    pub fn insert_line(&mut self, line: i32) {
        // TODO: Undo
        let mut binding = self.buffer_view.lock();
        let cur_layer = self.get_cur_layer_index();

        let layer = &mut binding.get_buffer_mut().layers[cur_layer];
        layer.insert_line(line, Line::new());
    }

    pub fn pickup_color(&mut self, pos: Position) {
        let ch = self.buffer_view.lock().get_buffer().get_char(pos);
        if ch.is_visible() {
            self.buffer_view
                .lock()
                .get_caret_mut()
                .set_attr(ch.attribute);
        }
    }

    pub fn set_caret(&mut self, x: i32, y: i32) -> Event {
        let old = self.buffer_view.lock().get_caret().get_position();
        let w = self.buffer_view.lock().get_buffer().get_width() - 1;
        let h = self.buffer_view.lock().get_buffer().get_line_count() - 1;
        self.set_caret_position(Position::new(min(max(0, x), w), min(max(0, y), h)));
        Event::CursorPositionChange(old, self.buffer_view.lock().get_caret().get_position())
    }

    pub fn get_cur_outline(&self) -> usize {
        self.cur_outline
    }

    pub fn set_cur_outline(&mut self, outline: usize) {
        self.cur_outline = outline;
        //(self.outline_changed)(self);
    }

    pub fn save_content(&self, file_name: &Path, options: &SaveOptions) -> EngineResult<bool> {
        let mut f = File::create(file_name)?;

        let content = if let Some(ext) = file_name.extension() {
            let ext = OsStr::to_str(ext).unwrap().to_lowercase();
            self.buffer_view
                .lock()
                .get_buffer()
                .to_bytes(ext.as_str(), options)?
        } else {
            self.buffer_view
                .lock()
                .get_buffer()
                .to_bytes("icd", options)?
        };
        f.write_all(&content)?;
        Ok(true)
    }

    pub fn get_outline_char_code(&self, i: usize) -> Result<u16, &str> {
        if self.cur_outline >= DEFAULT_OUTLINE_TABLE.len() {
            return Err("current outline out of range.");
        }
        if !(0..=10).contains(&i) {
            return Err("outline char# out of range.");
        }

        Ok(DEFAULT_OUTLINE_TABLE[self.cur_outline][i] as u16)
    }

    pub fn get_outline_char_code_from(outline: usize, i: usize) -> Result<u16, &'static str> {
        if outline >= DEFAULT_OUTLINE_TABLE.len() {
            return Err("current outline out of range.");
        }
        if !(0..=10).contains(&i) {
            return Err("outline char# out of range.");
        }
        Ok(DEFAULT_OUTLINE_TABLE[outline][i] as u16)
    }

    pub fn get_char(&self, pos: Position) -> AttributedChar {
        self.buffer_view.lock().get_buffer().get_char(pos)
    }

    pub fn get_char_from_cur_layer(&self, pos: Position) -> AttributedChar {
        if self.get_cur_layer_index() >= self.buffer_view.lock().get_buffer().layers.len() {
            return AttributedChar::invisible();
        }
        let cur_layer = self.get_cur_layer_index();
        self.buffer_view.lock().get_buffer().layers[cur_layer].get_char(pos)
    }

    pub fn set_char(&mut self, pos: impl Into<Position>, attributed_char: AttributedChar) {
        let _ = self
            .buffer_view
            .lock()
            .get_edit_state_mut()
            .set_char(pos, attributed_char);
    }

    #[must_use]
    pub fn begin_atomic_undo(&mut self, description: impl Into<String>) -> AtomicUndoGuard {
        self.buffer_view
            .lock()
            .get_edit_state_mut()
            .begin_atomic_undo(description.into())
    }

    pub fn fill(&mut self, rect: Rectangle, dos_char: AttributedChar) {
        let mut pos = rect.start;
        let _undo = self.begin_atomic_undo("Fill");
        for _ in 0..rect.size.height {
            for _ in 0..rect.size.width {
                self.set_char(pos, dos_char);
                pos.x += 1;
            }
            pos.y += 1;
            pos.x = rect.start.x;
        }
    }

    pub fn type_key(&mut self, char_code: char) {
        let pos = self.buffer_view.lock().get_caret().get_position();
        if self.buffer_view.lock().get_caret().insert_mode {
            let start = self.buffer_view.lock().get_buffer().get_width() - 1;
            for i in start..=pos.x {
                let next = self.get_char_from_cur_layer(Position::new(i - 1, pos.y));
                self.set_char(Position::new(i, pos.y), next);
            }
        }
        let attr = self.buffer_view.lock().get_caret().get_attribute();
        self.set_char(pos, AttributedChar::new(char_code, attr));
        self.set_caret(pos.x + 1, pos.y);
    }

    pub fn delete_selection(&mut self) {
        self.buffer_view
            .lock()
            .get_edit_state_mut()
            .delete_selection();
    }

    pub fn redraw_view(&self) {
        self.buffer_view.lock().redraw_view();
    }

    pub fn switch_fg_bg_color(&mut self) {
        let mut attr = self.buffer_view.lock().get_caret().get_attribute();
        let bg = attr.get_background();
        attr.set_background(attr.get_foreground());
        attr.set_foreground(bg);
        self.set_caret_attribute(attr);
    }

    pub fn erase_line(&mut self) {
        let _undo = self.begin_atomic_undo("Erase line");
        // TODO
    }

    pub fn erase_line_to_start(&mut self) {
        let _undo = self.begin_atomic_undo("Erase line to start");
        // TODO
    }

    pub fn erase_line_to_end(&mut self) {
        let _undo = self.begin_atomic_undo("Erase line to end");
        // TODO
    }

    pub fn erase_column(&mut self) {
        let _undo = self.begin_atomic_undo("Erase column");
        // TODO
    }

    pub fn erase_column_to_start(&mut self) {
        let _undo = self.begin_atomic_undo("Erase column to start");
        // TODO
    }

    pub fn erase_column_to_end(&mut self) {
        let _undo = self.begin_atomic_undo("Erase column to end");
        // TODO
    }

    pub fn delete_row(&mut self) {
        let _undo = self.begin_atomic_undo("Delete row");
        // TODO
    }

    pub fn insert_row(&mut self) {
        let _undo = self.begin_atomic_undo("Insert row");
        // TODO
    }

    pub fn delete_column(&mut self) {
        let _undo = self.begin_atomic_undo("Delete column");
        // TODO
    }

    pub fn insert_column(&mut self) {
        let _undo = self.begin_atomic_undo("Insert column");
        // TODO
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let pos = self.buffer_view.lock().get_caret().get_position();

            let label_font_size = 20.0;

            ui.vertical(|ui| {
                ui.add_space(4.);
                ui.label(
                    RichText::new(fl!(
                        crate::LANGUAGE_LOADER,
                        "toolbar-position",
                        line = pos.y,
                        column = pos.x
                    ))
                    .font(FontId::proportional(label_font_size)),
                );
            });

            let r = ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let cur_outline = self.cur_outline;
                let cur_font_page = self.buffer_view.lock().get_caret().get_font_page();

                let button_font_size = 16.0;
                if ui
                    .selectable_label(
                        false,
                        RichText::new("▶").font(FontId::proportional(button_font_size)),
                    )
                    .clicked()
                {
                    self.cur_outline = (cur_outline + 1) % DEFAULT_OUTLINE_TABLE.len();
                }
                ui.label(
                    RichText::new((cur_outline + 1).to_string())
                        .font(FontId::proportional(label_font_size)),
                );

                if ui
                    .selectable_label(
                        false,
                        RichText::new("◀").font(FontId::proportional(button_font_size)),
                    )
                    .clicked()
                {
                    self.cur_outline = (cur_outline + DEFAULT_OUTLINE_TABLE.len() - 1)
                        % DEFAULT_OUTLINE_TABLE.len();
                }

                for i in (0..10).rev() {
                    let ch = self.get_outline_char_code(i).unwrap();
                    ui.add(crate::model::pencil_imp::draw_glyph_plain(
                        self,
                        unsafe { char::from_u32_unchecked(ch as u32) },
                        cur_font_page,
                    ));

                    ui.label(
                        RichText::new(format!("F{}", i + 1))
                            .font(FontId::proportional(label_font_size)),
                    );
                }
            });
            r.response
        });
    }

    fn handle_response(
        &mut self,
        ui: &egui::Ui,
        mut response: Response,
        calc: TerminalCalc,
        cur_tool: &mut Box<dyn Tool>,
    ) -> Response {
        if response.has_focus() {
            let events = ui.input(|i| i.events.clone());
            for e in &events {
                match e {
                    egui::Event::Copy => {
                        let buffer_view = self.buffer_view.clone();
                        let mut l = buffer_view.lock();
                        if let Some(txt) = l.get_copy_text() {
                            ui.output_mut(|o| o.copied_text = txt);
                        }
                    }
                    egui::Event::Cut => {}
                    egui::Event::Paste(text) => {
                        self.output_string(text);
                        self.buffer_view.lock().redraw_view();
                    }

                    egui::Event::CompositionEnd(text) | egui::Event::Text(text) => {
                        for c in text.chars() {
                            cur_tool.handle_key(self, MKey::Character(c as u16), MModifiers::None);
                        }
                        self.redraw_view();
                    }

                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => {
                        let mut key_code = *key as u32;
                        if modifiers.ctrl || modifiers.command {
                            key_code |= CTRL_MOD;
                        }
                        if modifiers.shift {
                            key_code |= SHIFT_MOD;
                        }

                        let mut modifier: MModifiers = MModifiers::None;
                        if modifiers.ctrl || modifiers.command {
                            modifier = MModifiers::Control;
                        }

                        if modifiers.shift {
                            modifier = MModifiers::Shift;
                        }
                        for (k, m) in ANSI_KEY_MAP {
                            if *k == key_code {
                                cur_tool.handle_key(self, *m, modifier);
                                self.buffer_view.lock().redraw_view();
                                ui.input_mut(|i| i.consume_key(*modifiers, *key));
                                self.redraw_view();
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if response.clicked_by(egui::PointerButton::Primary) {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if calc.buffer_rect.contains(mouse_pos) {
                    let click_pos = calc.calc_click_pos(mouse_pos);
                    let cp: Position = Position::new(click_pos.x as i32, click_pos.y as i32)
                        - self.get_cur_click_offset();
                    /*
                    let b: i32 = match responsee.b {
                                     PointerButton::Primary => 1,
                                     PointerButton::Secondary => 2,
                                     PointerButton::Middle => 3,
                                     PointerButton::Extra1 => 4,
                                     PointerButton::Extra2 => 5,
                                 }; */
                    cur_tool.handle_click(self, 1, cp);
                    self.redraw_view();
                }
            }
        }

        if response.drag_started_by(egui::PointerButton::Primary) {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if calc.buffer_rect.contains(mouse_pos) {
                    let click_pos = calc.calc_click_pos(mouse_pos);
                    let click_pos = Position::new(click_pos.x as i32, click_pos.y as i32);
                    
                    let cp: Position = click_pos 
                        - self.get_cur_click_offset();
                    self.drag_pos.start_abs = click_pos;
                    self.drag_pos.start = cp;

                    self.drag_pos.cur_abs = click_pos;
                    self.drag_pos.cur = cp;
    
                    cur_tool.handle_drag_begin(self);
                }
            }
            self.redraw_view();
        }

        if response.dragged_by(egui::PointerButton::Primary) {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                let click_pos = calc.calc_click_pos(mouse_pos);
                let click_pos = Position::new(click_pos.x as i32, click_pos.y as i32);

                let mut c = self.drag_pos.cur;
                let mut c_abs = self.drag_pos.cur_abs;
                while c_abs != click_pos {
                    let s = (click_pos - c_abs).signum();
                    c += s;
                    c_abs += s;
                    self.drag_pos.cur_abs = c_abs;
                    self.drag_pos.cur = c;

                    response = cur_tool.handle_drag(ui, response, self, &calc);
                }
            }
            self.redraw_view();
        }

        if response.hovered() {
            if let Some(mouse_pos) = response.hover_pos() {
                if calc.buffer_rect.contains(mouse_pos) {
                    let click_pos = calc.calc_click_pos(mouse_pos);
                    let cp = Position::new(click_pos.x as i32, click_pos.y as i32)
                        - self.get_cur_click_offset();
                    response = cur_tool.handle_hover(ui, response, self, cp);
                }
            }
        }

        if response.drag_released_by(egui::PointerButton::Primary) {
            cur_tool.handle_drag_end(self);
            self.redraw_view();
        }

        response
    }

    fn get_cur_click_offset(&mut self) -> Position {
        if let Some(layer) = self.buffer_view.lock().get_edit_state().get_cur_layer() {
            return layer.get_offset();
        }
        Position::default()
    }

    pub(crate) fn set_file_name(&self, file_name: impl Into<PathBuf>) {
        self.buffer_view.lock().get_buffer_mut().file_name = Some(file_name.into());
    }

    pub(crate) fn clear_overlay_layer(&self) {
        let cur_offset = self
            .buffer_view
            .lock()
            .get_edit_state()
            .get_cur_layer()
            .unwrap()
            .get_offset();

        if let Some(layer) = self.buffer_view.lock().get_buffer_mut().get_overlay_layer() {
            layer.set_offset(cur_offset);
            layer.clear();
        }
    }
}

pub const DEFAULT_OUTLINE_TABLE: [[u8; 10]; 15] = [
    [218, 191, 192, 217, 196, 179, 195, 180, 193, 194],
    [201, 187, 200, 188, 205, 186, 204, 185, 202, 203],
    [213, 184, 212, 190, 205, 179, 198, 181, 207, 209],
    [214, 183, 211, 189, 196, 186, 199, 182, 208, 210],
    [197, 206, 216, 215, 232, 233, 155, 156, 153, 239],
    [176, 177, 178, 219, 223, 220, 221, 222, 254, 250],
    [1, 2, 3, 4, 5, 6, 240, 127, 14, 15],
    [24, 25, 30, 31, 16, 17, 18, 29, 20, 21],
    [174, 175, 242, 243, 169, 170, 253, 246, 171, 172],
    [227, 241, 244, 245, 234, 157, 228, 248, 251, 252],
    [224, 225, 226, 229, 230, 231, 235, 236, 237, 238],
    [128, 135, 165, 164, 152, 159, 247, 249, 173, 168],
    [131, 132, 133, 160, 166, 134, 142, 143, 145, 146],
    [136, 137, 138, 130, 144, 140, 139, 141, 161, 158],
    [147, 148, 149, 162, 167, 150, 129, 151, 163, 154],
];

pub fn terminal_context_menu(editor: &mut AnsiEditor, ui: &mut egui::Ui) -> Option<Message> {
    let mut result = None;
    ui.input_mut(|i| i.events.clear());

    if ui.button(fl!(crate::LANGUAGE_LOADER, "menu-cut")).clicked() {
        result = Some(Message::Cut);
        ui.close_menu();
    }

    if ui
        .button(fl!(crate::LANGUAGE_LOADER, "menu-copy"))
        .clicked()
    {
        result = Some(Message::Copy);
        ui.close_menu();
    }

    if ui
        .button(fl!(crate::LANGUAGE_LOADER, "menu-paste"))
        .clicked()
    {
        result = Some(Message::Paste);
        ui.close_menu();
    }

    let sel = editor.buffer_view.lock().get_selection();

    if let Some(_sel) = sel {
        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-erase"))
            .clicked()
        {
            editor.delete_selection();
            ui.close_menu();
        }
        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-flipx"))
            .clicked()
        {
            result = Some(Message::FlipX);
            ui.close_menu();
        }
        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-flipy"))
            .clicked()
        {
            result = Some(Message::FlipY);
            ui.close_menu();
        }

        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-justifyleft"))
            .clicked()
        {
            result = Some(Message::JustifyLeft);
            ui.close_menu();
        }
        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-justifyright"))
            .clicked()
        {
            result = Some(Message::JustifyRight);
            ui.close_menu();
        }
        if ui
            .button(fl!(crate::LANGUAGE_LOADER, "menu-justifycenter"))
            .clicked()
        {
            result = Some(Message::Center);
            ui.close_menu();
        }
    }
    result
}

pub const CTRL_MOD: u32 = 0b1000_0000_0000_0000_0000;
pub const SHIFT_MOD: u32 = 0b0100_0000_0000_0000_0000;

pub static ANSI_KEY_MAP: &[(u32, MKey)] = &[
    (Key::Escape as u32, MKey::Escape),
    (Key::Home as u32, MKey::Home),
    (Key::Insert as u32, MKey::Insert),
    (Key::Backspace as u32, MKey::Backspace),
    (Key::Enter as u32, MKey::Return),
    (Key::Tab as u32, MKey::Tab),
    (Key::Delete as u32, MKey::Delete),
    (Key::End as u32, MKey::End),
    (Key::PageUp as u32, MKey::PageUp),
    (Key::PageDown as u32, MKey::PageDown),
    (Key::F1 as u32, MKey::F1),
    (Key::F2 as u32, MKey::F2),
    (Key::F3 as u32, MKey::F3),
    (Key::F4 as u32, MKey::F4),
    (Key::F5 as u32, MKey::F5),
    (Key::F6 as u32, MKey::F6),
    (Key::F7 as u32, MKey::F7),
    (Key::F8 as u32, MKey::F8),
    (Key::F9 as u32, MKey::F9),
    (Key::F10 as u32, MKey::F10),
    (Key::F11 as u32, MKey::F11),
    (Key::F12 as u32, MKey::F12),
    (Key::ArrowUp as u32, MKey::Up),
    (Key::ArrowDown as u32, MKey::Down),
    (Key::ArrowRight as u32, MKey::Right),
    (Key::ArrowLeft as u32, MKey::Left),
    (Key::A as u32 | CTRL_MOD, MKey::Character(1)),
    (Key::B as u32 | CTRL_MOD, MKey::Character(2)),
    (Key::C as u32 | CTRL_MOD, MKey::Character(3)),
    (Key::D as u32 | CTRL_MOD, MKey::Character(4)),
    (Key::E as u32 | CTRL_MOD, MKey::Character(5)),
    (Key::F as u32 | CTRL_MOD, MKey::Character(6)),
    (Key::G as u32 | CTRL_MOD, MKey::Character(7)),
    (Key::H as u32 | CTRL_MOD, MKey::Character(8)),
    (Key::I as u32 | CTRL_MOD, MKey::Character(9)),
    (Key::J as u32 | CTRL_MOD, MKey::Character(10)),
    (Key::K as u32 | CTRL_MOD, MKey::Character(11)),
    (Key::L as u32 | CTRL_MOD, MKey::Character(12)),
    (Key::M as u32 | CTRL_MOD, MKey::Character(13)),
    (Key::N as u32 | CTRL_MOD, MKey::Character(14)),
    (Key::O as u32 | CTRL_MOD, MKey::Character(15)),
    (Key::P as u32 | CTRL_MOD, MKey::Character(16)),
    (Key::Q as u32 | CTRL_MOD, MKey::Character(17)),
    (Key::R as u32 | CTRL_MOD, MKey::Character(18)),
    (Key::S as u32 | CTRL_MOD, MKey::Character(19)),
    (Key::T as u32 | CTRL_MOD, MKey::Character(20)),
    (Key::U as u32 | CTRL_MOD, MKey::Character(21)),
    (Key::V as u32 | CTRL_MOD, MKey::Character(22)),
    (Key::W as u32 | CTRL_MOD, MKey::Character(23)),
    (Key::X as u32 | CTRL_MOD, MKey::Character(24)),
    (Key::Y as u32 | CTRL_MOD, MKey::Character(25)),
    (Key::Z as u32 | CTRL_MOD, MKey::Character(26)),
    (Key::Num2 as u32 | CTRL_MOD, MKey::Character(0)),
    (Key::Num3 as u32 | CTRL_MOD, MKey::Character(0x1B)),
    (Key::Num4 as u32 | CTRL_MOD, MKey::Character(0x1C)),
    (Key::Num5 as u32 | CTRL_MOD, MKey::Character(0x1D)),
    (Key::Num6 as u32 | CTRL_MOD, MKey::Character(0x1E)),
    (Key::Num7 as u32 | CTRL_MOD, MKey::Character(0x1F)),
];
