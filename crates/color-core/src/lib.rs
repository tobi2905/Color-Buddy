//! Pure color logic for Color-Buddy.
//!
//! This crate has no Tauri dependency so it can be tested on its own.
//! Conversions, naming, palettes, simulation and checks are added from
//! Phase 2 onwards; see `SPEC.md`.

/// An 8-bit sRGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Uppercase `#RRGGBB`.
    pub fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_hex_is_uppercase_and_zero_padded() {
        assert_eq!(Rgb::new(0x87, 0xCE, 0xEB).to_hex(), "#87CEEB");
        assert_eq!(Rgb::new(0, 0, 0).to_hex(), "#000000");
        assert_eq!(Rgb::new(255, 0, 10).to_hex(), "#FF000A");
    }
}
