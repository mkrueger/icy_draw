use egui_extras::RetainedImage;

lazy_static::lazy_static! {
    pub static ref ADD_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("add.svg", include_bytes!("../../../data/icons/add.svg")).unwrap();
    pub static ref BRUSH_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("brush.svg", include_bytes!("../../../data/icons/brush.svg")).unwrap();
    pub static ref CURSOR_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("cursor.svg", include_bytes!("../../../data/icons/cursor.svg")).unwrap();
    pub static ref DELETE_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("delete.svg", include_bytes!("../../../data/icons/delete.svg")).unwrap();
    pub static ref DOWN_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("down.svg", include_bytes!("../../../data/icons/down.svg")).unwrap();
    pub static ref DROPPER_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("dropper.svg", include_bytes!("../../../data/icons/dropper.svg")).unwrap();
    pub static ref ELLIPSE_FILLED_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("ellipse_filled.svg", include_bytes!("../../../data/icons/ellipse_filled.svg")).unwrap();
    pub static ref ELLIPSE_OUTLINE_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("ellipse_outline.svg", include_bytes!("../../../data/icons/ellipse_outline.svg")).unwrap();
    pub static ref ERASER_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("eraser.svg", include_bytes!("../../../data/icons/eraser.svg")).unwrap();
    pub static ref FILL_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("fill.svg", include_bytes!("../../../data/icons/fill.svg")).unwrap();
    pub static ref LINE_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("line.svg", include_bytes!("../../../data/icons/line.svg")).unwrap();
    pub static ref FONT_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("font.svg", include_bytes!("../../../data/icons/font.svg")).unwrap();
    pub static ref MOVE_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("move.svg", include_bytes!("../../../data/icons/move.svg")).unwrap();
    pub static ref RECTANGLE_FILLED_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("rectangle_filled.svg", include_bytes!("../../../data/icons/rectangle_filled.svg")).unwrap();
    pub static ref RECTANGLE_OUTLINE_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("rectangle_outline.svg", include_bytes!("../../../data/icons/rectangle_outline.svg")).unwrap();
    pub static ref SWAP_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("swap.svg", include_bytes!("../../../data/icons/swap.svg")).unwrap();
    pub static ref UP_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("up.svg", include_bytes!("../../../data/icons/up.svg")).unwrap();
}

