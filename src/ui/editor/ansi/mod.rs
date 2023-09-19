use std::{
    cmp::{max, min},
    ffi::OsStr,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use eframe::{
    egui::{self, Id, Key, Response},
    epaint::{mutex::Mutex, Vec2},
};
use i18n_embed_fl::fl;
use icy_engine::{
    editor::{AtomicUndoGuard, UndoState},
    util::{pop_data, pop_sixel_image, push_data, BUFFER_DATA},
    AttributedChar, Buffer, EngineResult, Line, Position, Rectangle, SaveOptions, TextAttribute,
    TextPane,
};

use icy_engine_egui::{show_terminal_area, BufferView, TerminalCalc};

use crate::{
    model::{DragPos, MKey, MModifiers, Tool},
    ClipboardHandler, Commands, Document, DocumentOptions, Message, SavingError, Settings,
    TerminalResult, UndoHandler, SETTINGS,
};

pub enum Event {
    None,
    CursorPositionChange(Position, Position),
}

pub struct AnsiEditor {
    pub id: usize,
    pub drag_pos: DragPos,
    drag_started: bool,
    pub buffer_view: Arc<eframe::epaint::mutex::Mutex<BufferView>>,
    pub is_inactive: bool,

    pub outline_font_mode: bool,

    pub reference_image: Option<PathBuf>,
    // pub outline_changed: std::boxed::Box<dyn Fn(&Editor)>,
    //pub request_refresh: Box<dyn Fn ()>,
    pub egui_id: Id,
    pub next_scroll_position: Option<f32>,
    pub guide: Option<Vec2>,
    pub raster: Option<Vec2>, //pub pos_changed: std::boxed::Box<dyn Fn(&Editor, Position)>,
                              //pub attr_changed: std::boxed::Box<dyn Fn(TextAttribute)>
}

impl UndoHandler for AnsiEditor {
    fn undo_description(&self) -> Option<String> {
        self.buffer_view.lock().get_edit_state().undo_description()
    }

    fn can_undo(&self) -> bool {
        self.buffer_view.lock().get_edit_state().can_undo()
    }

    fn undo(&mut self) -> EngineResult<Option<Message>> {
        self.buffer_view.lock().get_edit_state_mut().undo()?;
        Ok(None)
    }

    fn redo_description(&self) -> Option<String> {
        self.buffer_view.lock().get_edit_state().redo_description()
    }

    fn can_redo(&self) -> bool {
        self.buffer_view.lock().get_edit_state().can_redo()
    }

    fn redo(&mut self) -> EngineResult<Option<Message>> {
        self.buffer_view.lock().get_edit_state_mut().redo()?;
        Ok(None)
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
            .erase_selection()?;
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
        } else {
            log::error!("can't get clipboard data!");
        }
        Ok(())
    }

    fn can_paste(&self) -> bool {
        pop_data(BUFFER_DATA).is_some() || pop_sixel_image().is_some()
    }

    fn paste(&mut self) -> EngineResult<()> {
        if self
            .buffer_view
            .lock()
            .get_edit_state_mut()
            .has_floating_layer()
        {
            return Ok(());
        }

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
const ICED_EXT: &str = "icy";

impl Document for AnsiEditor {
    fn default_extension(&self) -> &'static str {
        ICED_EXT
    }

    fn undo_stack_len(&self) -> usize {
        if let Ok(stack) = self
            .buffer_view
            .lock()
            .get_edit_state()
            .get_undo_stack()
            .lock()
        {
            for i in (0..stack.len()).rev() {
                if stack[i].changes_data() {
                    return i + 1;
                }
            }
        }
        0
    }

    fn get_bytes(&mut self, path: &Path) -> TerminalResult<Vec<u8>> {
        let ext = if let Some(ext) = path.extension() {
            OsStr::to_str(ext).unwrap_or(ICED_EXT).to_lowercase()
        } else {
            ICED_EXT.to_string()
        };
        let options = SaveOptions::new();
        let bytes = self
            .buffer_view
            .lock()
            .get_buffer()
            .to_bytes(&ext, &options)?;
        Ok(bytes)
    }

    fn show_ui(
        &mut self,
        ui: &mut egui::Ui,
        cur_tool: &mut Box<dyn Tool>,
        _selected_tool: usize,
        options: &DocumentOptions,
    ) -> Option<Message> {
        let mut message = None;

        let mut scale = options.get_scale();
        if self.buffer_view.lock().get_buffer().use_aspect_ratio() {
            scale.y *= 1.35;
        }
        let opt = icy_engine_egui::TerminalOptions {
            stick_to_bottom: false,
            scale: Some(scale),
            fit_width: options.fit_width,
            monitor_settings: unsafe { SETTINGS.monitor_settings.clone() },
            marker_settings: unsafe { SETTINGS.marker_settings.clone() },
            id: Some(Id::new(self.id + 10000)),
            raster: self.raster,
            guide: self.guide,
            scroll_offset: self.next_scroll_position.take(),
            show_layer_borders: unsafe { SETTINGS.show_layer_borders },
            show_line_numbers: unsafe { SETTINGS.show_line_numbers },
            ..Default::default()
        };
        let (response, calc) = show_terminal_area(ui, self.buffer_view.clone(), opt);

        let response = response.context_menu(|ui| {
            message = terminal_context_menu(self, &options.commands, ui);
        });
        self.handle_response(ui, response, calc, cur_tool, &mut message);

        message
    }

    fn get_ansi_editor_mut(&mut self) -> Option<&mut AnsiEditor> {
        Some(self)
    }

    fn get_ansi_editor(&self) -> Option<&AnsiEditor> {
        Some(self)
    }

    fn destroy(&self, gl: &glow::Context) -> Option<Message> {
        self.buffer_view.lock().destroy(gl);
        None
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
            is_inactive: false,
            reference_image: None,
            drag_started: false,
            drag_pos: DragPos::default(),
            egui_id: Id::new(id),
            guide: None,
            raster: None,
            outline_font_mode: false,
            next_scroll_position: None,
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
            self.type_key(ch);
        }
    }

    pub fn get_caret_position(&self) -> Position {
        self.buffer_view.lock().get_caret().get_position()
    }

    pub fn set_caret_position(&mut self, pos: Position) {
        let buffer_view = &mut self.buffer_view.lock();
        let pos = Position::new(
            min(buffer_view.get_width() - 1, max(0, pos.x)),
            min(buffer_view.get_buffer().get_height() - 1, max(0, pos.y)),
        );
        buffer_view.get_caret_mut().set_position(pos);
        buffer_view.reset_caret_blink();
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
        let use_selection = self
            .buffer_view
            .lock()
            .get_edit_state()
            .is_something_selected();

        if let Some(layer) = &opt_layer {
            for y in 0..layer.lines.len() {
                let line = &layer.lines[y];
                for x in 0..line.chars.len() {
                    let ch = line.chars[x];
                    let pos = Position::new(x as i32, y as i32);
                    if ch.is_visible()
                        && (!use_selection
                            || self
                                .buffer_view
                                .lock()
                                .get_edit_state()
                                .get_is_selected(pos + layer.get_offset()))
                    {
                        self.set_char(pos, ch);
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
        let h = self.buffer_view.lock().get_buffer().get_height() - 1;

        let char_scroll_position = self.buffer_view.lock().calc.char_scroll_positon;
        let terminal_height = self.buffer_view.lock().calc.terminal_rect.height();
        let buffer_height = self
            .buffer_view
            .lock()
            .calc
            .buffer_rect
            .height()
            .min(terminal_height);
        let buffer_char_height = buffer_height / self.buffer_view.lock().calc.scale.y;

        let y = min(max(0, y), h);
        self.set_caret_position(Position::new(min(max(0, x), w), y));

        let char_height = self
            .buffer_view
            .lock()
            .get_buffer()
            .get_font_dimensions()
            .height as f32;
        let y = y as f32 * char_height;

        if y < char_scroll_position {
            self.next_scroll_position = Some(y);
        }

        if y > (char_scroll_position + buffer_char_height - char_height) {
            self.next_scroll_position = Some((y - buffer_char_height + char_height).max(0.0));
        }

        Event::CursorPositionChange(old, self.buffer_view.lock().get_caret().get_position())
    }

    pub fn save_content(&self, file_name: &Path, options: &SaveOptions) -> EngineResult<bool> {
        match File::create(file_name) {
            Ok(mut f) => {
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
                        .to_bytes(ICED_EXT, options)?
                };
                if let Err(err) = f.write_all(&content) {
                    return Err(SavingError::ErrorWritingFile(format!("{err}")).into());
                }
            }
            Err(err) => {
                return Err(SavingError::ErrorCreatingFile(format!("{err}")).into());
            }
        }

        Ok(true)
    }

    pub fn get_outline_char_code(&self, i: usize) -> Result<u16, &str> {
        let outline = Settings::get_character_set();
        if outline >= DEFAULT_OUTLINE_TABLE.len() {
            return Err("current outline out of range.");
        }
        if !(0..=10).contains(&i) {
            return Err("outline char# out of range.");
        }

        Ok(DEFAULT_OUTLINE_TABLE[outline][i] as u16)
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
            let end = self.buffer_view.lock().get_buffer().get_width();
            for i in (pos.x..end).rev() {
                let next = self.get_char_from_cur_layer(Position::new(i - 1, pos.y));
                self.set_char(Position::new(i, pos.y), next);
            }
        }
        let attr = self.buffer_view.lock().get_caret().get_attribute();
        self.set_char(pos, AttributedChar::new(char_code, attr));
        self.set_caret(pos.x + 1, pos.y);
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

    fn handle_response(
        &mut self,
        ui: &egui::Ui,
        mut response: Response,
        calc: TerminalCalc,
        cur_tool: &mut Box<dyn Tool>,
        message: &mut Option<Message>,
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
                        if !ui.input(|i| i.modifiers.ctrl || i.modifiers.command || i.modifiers.alt)
                        {
                            for c in text.chars() {
                                cur_tool.handle_key(
                                    self,
                                    MKey::Character(c as u16),
                                    MModifiers::None,
                                );
                            }
                            self.redraw_view();
                        }
                    }

                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => {
                        let mut key_code = *key as u32;
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
                        for (k, m) in EDITOR_KEY_MAP {
                            if *k == key_code {
                                cur_tool.handle_key(self, *m, modifier);
                                self.buffer_view.lock().redraw_view();
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
                if calc.buffer_rect.contains(mouse_pos) && !calc.scrollbar_rect.contains(mouse_pos)
                {
                    let click_pos = calc.calc_click_pos(mouse_pos);
                    let cp_abs = Position::new(click_pos.x as i32, click_pos.y as i32);
                    let cp = cp_abs - self.get_cur_click_offset();
                    /*
                    let b: i32 = match responsee.b {
                                     PointerButton::Primary => 1,
                                     PointerButton::Secondary => 2,
                                     PointerButton::Middle => 3,
                                     PointerButton::Extra1 => 4,
                                     PointerButton::Extra2 => 5,
                                 }; */
                    let msg = cur_tool.handle_click(self, 1, cp, cp_abs, &response);
                    if message.is_none() {
                        *message = msg;
                    }
                    self.redraw_view();
                }
            }
        }

        if response.drag_started_by(egui::PointerButton::Primary) {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if calc.buffer_rect.contains(mouse_pos) && !calc.scrollbar_rect.contains(mouse_pos)
                {
                    let click_pos = calc.calc_click_pos(mouse_pos);
                    let click_pos = Position::new(click_pos.x as i32, click_pos.y as i32);

                    let cp: Position = click_pos - self.get_cur_click_offset();
                    self.drag_pos.start_abs = click_pos;
                    self.drag_pos.start = cp;

                    self.drag_pos.cur_abs = click_pos;
                    self.drag_pos.cur = cp;
                    self.drag_started = true;
                    cur_tool.handle_drag_begin(self, &response);
                }
            }
            self.redraw_view();
        }

        if response.dragged_by(egui::PointerButton::Primary) && self.drag_started {
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
        self.buffer_view
            .lock()
            .get_edit_state_mut()
            .get_tool_overlay_mask_mut()
            .clear();
        if response.hovered() {
            if let Some(mouse_pos) = response.hover_pos() {
                if calc.buffer_rect.contains(mouse_pos) && !calc.scrollbar_rect.contains(mouse_pos)
                {
                    let click_pos = calc.calc_click_pos(mouse_pos);
                    let cp_abs = Position::new(click_pos.x as i32, click_pos.y as i32);
                    let cp = cp_abs - self.get_cur_click_offset();
                    response = cur_tool.handle_hover(ui, response, self, cp, cp_abs);
                } else {
                    cur_tool.handle_no_hover(self);
                }
            } else {
                cur_tool.handle_no_hover(self);
            }
        } else {
            cur_tool.handle_no_hover(self);
        }

        if response.drag_released_by(egui::PointerButton::Primary) {
            cur_tool.handle_drag_end(self);
            self.drag_started = false;
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

    pub fn backspace(&mut self) {
        self.buffer_view.lock().clear_selection();
        let pos = self.get_caret_position();
        if pos.x > 0 {
            self.set_caret_position(pos + Position::new(-1, 0));
            if self.buffer_view.lock().get_caret().insert_mode {
                let end = self.buffer_view.lock().get_width() - 1;
                for i in pos.x..end {
                    let next = self.get_char_from_cur_layer(Position::new(i + 1, pos.y));
                    self.set_char(Position::new(i, pos.y), next);
                }
                let last_pos = Position::new(self.buffer_view.lock().get_width() - 1, pos.y);
                self.set_char(last_pos, AttributedChar::invisible());
            } else {
                let pos = self.get_caret_position();
                self.set_char(pos, AttributedChar::invisible());
            }
        }
    }
    pub fn delete(&mut self) {
        self.buffer_view.lock().clear_selection();
        let pos = self.get_caret_position();
        if pos.x > 0 {
            let end = self.buffer_view.lock().get_width() - 1;
            for i in pos.x..end {
                let next = self.get_char_from_cur_layer(Position::new(i + 1, pos.y));
                self.set_char(Position::new(i, pos.y), next);
            }
            let last_pos = Position::new(self.buffer_view.lock().get_width() - 1, pos.y);
            self.set_char(last_pos, AttributedChar::invisible());
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

pub fn terminal_context_menu(
    editor: &AnsiEditor,
    commands: &Commands,
    ui: &mut egui::Ui,
) -> Option<Message> {
    let mut result = None;
    ui.input_mut(|i| i.events.clear());

    commands.cut.ui(ui, &mut result);
    commands.copy.ui(ui, &mut result);
    commands.paste.ui(ui, &mut result);

    let sel = editor.buffer_view.lock().get_selection();

    if let Some(_sel) = sel {
        commands.erase_selection.ui(ui, &mut result);
        commands.flip_x.ui(ui, &mut result);
        commands.flip_y.ui(ui, &mut result);
        commands.justifycenter.ui(ui, &mut result);
        commands.justifyleft.ui(ui, &mut result);
        commands.justifyright.ui(ui, &mut result);
        commands.crop.ui(ui, &mut result);
    }
    result
}

pub const CTRL_MOD: u32 = 0b1000_0000_0000_0000_0000;
pub const SHIFT_MOD: u32 = 0b0100_0000_0000_0000_0000;

pub static EDITOR_KEY_MAP: &[(u32, MKey)] = &[
    (Key::Escape as u32, MKey::Escape),
    (Key::Home as u32, MKey::Home),
    (Key::End as u32, MKey::End),
    (Key::Home as u32 | CTRL_MOD, MKey::Home),
    (Key::End as u32 | CTRL_MOD, MKey::End),
    (Key::Insert as u32, MKey::Insert),
    (Key::Backspace as u32, MKey::Backspace),
    (Key::Enter as u32, MKey::Return),
    (Key::Tab as u32, MKey::Tab),
    (Key::Tab as u32 | SHIFT_MOD, MKey::Tab),
    (Key::Delete as u32, MKey::Delete),
    (Key::PageUp as u32, MKey::PageUp),
    (Key::PageDown as u32, MKey::PageDown),
    (Key::ArrowUp as u32, MKey::Up),
    (Key::ArrowDown as u32, MKey::Down),
    (Key::ArrowRight as u32, MKey::Right),
    (Key::ArrowLeft as u32, MKey::Left),
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
];
