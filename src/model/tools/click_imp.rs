use eframe::egui;
use egui_extras::RetainedImage;
use i18n_embed_fl::fl;
use icy_engine::{editor::AtomicUndoGuard, Rectangle};
use icy_engine_egui::TerminalCalc;

use crate::{
    model::{tools::handle_outline_insertion, MKey},
    AnsiEditor, Message,
};

use super::{Event, MModifiers, Position, Tool};

#[derive(Default)]
enum SelectionDrag {
    #[default]
    None,
    Move,
    Left,
    Right,
    Top,
    Bottom,

    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Default)]
pub struct ClickTool {
    start_selection: Rectangle,
    selection_drag: SelectionDrag,
    undo_op: Option<AtomicUndoGuard>,
}

impl Tool for ClickTool {
    fn get_icon_name(&self) -> &'static RetainedImage {
        &super::icons::CURSOR_SVG
    }

    fn show_ui(
        &mut self,
        _ctx: &egui::Context,
        _ui: &mut egui::Ui,
        _buffer_opt: &AnsiEditor,
    ) -> Option<Message> {
        None
    }

    fn handle_click(
        &mut self,
        editor: &mut AnsiEditor,
        button: i32,
        pos: Position,
        cur_abs: Position,
        _response: &egui::Response,
    ) -> Event {
        if button == 1 && !is_inside_selection(editor, cur_abs) {
            editor.set_caret_position(pos);
            editor.buffer_view.lock().clear_selection();
        }
        Event::None
    }

    fn handle_drag_begin(&mut self, editor: &mut AnsiEditor, _response: &egui::Response) -> Event {
        self.selection_drag = get_selection_drag(editor, editor.drag_pos.start_abs);

        if !matches!(self.selection_drag, SelectionDrag::None) {
            if let Some(selection) = editor.buffer_view.lock().get_selection() {
                self.start_selection = selection.as_rectangle();
            }
        }
        self.undo_op = Some(editor.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-select")));

        Event::None
    }
    fn handle_drag(
        &mut self,
        _ui: &egui::Ui,
        response: egui::Response,
        editor: &mut AnsiEditor,
        _calc: &TerminalCalc,
    ) -> egui::Response {
        let mut rect = if let Some(selection) = editor.buffer_view.lock().get_selection() {
            selection.as_rectangle()
        } else {
            Rectangle::from_coords(0, 0, 0, 0)
        };

        match self.selection_drag {
            SelectionDrag::Move => {
                rect.start = self.start_selection.top_left() - editor.drag_pos.start_abs
                    + editor.drag_pos.cur_abs;
                editor.buffer_view.lock().set_selection(rect);
            }
            SelectionDrag::Left => {
                self.move_left(editor, &mut rect);
                editor.buffer_view.lock().set_selection(rect);
            }
            SelectionDrag::Right => {
                self.move_right(editor, &mut rect);
                editor.buffer_view.lock().set_selection(rect);
            }
            SelectionDrag::Top => {
                self.move_top(editor, &mut rect);
                editor.buffer_view.lock().set_selection(rect);
            }
            SelectionDrag::Bottom => {
                self.move_bottom(editor, &mut rect);
                editor.buffer_view.lock().set_selection(rect);
            }
            SelectionDrag::TopLeft => {
                self.move_left(editor, &mut rect);
                self.move_top(editor, &mut rect);
                editor.buffer_view.lock().set_selection(rect);
            }
            SelectionDrag::TopRight => {
                self.move_right(editor, &mut rect);
                self.move_top(editor, &mut rect);
                editor.buffer_view.lock().set_selection(rect);
            }
            SelectionDrag::BottomLeft => {
                self.move_left(editor, &mut rect);
                self.move_bottom(editor, &mut rect);
                editor.buffer_view.lock().set_selection(rect);
            }
            SelectionDrag::BottomRight => {
                self.move_right(editor, &mut rect);
                self.move_bottom(editor, &mut rect);
                editor.buffer_view.lock().set_selection(rect);
            }

            SelectionDrag::None => {
                if editor.drag_pos.start == editor.drag_pos.cur {
                    editor.buffer_view.lock().clear_selection();
                } else {
                    editor.buffer_view.lock().set_selection(Rectangle::from(
                        editor.drag_pos.start_abs.x.min(editor.drag_pos.cur_abs.x),
                        editor.drag_pos.start_abs.y.min(editor.drag_pos.cur_abs.y),
                        (editor.drag_pos.cur_abs.x - editor.drag_pos.start_abs.x).abs(),
                        (editor.drag_pos.cur_abs.y - editor.drag_pos.start_abs.y).abs(),
                    ));
                }
            }
        }

        let lock = &mut editor.buffer_view.lock();
        if let Some(mut selection) = lock.get_selection() {
            selection.is_negative_selection = response.ctx.input(|i| i.modifiers.command_only());
            lock.set_selection(selection);
        }

        response
    }

    fn handle_hover(
        &mut self,
        ui: &egui::Ui,
        response: egui::Response,
        editor: &mut AnsiEditor,
        _cur: Position,
        cur_abs: Position,
    ) -> egui::Response {
        match get_selection_drag(editor, cur_abs) {
            SelectionDrag::None => ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Text),
            SelectionDrag::Move => ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Move),
            SelectionDrag::Left => ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeWest),
            SelectionDrag::Right => ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeEast),
            SelectionDrag::Top => ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeNorth),
            SelectionDrag::Bottom => {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeSouth);
            }
            SelectionDrag::TopLeft => {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeNorthWest);
            }
            SelectionDrag::TopRight => {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeNorthEast);
            }
            SelectionDrag::BottomLeft => {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeSouthWest);
            }
            SelectionDrag::BottomRight => {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeSouthEast);
            }
        }
        response
    }

    fn handle_drag_end(&mut self, editor: &mut AnsiEditor) -> Event {
        if !matches!(self.selection_drag, SelectionDrag::None) {
            self.selection_drag = SelectionDrag::None;
            self.undo_op = None;
            return Event::None;
        }

        let mut cur = editor.drag_pos.cur;
        if editor.drag_pos.start < cur {
            cur += Position::new(1, 1);
        }

        if editor.drag_pos.start == cur {
            editor.buffer_view.lock().clear_selection();
        }
        self.undo_op = None;

        Event::None
    }

    fn handle_key(&mut self, editor: &mut AnsiEditor, key: MKey, modifier: MModifiers) -> Event {
        // TODO Keys:

        // Tab - Next tab
        // Shift+Tab - Prev tab

        // ctrl+pgup  - upper left corner
        // ctrl+pgdn  - lower left corner

        let pos = editor.buffer_view.lock().get_caret().get_position();
        match key {
            MKey::Down => {
                if let MModifiers::Control = modifier {
                    let fg = (editor
                        .buffer_view
                        .lock()
                        .get_caret()
                        .get_attribute()
                        .get_foreground()
                        + 14)
                        % 16;
                    editor
                        .buffer_view
                        .lock()
                        .get_caret()
                        .get_attribute()
                        .set_foreground(fg);
                } else {
                    editor.set_caret(pos.x, pos.y + 1);
                }
            }
            MKey::Up => {
                if let MModifiers::Control = modifier {
                    let fg = (editor
                        .buffer_view
                        .lock()
                        .get_caret()
                        .get_attribute()
                        .get_foreground()
                        + 1)
                        % 16;
                    editor
                        .buffer_view
                        .lock()
                        .get_caret()
                        .get_attribute()
                        .set_foreground(fg);
                } else {
                    editor.set_caret(pos.x, pos.y - 1);
                }
            }
            MKey::Left => {
                // TODO: ICE Colors
                if let MModifiers::Control = modifier {
                    let bg = (editor
                        .buffer_view
                        .lock()
                        .get_caret()
                        .get_attribute()
                        .get_background()
                        + 7)
                        % 8;
                    editor
                        .buffer_view
                        .lock()
                        .get_caret()
                        .get_attribute()
                        .set_background(bg);
                } else {
                    editor.set_caret(pos.x - 1, pos.y);
                }
            }
            MKey::Right => {
                // TODO: ICE Colors
                if let MModifiers::Control = modifier {
                    let bg = (editor
                        .buffer_view
                        .lock()
                        .get_caret()
                        .get_attribute()
                        .get_background()
                        + 1)
                        % 8;
                    editor
                        .buffer_view
                        .lock()
                        .get_caret()
                        .get_attribute()
                        .set_background(bg);
                } else {
                    editor.set_caret(pos.x + 1, pos.y);
                }
            }
            MKey::PageDown => {
                // TODO
                println!("pgdn");
            }
            MKey::PageUp => {
                // TODO
                println!("pgup");
            }

            MKey::Escape => {
                editor.buffer_view.lock().clear_selection();
            }
            /*
            MKey::Tab => {
                let tab_size = unsafe { crate::WORKSPACE.settings.tab_size } ;
                if let MModifiers::Control = modifier {
                    let tabs = max(0, (pos.x / tab_size) - 1);
                    let next_tab = tabs * tab_size;
                    editor.set_caret(next_tab, pos.y);
                } else {
                    let tabs = 1 + pos.x / tab_size;
                    let next_tab = min(editor.get_buffer().width as i32 - 1, tabs * tab_size);
                    editor.set_caret(next_tab, pos.y);
                }
            }
            MKey::Home  => {
                if let MModifiers::Control = modifier {
                    for i in 0..editor.get_buffer().width {
                        if !editor.get_char_from_cur_layer(pos.with_x(i as i32)).unwrap_or_default().is_transparent() {
                            editor.set_caret(i as i32, pos.y);
                            return Event::None;
                        }
                    }
                }
                editor.set_caret(0, pos.y);
            }
            MKey::End => {
                if let MModifiers::Control = modifier {
                    for i in (0..editor.get_buffer().width).rev()  {
                        if !editor.get_char_from_cur_layer(pos.with_x(i as i32)).unwrap_or_default().is_transparent() {
                            editor.set_caret(i as i32, pos.y);
                            return Event::None;
                        }
                    }
                }
                let w = editor.get_buffer().width as i32;
                editor.set_caret(w - 1, pos.y);
            }*/
            MKey::Return => {
                editor.set_caret(0, pos.y + 1);
            }
            MKey::Insert => {
                editor.buffer_view.lock().get_caret_mut().insert_mode =
                    !editor.buffer_view.lock().get_caret().insert_mode;
            }
            MKey::Backspace => {
                editor.backspace();
            }

            MKey::Home => {
                let mut pos = editor.get_caret_position();
                pos.x = 0;
                editor.set_caret_position(pos);
            }
            MKey::End => {
                let mut pos = editor.get_caret_position();
                pos.x = i32::MAX;
                editor.set_caret_position(pos);
            }

            MKey::Character(ch) => {
                editor.buffer_view.lock().clear_selection();
                /*        if let MModifiers::Alt = modifier {
                    match key_code {
                        MKeyCode::KeyI => editor.insert_line(pos.y),
                        MKeyCode::KeyU => editor.pickup_color(pos),
                        MKeyCode::KeyY => editor.delete_line(pos.y),
                        MKeyCode::Unknown => {}
                    }
                    return Event::None;
                }*/
                editor.type_key(unsafe { char::from_u32_unchecked(ch as u32) });
            }

            MKey::F1 => {
                handle_outline_insertion(editor, modifier, 0);
            }
            MKey::F2 => {
                handle_outline_insertion(editor, modifier, 1);
            }
            MKey::F3 => {
                handle_outline_insertion(editor, modifier, 2);
            }
            MKey::F4 => {
                handle_outline_insertion(editor, modifier, 3);
            }
            MKey::F5 => {
                handle_outline_insertion(editor, modifier, 4);
            }
            MKey::F6 => {
                handle_outline_insertion(editor, modifier, 5);
            }
            MKey::F7 => {
                handle_outline_insertion(editor, modifier, 6);
            }
            MKey::F8 => {
                handle_outline_insertion(editor, modifier, 7);
            }
            MKey::F9 => {
                handle_outline_insertion(editor, modifier, 8);
            }
            MKey::F10 => {
                handle_outline_insertion(editor, modifier, 9);
            }
            _ => {}
        }
        Event::None
    }
}

