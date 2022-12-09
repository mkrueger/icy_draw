use std::sync::{Arc, Mutex};
use eframe::epaint::{Vec2, Rect, Pos2, Rounding, Color32, Stroke};
use eframe::egui::{self, Sense};
use super::ansi_editor::BufferView;

pub fn palette_editor_16(buffer_opt: Option<Arc<Mutex<BufferView>>>) -> impl egui::Widget {
    move |ui: &mut egui::Ui| {
        let height  = ui.available_width() / 8.0;

        let (id, stroke_rect) = ui.allocate_space(Vec2::new(ui.available_width(), height * 2.0));
        let mut response = ui.interact(stroke_rect, id, Sense::click());
        
        let painter = ui.painter_at(stroke_rect);

        if let Some(buffer_view) = buffer_opt {
            let caret_attr = &buffer_view.lock().unwrap().caret.get_attribute();
            let palette = buffer_view.lock().unwrap().buf.palette.clone();

            for i in 0..16 {
                let (r, g, b) = palette.colors[i].get_rgb();
                painter.rect_filled( Rect::from_min_size(
                    Pos2::new(stroke_rect.left() + (i % 8) as f32  * height,  stroke_rect.top() + (i / 8) as f32  * height),
                    Vec2::new(height, height)
                ), Rounding::none(), Color32::from_rgb(r, g, b));
            }

            let marker_len = height / 3.;
            // paint fg marker
            let stroke = Stroke::new(1., Color32::WHITE);
            let origin = Pos2::new(
                stroke_rect.left() + (caret_attr.get_foreground() % 8) as f32  * height,  
                stroke_rect.top() + (caret_attr.get_foreground() / 8) as f32  * height);
            painter.line_segment([origin, origin + Vec2::new(marker_len, 0.)], stroke);
            painter.line_segment([origin, origin + Vec2::new(0., marker_len)], stroke);
            for i in 0..marker_len as usize  {
                painter.line_segment([origin + Vec2::new(i as f32, 0.), origin + Vec2::new(0., i as f32)], stroke);
            }
            let stroke = Stroke::new(1., Color32::GRAY);
            painter.line_segment([origin, origin + Vec2::new(marker_len, 0.)], stroke);
            painter.line_segment([origin, origin + Vec2::new(0., marker_len)], stroke);
            painter.line_segment([origin + Vec2::new(marker_len, 0.), origin + Vec2::new(0., marker_len)], stroke);

            // paint bg marker
            println!("{}", caret_attr.get_background());
            let stroke = Stroke::new(1., Color32::WHITE);
            let origin = Pos2::new(
                stroke_rect.left() + (1 + caret_attr.get_background() % 8) as f32 * height,  
                stroke_rect.top() + (1 + caret_attr.get_background() / 8) as f32 * height);
            painter.line_segment([origin, origin - Vec2::new(marker_len, 0.)], stroke);
            painter.line_segment([origin, origin - Vec2::new(0., marker_len)], stroke);
            for i in 0..marker_len as usize {
                painter.line_segment([origin - Vec2::new(i as f32, 0.), origin - Vec2::new(0., i as f32)], stroke);
            }
            let stroke = Stroke::new(1., Color32::GRAY);
            painter.line_segment([origin, origin - Vec2::new(marker_len, 0.)], stroke);
            painter.line_segment([origin, origin - Vec2::new(0., marker_len)], stroke);
            painter.line_segment([origin - Vec2::new(marker_len, 0.), origin - Vec2::new(0., marker_len)], stroke);
            if let Some(hp) = response.hover_pos() {
                let pos = hp.to_vec2() / Vec2::new(height, height);
                let color = pos.x as u32 + (pos.y - 1.) as u32 * 8;
                if response.clicked() {
                    buffer_view.lock().unwrap().caret.set_foreground(color);
                    response.mark_changed();
                }
                if response.secondary_clicked() {
                    buffer_view.lock().unwrap().caret.set_background(color);
                    response.mark_changed();
                }
            }
        }

        response
    }
}