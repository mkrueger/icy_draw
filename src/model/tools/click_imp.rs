use eframe::egui;
use egui_extras::RetainedImage;
use icy_engine::Rectangle;
use icy_engine_egui::TerminalCalc;

use crate::{AnsiEditor, Message};

use super::{Event, Position, Tool};

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
    ) -> Event {
        if button == 1 && !is_inside_selection(editor, cur_abs) {
            editor.set_caret_position(pos);
            editor.buffer_view.lock().clear_selection();
        }
        Event::None
    }

    fn handle_drag_begin(&mut self, editor: &mut AnsiEditor) -> Event {
        self.selection_drag = get_selection_drag(editor, editor.drag_pos.start_abs);
        if !matches!(self.selection_drag, SelectionDrag::None) {
            if let Some(selection) = editor.buffer_view.lock().get_selection() {
                self.start_selection = selection.as_rectangle();
            }
        }
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
            return Event::None;
        }

        let mut cur = editor.drag_pos.cur;
        if editor.drag_pos.start < cur {
            cur += Position::new(1, 1);
        }

        if editor.drag_pos.start == cur {
            editor.buffer_view.lock().clear_selection();
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
