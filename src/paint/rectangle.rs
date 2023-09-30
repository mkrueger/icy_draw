use icy_engine::Position;
use icy_engine_egui::BufferView;

use super::{plot_point, BrushMode, ColorMode, PointRole};

pub fn draw_rectangle(buffer_view: &mut BufferView, from: impl Into<Position>, to: impl Into<Position>, mode: BrushMode, color_mode: ColorMode) {
    let from = from.into();
    let to = to.into();
    plot_point(buffer_view, from, mode.clone(), color_mode, PointRole::NWCorner);
    plot_point(buffer_view, (to.x, from.y), mode.clone(), color_mode, PointRole::NECorner);

    plot_point(buffer_view, (from.x, to.y), mode.clone(), color_mode, PointRole::SWCorner);
    plot_point(buffer_view, (to.x, to.y), mode.clone(), color_mode, PointRole::SECorner);

    for x in from.x + 1..to.x {
        plot_point(buffer_view, (x, from.y), mode.clone(), color_mode, PointRole::TopSide);
        plot_point(buffer_view, (x, to.y), mode.clone(), color_mode, PointRole::BottomSide);
    }

    for y in from.y + 1..to.y {
        plot_point(buffer_view, (from.x, y), mode.clone(), color_mode, PointRole::LeftSide);
        plot_point(buffer_view, (to.x, y), mode.clone(), color_mode, PointRole::RightSide);
    }
}

pub fn fill_rectangle(buffer_view: &mut BufferView, from: impl Into<Position>, to: impl Into<Position>, mode: BrushMode, color_mode: ColorMode) {
    let from = from.into();
    let to = to.into();

    for y in from.y + 1..to.y {
        for x in from.x + 1..to.x {
            plot_point(buffer_view, (x, y), mode.clone(), color_mode, PointRole::Fill);
        }
    }
    draw_rectangle(buffer_view, from, to, mode, color_mode);
}
