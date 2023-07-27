pub struct Settings {
    pub font_outline_style: usize
}

pub static mut SETTINGS: Settings = Settings {
    font_outline_style: 0
};