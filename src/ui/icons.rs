use egui_extras::RetainedImage;

lazy_static::lazy_static! {
    pub static ref SWAP_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("swap.svg", include_bytes!("../../data/icons/swap.svg")).unwrap();
}

