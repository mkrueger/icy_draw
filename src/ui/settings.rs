use icy_engine::Color;

pub struct Settings {
    font_outline_style: usize,
    character_set: usize,

    custom_palette: IcePalette,
}

impl Settings {
    pub fn set_character_set(character_set: usize) {
        unsafe {
            SETTINGS.character_set = character_set;
        }
    }

    pub fn get_character_set() -> usize {
        unsafe { SETTINGS.character_set }
    }

    pub fn set_font_outline_style(font_outline_style: usize) {
        unsafe {
            SETTINGS.font_outline_style = font_outline_style;
        }
    }

    pub fn get_font_outline_style() -> usize {
        unsafe { SETTINGS.font_outline_style }
    }

    pub fn get_custom_palette() -> &'static mut IcePalette {
        unsafe { &mut SETTINGS.custom_palette }
    }

    pub fn set_custom_palette(pal: IcePalette) {
        unsafe {
            SETTINGS.custom_palette = pal;
        }
    }
}

pub static mut SETTINGS: Settings = Settings {
    font_outline_style: 0,
    custom_palette: IcePalette {
        title: String::new(),
        colors: Vec::new(),
    },
    character_set: 0,
};

#[derive(Default)]
pub struct IceColor {
    pub name: Option<String>,
    pub color: Color,
}

impl IceColor {
    pub fn get_rgb(&self) -> (u8, u8, u8) {
        self.color.get_rgb()
    }

    pub(crate) fn get_name(&self) -> String {
        if let Some(name) = &self.name {
            name.clone()
        } else {
            self.get_rgb_text()
        }
    }

    pub(crate) fn get_rgb_text(&self) -> String {
        let (r, g, b) = self.get_rgb();
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> IceColor {
        IceColor {
            name: None,
            color: Color::new(r, g, b),
        }
    }

    pub fn set_name(&mut self, name: String) {
        if name.is_empty() {
            self.name = None;
        } else {
            self.name = Some(name);
        }
    }

    pub(crate) fn set_rgb(&mut self, r: u8, g: u8, b: u8) {
        self.color = Color::new(r, g, b);
    }
}

#[derive(Default)]
pub struct IcePalette {
    pub title: String,
    pub colors: Vec<IceColor>,
}

impl IcePalette {
    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }

    pub fn push_rgb(&mut self, r: u8, g: u8, b: u8) {
        self.colors.push(IceColor::from_rgb(r, g, b));
    }
}
