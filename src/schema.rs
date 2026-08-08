use serde::{Deserialize, Serialize};

pub const VERSION: &str = "1.0";
pub const COLOR_SPACE: &str = "sRGB";
pub const CHANNELS: &str = "RGBA";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pixel {
    pub x: u32,
    pub y: u32,
    pub hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImageJson {
    pub img2jx_version: String,
    pub width: u32,
    pub height: u32,
    pub color_space: String,
    pub channels: String,
    pub rows: Vec<Vec<Pixel>>,
}

impl ImageJson {
    pub fn new(width: u32, height: u32, rows: Vec<Vec<Pixel>>) -> Self {
        Self {
            img2jx_version: VERSION.to_string(),
            width,
            height,
            color_space: COLOR_SPACE.to_string(),
            channels: CHANNELS.to_string(),
            rows,
        }
    }
}

/// Format RGBA bytes as `#RRGGBBAA`.
pub fn rgba_to_hex(r: u8, g: u8, b: u8, a: u8) -> String {
    format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
}

/// Parse `#RRGGBBAA` into RGBA bytes.
pub fn hex_to_rgba(hex: &str) -> Option<[u8; 4]> {
    let s = hex.strip_prefix('#')?;
    if s.len() != 8 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    let a = u8::from_str_radix(&s[6..8], 16).ok()?;
    Some([r, g, b, a])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        let hex = rgba_to_hex(255, 128, 0, 255);
        assert_eq!(hex, "#FF8000FF");
        assert_eq!(hex_to_rgba(&hex), Some([255, 128, 0, 255]));
    }
}
