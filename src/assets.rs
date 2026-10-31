use fontdue::{Font, FontSettings};

use crate::i18n::{Key, Language};

pub const UI_FONT: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");
pub const UI_FONT_LICENSE: &str = include_str!("../assets/fonts/OFL-Inter.txt");
pub const UI_FONT_NAME: &str = "Inter Regular";
pub const UI_FONT_LICENSE_NAME: &str = "OFL-1.1";

pub fn ui_font() -> Option<Font> {
    Font::from_bytes(UI_FONT, FontSettings::default()).ok()
}

pub fn catalog_chars() -> Vec<char> {
    let mut chars: Vec<char> = Key::ALL
        .iter()
        .flat_map(|key| {
            [Language::Pt, Language::En]
                .into_iter()
                .flat_map(move |language| key.text(language).chars())
        })
        .collect();
    chars.sort_unstable();
    chars.dedup();
    chars
}

pub fn missing_glyphs(font: &Font) -> Vec<char> {
    let mut missing: Vec<char> = catalog_chars()
        .into_iter()
        .filter(|ch| !ch.is_control() && font.lookup_glyph_index(*ch) == 0)
        .collect();
    missing.sort_unstable();
    missing.dedup();
    missing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_font_decodes() {
        let font = ui_font().expect("embedded font must decode");
        assert!(font.horizontal_line_metrics(12.0).is_some());
        assert!(font.units_per_em() > 0.0);
    }

    #[test]
    fn embedded_license_is_present() {
        assert!(UI_FONT_LICENSE.contains("SIL OPEN FONT LICENSE"));
        assert!(UI_FONT_LICENSE.contains("Version 1.1"));
        assert!(!UI_FONT_NAME.is_empty());
        assert_eq!(UI_FONT_LICENSE_NAME, "OFL-1.1");
    }

    #[test]
    fn catalog_glyphs_are_covered() {
        let font = ui_font().unwrap();
        let missing = missing_glyphs(&font);
        assert!(missing.is_empty(), "glifos ausentes: {missing:?}");
    }

    #[test]
    fn documented_special_characters_are_covered() {
        let font = ui_font().unwrap();
        for ch in "áéíóúãõçâêô–#+-/%()[]".chars() {
            assert!(font.lookup_glyph_index(ch) != 0, "glifo ausente: {ch:?}");
        }
    }
}
