use crate::MainWindow;
use eframe::{
    egui::{self, Sense},
    epaint::{pos2, Rect, Rounding, Vec2},
};

pub fn add_tool_switcher(ctx: &egui::Context, ui: &mut egui::Ui, arg: &mut MainWindow) {
    let (id, back_rect) = ui.allocate_space(Vec2::new(200., 68.0));

    let spacing = 4.0;
    let icon_size = 28.0;

    if let Ok(tools) = arg.document_behavior.tools.lock() {
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        let mut pos = back_rect.min + Vec2::new(spacing, spacing);
        if tools[arg.document_behavior.selected_tool].is_exclusive() {
            return;
        }
        for i in 0..tools.len() {
            let t = &tools[i];
            if !t.is_visible() {
                continue;
            }
            let image = t.get_icon_name();

            let rect = Rect::from_min_size(pos.floor(), Vec2::new(icon_size, icon_size));
            let response = ui.interact(rect, id.with(i), Sense::click());
            if i == arg.document_behavior.selected_tool {
                ui.painter().rect_filled(
                    rect.expand(2.0),
                    Rounding::same(4.0),
                    ui.style().visuals.extreme_bg_color,
                );
                ui.painter().rect_stroke(
                    rect.expand(2.0),
                    Rounding::same(4.0),
                    ui.style().visuals.window_stroke,
                );
            }

            if response.hovered() {
                ui.painter().rect_filled(
                    rect.expand(2.0),
                    Rounding::same(4.0),
                    ui.style().visuals.widgets.active.bg_fill,
                );
                ui.painter().rect_stroke(
                    rect.expand(2.0),
                    Rounding::same(4.0),
                    ui.style().visuals.window_stroke,
                );
            }

            let painter = ui.painter_at(rect);
            let tint = if i == arg.document_behavior.selected_tool {
                ui.visuals().widgets.active.fg_stroke.color
            } else {
                ui.visuals().widgets.inactive.fg_stroke.color
            };
            painter.image(image.texture_id(ctx), rect, uv, tint);

            pos.x += icon_size + spacing;
            if pos.x - back_rect.min.x - spacing > back_rect.width() {
                pos.x = back_rect.min.x + spacing;
                pos.y += icon_size + spacing;
            }

            if response.clicked() {
                arg.document_behavior.selected_tool = i;
            }
        }
    }
}
