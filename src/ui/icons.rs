use egui_extras::RetainedImage;

lazy_static::lazy_static! {
    pub static ref SWAP_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("swap.svg", include_bytes!("../../data/icons/swap.svg")).unwrap();
}
lazy_static::lazy_static! {
    pub static ref ADD_LAYER_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes(
        "add_layer.svg",
        include_bytes!("../../data/icons/add_layer.svg"),
    )
    .unwrap();
}

lazy_static::lazy_static! {
    pub static ref MOVE_DOWN_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes(
        "move_down.svg",
        include_bytes!("../../data/icons/move_down.svg"),
    )
    .unwrap();
}

lazy_static::lazy_static! {
    pub static ref MOVE_UP_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes(
        "move_up.svg",
        include_bytes!("../../data/icons/move_up.svg"),
    )
    .unwrap();
}

lazy_static::lazy_static! {
    pub static ref DELETE_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes(
        "delete.svg",
        include_bytes!("../../data/icons/delete.svg"),
    )
    .unwrap();

    pub static ref VISIBLE_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("visible.svg", include_bytes!("../../data/icons/visible.svg")).unwrap();
    pub static ref INVISIBLE_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("invisible.svg", include_bytes!("../../data/icons/invisible.svg")).unwrap();
    pub static ref ANCHOR_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("anchor.svg", include_bytes!("../../data/icons/anchor.svg")).unwrap();


    pub static ref PLAY_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("play.svg", include_bytes!("../../data/icons/play.svg")).unwrap();
    pub static ref REPLAY_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("replay.svg", include_bytes!("../../data/icons/replay.svg")).unwrap();
    pub static ref PAUSE_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("pause.svg", include_bytes!("../../data/icons/pause.svg")).unwrap();
    pub static ref PLAY_PAUSE_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("play_pause.svg", include_bytes!("../../data/icons/play_pause.svg")).unwrap();
    pub static ref SKIP_NEXT_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("skip_next.svg", include_bytes!("../../data/icons/skip_next.svg")).unwrap();
    pub static ref REPEAT_SVG: RetainedImage = egui_extras::RetainedImage::from_svg_bytes("repeat.svg", include_bytes!("../../data/icons/repeat.svg")).unwrap();
}
