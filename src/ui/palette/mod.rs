use regex::Regex;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct IceColor {
    pub name: Option<String>,
    pub color: (u8, u8, u8),
}

impl IceColor {
    pub fn get_rgb(&self) -> (u8, u8, u8) {
        self.color
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
            color: (r, g, b),
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
        self.color = (r, g, b);
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct IcePalette {
    pub title: String,
    pub description: String,
    pub colors: Vec<IceColor>,
}

pub enum PaletteFormat {
    Hex,
    Pal,
    Gpl,
    Txt,
    Ase,
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

    pub fn load_palette(format: PaletteFormat, bytes: &[u8]) -> anyhow::Result<Self> {
        let mut colors = Vec::new();
        let mut title = String::new();
        let mut description = String::new();
        match format {
            PaletteFormat::Hex => match String::from_utf8(bytes.to_vec()) {
                Ok(data) => {
                    let re = Regex::new(r"([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})")?;
                    for (_, [r, g, b]) in re.captures_iter(&data).map(|c| c.extract()) {
                        let r = u32::from_str_radix(r, 16)?;
                        let g = u32::from_str_radix(g, 16)?;
                        let b = u32::from_str_radix(b, 16)?;
                        colors.push(IceColor::from_rgb(r as u8, g as u8, b as u8));
                    }
                }
                Err(err) => return Err(anyhow::anyhow!("Invalid input: {err}")),
            },
            PaletteFormat::Pal => {
                match String::from_utf8(bytes.to_vec()) {
                    Ok(data) => {
                        let re = Regex::new(r"(\d+)\s+(\d+)\s+(\d+)")?;

                        for (i, line) in data.lines().enumerate() {
                            match i {
                                0 => {
                                    if line != "JASC-PAL" {
                                        return Err(anyhow::anyhow!(
                                            "Only JASC-PAL supported: {line}"
                                        ));
                                    }
                                }
                                1 | 2 => {
                                    // Ignore
                                }
                                _ => {
                                    for (_, [r, g, b]) in
                                        re.captures_iter(line).map(|c| c.extract())
                                    {
                                        let r = r.parse::<u32>()?;
                                        let g = g.parse::<u32>()?;
                                        let b = b.parse::<u32>()?;
                                        colors.push(IceColor::from_rgb(r as u8, g as u8, b as u8));
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => return Err(anyhow::anyhow!("Invalid input: {err}")),
                }
            }
            PaletteFormat::Gpl => match String::from_utf8(bytes.to_vec()) {
                Ok(data) => {
                    let color_regex = Regex::new(r"(\d+)\s+(\d+)\s+(\d+)\s+\S+")?;
                    let name_regex = Regex::new(r"\s*#Palette Name:\s*(.*)\s*")?;
                    let description_regex = Regex::new(r"\s*#Description:\s*(.*)\s*")?;
                    for (i, line) in data.lines().enumerate() {
                        match i {
                            0 => {
                                if line != "GIMP Palette" {
                                    return Err(anyhow::anyhow!(
                                        "Only GIMP Palette supported: {line}"
                                    ));
                                }
                            }
                            _ => {
                                if line.starts_with('#') {
                                    if let Some(cap) = name_regex.captures(line) {
                                        if let Some(name) = cap.get(1) {
                                            title = name.as_str().to_string();
                                        }
                                    }
                                    if let Some(cap) = description_regex.captures(line) {
                                        if let Some(name) = cap.get(1) {
                                            description = name.as_str().to_string();
                                        }
                                    }
                                } else if let Some(cap) = color_regex.captures(line) {
                                    let (_, [r, g, b]) = cap.extract();

                                    let r = r.parse::<u32>()?;
                                    let g = g.parse::<u32>()?;
                                    let b = b.parse::<u32>()?;
                                    colors.push(IceColor::from_rgb(r as u8, g as u8, b as u8));
                                }
                            }
                        }
                    }
                }
                Err(err) => return Err(anyhow::anyhow!("Invalid input: {err}")),
            },
            PaletteFormat::Txt => match String::from_utf8(bytes.to_vec()) {
                Ok(data) => {
                    let color_regex = Regex::new(
                        r"([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})",
                    )?;
                    let name_regex = Regex::new(r"\s*;Palette Name:\s*(.*)\s*")?;
                    let description_regex = Regex::new(r"\s*;Description:\s*(.*)\s*")?;
                    for line in data.lines() {
                        if line.starts_with(';') {
                            if let Some(cap) = name_regex.captures(line) {
                                if let Some(name) = cap.get(1) {
                                    title = name.as_str().to_string();
                                }
                            }
                            if let Some(cap) = description_regex.captures(line) {
                                if let Some(name) = cap.get(1) {
                                    description = name.as_str().to_string();
                                }
                            }
                        } else if let Some(cap) = color_regex.captures(line) {
                            let (_, [_a, r, g, b]) = cap.extract();

                            let r = u32::from_str_radix(r, 16).unwrap();
                            let g = u32::from_str_radix(g, 16).unwrap();
                            let b = u32::from_str_radix(b, 16).unwrap();
                            colors.push(IceColor::from_rgb(r as u8, g as u8, b as u8));
                        }
                    }
                }
                Err(err) => return Err(anyhow::anyhow!("Invalid input: {err}")),
            },
            PaletteFormat::Ase => todo!(),
        }
        Ok(Self {
            title,
            description,
            colors,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{IcePalette, PaletteFormat};

    #[test]
    fn test_hex_format() {
        let palette =
            IcePalette::load_palette(PaletteFormat::Hex, include_bytes!("ansi32.hex")).unwrap();
        assert_eq!(32, palette.len());
    }

    #[test]
    fn test_txt_format() {
        let palette_hex =
            IcePalette::load_palette(PaletteFormat::Hex, include_bytes!("ansi32.hex")).unwrap();
        let palette =
            IcePalette::load_palette(PaletteFormat::Txt, include_bytes!("ansi32.txt")).unwrap();
        assert_eq!("ANSI32", palette.title);
        assert_eq!("All 16 original EGA ANSI art colors and 16 more in-between. I averaged these to find 8 darker and 8 lighter colors to complement the originals.", palette.description);
        assert_eq!(palette_hex.colors, palette.colors);
    }

    #[test]
    fn test_pal_format() {
        let palette_hex =
            IcePalette::load_palette(PaletteFormat::Hex, include_bytes!("ansi32.hex")).unwrap();
        let palette =
            IcePalette::load_palette(PaletteFormat::Pal, include_bytes!("ansi32.pal")).unwrap();
        assert_eq!(palette_hex, palette);
    }

    #[test]
    fn test_gpl_format() {
        let palette_hex =
            IcePalette::load_palette(PaletteFormat::Hex, include_bytes!("ansi32.hex")).unwrap();
        let palette =
            IcePalette::load_palette(PaletteFormat::Gpl, include_bytes!("ansi32.gpl")).unwrap();
        assert_eq!("ANSI32", palette.title);
        assert_eq!("All 16 original EGA ANSI art colors and 16 more in-between. I averaged these to find 8 darker and 8 lighter colors to complement the originals.", palette.description);
        assert_eq!(palette_hex.colors, palette.colors);
    }
}