impl ClickTool {
    fn move_left(&mut self, editor: &AnsiEditor, rect: &mut Rectangle) {
        let delta = editor.drag_pos.start_abs.x - editor.drag_pos.cur_abs.x;
        rect.start.x = self.start_selection.left() - delta;
        rect.size.width = self.start_selection.get_width() + delta;

        if rect.size.width < 0 {
            rect.size.width = rect.start.x - self.start_selection.right();
            rect.start.x = self.start_selection.right();
        }
    }

    fn move_right(&mut self, editor: &AnsiEditor, rect: &mut Rectangle) {
        rect.size.width = self.start_selection.get_width() - editor.drag_pos.start_abs.x
            + editor.drag_pos.cur_abs.x;
        if rect.size.width < 0 {
            rect.start.x = self.start_selection.left() + rect.size.width;
            rect.size.width = self.start_selection.left() - rect.start.x;
        }
    }

    fn move_top(&mut self, editor: &AnsiEditor, rect: &mut Rectangle) {
        let delta = editor.drag_pos.start_abs.y - editor.drag_pos.cur_abs.y;
        rect.start.y = self.start_selection.top() - delta;
        rect.size.height = self.start_selection.get_height() + delta;

        if rect.size.height < 0 {
            rect.size.height = rect.start.y - self.start_selection.bottom();
            rect.start.y = self.start_selection.bottom();
        }
    }

