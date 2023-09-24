use icy_engine::Position;
use icy_engine_egui::BufferView;

use super::{plot_point, BrushMode, ColorMode, PointRole};

fn get_line_points(from: Position, to: Position) -> Vec<Position> {
    let dx = (to.x - from.x).abs();
    let sx = if from.x < to.x { 1 } else { -1 };
    let dy = (to.y - from.y).abs();
    let sy = if from.y < to.y { 1 } else { -1 };

    let mut err = if dx > dy { dx } else { -dy } / 2;

    let mut result = Vec::new();
    let mut cur = from;
    loop {
        result.push(cur);
        if cur == to {
            break;
        }

        let e2 = err;
        if e2 > -dx {
            err -= dy;
            cur.x += sx;
        }
        if e2 < dy {
            err += dx;
            cur.y += sy;
        }
    }
    result
}

pub fn draw_line(
    buffer_view: &mut BufferView,
    from: impl Into<Position>,
    to: impl Into<Position>,
    mode: BrushMode,
    color_mode: ColorMode,
) {
    let from = from.into();
    let to = to.into();
    for point in get_line_points(from, to) {
        plot_point(
            buffer_view,
            point,
            mode.clone(),
            color_mode,
            PointRole::Line,
        );
    }
}