    fn move_bottom(&mut self, editor: &AnsiEditor, rect: &mut Rectangle) {
        rect.size.height = self.start_selection.get_height() - editor.drag_pos.start_abs.y
            + editor.drag_pos.cur_abs.y;
        if rect.size.height < 0 {
            rect.start.y = self.start_selection.top() + rect.size.height;
            rect.size.height = self.start_selection.top() - rect.start.y;
        }
    }
}

fn is_inside_selection(editor: &AnsiEditor, cur_abs: Position) -> bool {
    if let Some(selection) = editor.buffer_view.lock().get_selection() {
        return selection.is_inside(cur_abs);
    }
    false
}

fn get_selection_drag(editor: &AnsiEditor, cur_abs: Position) -> SelectionDrag {
    if let Some(selection) = editor.buffer_view.lock().get_selection() {
        let rect = selection.as_rectangle();

        if rect.is_inside(cur_abs) {
            let left = cur_abs.x - rect.left() < 2;
            let top = cur_abs.y - rect.top() < 2;
            let right = rect.right() - cur_abs.x < 2;
            let bottom = rect.bottom() - cur_abs.y < 2;

            if left && top {
                return SelectionDrag::TopLeft;
            }

            if right && top {
                return SelectionDrag::TopRight;
            }
            if left && bottom {
                return SelectionDrag::BottomLeft;
            }

            if right && bottom {
                return SelectionDrag::BottomRight;
            }

            if left {
                return SelectionDrag::Left;
            }
            if right {
                return SelectionDrag::Right;
            }

            if top {
                return SelectionDrag::Top;
            }
            if bottom {
                return SelectionDrag::Bottom;
            }

            return SelectionDrag::Move;
        }
    }
    SelectionDrag::None
}
