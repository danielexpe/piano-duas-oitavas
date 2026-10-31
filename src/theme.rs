use ratatui::style::Color;

use crate::cli::ColorMode as CliColorMode;

pub const SURFACE: Color = Color::Rgb(0x12, 0x10, 0x0e);
pub const PANEL: Color = Color::Rgb(0x1c, 0x19, 0x16);
pub const SURFACE_RAISED: Color = Color::Rgb(0x26, 0x21, 0x1c);
pub const IVORY: Color = Color::Rgb(0xf4, 0xea, 0xd8);
pub const IVORY_PRESSED: Color = Color::Rgb(0xff, 0xd7, 0x9a);
pub const IVORY_EDGE: Color = Color::Rgb(0xc9, 0xb9, 0x9f);
pub const EBONY: Color = Color::Rgb(0x17, 0x14, 0x12);
pub const EBONY_PRESSED: Color = Color::Rgb(0xb8, 0x79, 0x2a);
pub const EBONY_EDGE: Color = Color::Rgb(0x3a, 0x30, 0x28);
pub const ACCENT: Color = Color::Rgb(0xe0, 0xa2, 0x4a);
pub const ACCENT_SOFT: Color = Color::Rgb(0x8a, 0x5a, 0x24);
pub const TEXT_PRIMARY: Color = Color::Rgb(0xf4, 0xea, 0xd8);
pub const TEXT_SECONDARY: Color = Color::Rgb(0xb8, 0xa8, 0x8f);
pub const TEXT_ON_LIGHT: Color = Color::Rgb(0x1c, 0x19, 0x16);
pub const DANGER: Color = Color::Rgb(0xd4, 0x5a, 0x4a);
pub const SUCCESS: Color = Color::Rgb(0x7f, 0xb0, 0x69);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorMode {
    #[default]
    Auto,
    Truecolor,
    Ansi256,
    Ansi16,
    Mono,
}

impl ColorMode {
    pub fn from_cli(mode: CliColorMode) -> Self {
        match mode {
            CliColorMode::Auto => Self::Auto,
            CliColorMode::Truecolor => Self::Truecolor,
            CliColorMode::Color256 => Self::Ansi256,
            CliColorMode::Color16 => Self::Ansi16,
            CliColorMode::Never => Self::Mono,
        }
    }

    pub fn resolve(
        cli: CliColorMode,
        no_color: bool,
        colorterm: Option<&str>,
        term: Option<&str>,
    ) -> Self {
        if no_color {
            return Self::Mono;
        }
        let mode = Self::from_cli(cli);
        if mode != Self::Auto {
            return mode;
        }
        if term == Some("dumb") {
            return Self::Mono;
        }
        match colorterm {
            Some(value)
                if value.eq_ignore_ascii_case("truecolor")
                    || value.eq_ignore_ascii_case("24bit") =>
            {
                Self::Truecolor
            }
            _ => Self::Ansi256,
        }
    }

    pub fn resolve_from_env(cli: CliColorMode) -> Self {
        let no_color = std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty());
        let colorterm = std::env::var("COLORTERM").ok();
        let term = std::env::var("TERM").ok();
        Self::resolve(cli, no_color, colorterm.as_deref(), term.as_deref())
    }

    pub const fn is_mono(self) -> bool {
        matches!(self, Self::Mono)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub mode: ColorMode,
    pub surface: Color,
    pub panel: Color,
    pub surface_raised: Color,
    pub ivory: Color,
    pub ivory_pressed: Color,
    pub ivory_edge: Color,
    pub ebony: Color,
    pub ebony_pressed: Color,
    pub ebony_edge: Color,
    pub accent: Color,
    pub accent_soft: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_on_light: Color,
    pub danger: Color,
    pub success: Color,
}

impl Palette {
    pub const fn new(mode: ColorMode) -> Self {
        match mode {
            ColorMode::Auto | ColorMode::Truecolor => Self::truecolor(mode),
            ColorMode::Ansi256 => Self::ansi256(),
            ColorMode::Ansi16 => Self::ansi16(),
            ColorMode::Mono => Self::mono(),
        }
    }

    const fn truecolor(mode: ColorMode) -> Self {
        Self {
            mode,
            surface: SURFACE,
            panel: PANEL,
            surface_raised: SURFACE_RAISED,
            ivory: IVORY,
            ivory_pressed: IVORY_PRESSED,
            ivory_edge: IVORY_EDGE,
            ebony: EBONY,
            ebony_pressed: EBONY_PRESSED,
            ebony_edge: EBONY_EDGE,
            accent: ACCENT,
            accent_soft: ACCENT_SOFT,
            text_primary: TEXT_PRIMARY,
            text_secondary: TEXT_SECONDARY,
            text_on_light: TEXT_ON_LIGHT,
            danger: DANGER,
            success: SUCCESS,
        }
    }

    const fn ansi256() -> Self {
        Self {
            mode: ColorMode::Ansi256,
            surface: Color::Indexed(233),
            panel: Color::Indexed(234),
            surface_raised: Color::Indexed(236),
            ivory: Color::Indexed(230),
            ivory_pressed: Color::Indexed(222),
            ivory_edge: Color::Indexed(250),
            ebony: Color::Indexed(233),
            ebony_pressed: Color::Indexed(172),
            ebony_edge: Color::Indexed(237),
            accent: Color::Indexed(179),
            accent_soft: Color::Indexed(94),
            text_primary: Color::Indexed(230),
            text_secondary: Color::Indexed(180),
            text_on_light: Color::Indexed(234),
            danger: Color::Indexed(167),
            success: Color::Indexed(107),
        }
    }

    const fn ansi16() -> Self {
        Self {
            mode: ColorMode::Ansi16,
            surface: Color::Black,
            panel: Color::Black,
            surface_raised: Color::DarkGray,
            ivory: Color::White,
            ivory_pressed: Color::Yellow,
            ivory_edge: Color::Gray,
            ebony: Color::Black,
            ebony_pressed: Color::Yellow,
            ebony_edge: Color::DarkGray,
            accent: Color::Yellow,
            accent_soft: Color::DarkGray,
            text_primary: Color::White,
            text_secondary: Color::Gray,
            text_on_light: Color::Black,
            danger: Color::Red,
            success: Color::Green,
        }
    }

    const fn mono() -> Self {
        Self {
            mode: ColorMode::Mono,
            surface: Color::Reset,
            panel: Color::Reset,
            surface_raised: Color::Reset,
            ivory: Color::Reset,
            ivory_pressed: Color::Reset,
            ivory_edge: Color::Reset,
            ebony: Color::Reset,
            ebony_pressed: Color::Reset,
            ebony_edge: Color::Reset,
            accent: Color::Reset,
            accent_soft: Color::Reset,
            text_primary: Color::Reset,
            text_secondary: Color::Reset,
            text_on_light: Color::Reset,
            danger: Color::Reset,
            success: Color::Reset,
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::truecolor(ColorMode::Truecolor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_color_and_never_force_mono() {
        assert_eq!(
            ColorMode::resolve(CliColorMode::Auto, true, Some("truecolor"), Some("xterm")),
            ColorMode::Mono
        );
        assert_eq!(
            ColorMode::resolve(CliColorMode::Never, false, Some("truecolor"), Some("xterm")),
            ColorMode::Mono
        );
    }

    #[test]
    fn explicit_modes_win_over_terminal_detection() {
        assert_eq!(
            ColorMode::resolve(
                CliColorMode::Color16,
                false,
                Some("truecolor"),
                Some("xterm-256color")
            ),
            ColorMode::Ansi16
        );
        assert_eq!(
            ColorMode::resolve(
                CliColorMode::Color256,
                false,
                Some("truecolor"),
                Some("xterm-256color")
            ),
            ColorMode::Ansi256
        );
        assert_eq!(
            ColorMode::resolve(CliColorMode::Truecolor, false, None, Some("xterm")),
            ColorMode::Truecolor
        );
    }

    #[test]
    fn auto_detects_truecolor_and_dumb_terminals() {
        assert_eq!(
            ColorMode::resolve(
                CliColorMode::Auto,
                false,
                Some("truecolor"),
                Some("xterm-256color")
            ),
            ColorMode::Truecolor
        );
        assert_eq!(
            ColorMode::resolve(
                CliColorMode::Auto,
                false,
                Some("24bit"),
                Some("xterm-256color")
            ),
            ColorMode::Truecolor
        );
        assert_eq!(
            ColorMode::resolve(CliColorMode::Auto, false, None, Some("xterm")),
            ColorMode::Ansi256
        );
        assert_eq!(
            ColorMode::resolve(CliColorMode::Auto, false, None, Some("dumb")),
            ColorMode::Mono
        );
    }

    fn rgb(color: Color) -> Option<(f32, f32, f32)> {
        match color {
            Color::Rgb(r, g, b) => Some((
                f32::from(r) / 255.0,
                f32::from(g) / 255.0,
                f32::from(b) / 255.0,
            )),
            _ => None,
        }
    }

    fn channel(value: f32) -> f32 {
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    fn luminance(color: Color) -> f32 {
        let (red, green, blue) = rgb(color).expect("truecolor token");
        0.2126 * channel(red) + 0.7152 * channel(green) + 0.0722 * channel(blue)
    }

    fn contrast(foreground: Color, background: Color) -> f32 {
        let first = luminance(foreground);
        let second = luminance(background);
        let (light, dark) = if first > second {
            (first, second)
        } else {
            (second, first)
        };
        (light + 0.05) / (dark + 0.05)
    }

    #[test]
    fn documented_contrast_minimums_hold() {
        let primary = contrast(TEXT_PRIMARY, SURFACE);
        assert!(primary >= 7.0, "texto principal: {primary:.2}:1");
        let secondary = contrast(TEXT_SECONDARY, SURFACE);
        assert!(secondary >= 4.5, "texto secundário: {secondary:.2}:1");
        let on_light = contrast(TEXT_ON_LIGHT, IVORY);
        assert!(on_light >= 7.0, "texto sobre marfim: {on_light:.2}:1");
        let pressed = contrast(TEXT_ON_LIGHT, IVORY_PRESSED);
        assert!(
            pressed >= 7.0,
            "texto sobre marfim pressionado: {pressed:.2}:1"
        );
    }

    #[test]
    fn palettes_keep_semantic_tokens() {
        let truecolor = Palette::new(ColorMode::Truecolor);
        assert_eq!(truecolor.surface, SURFACE);
        assert_eq!(truecolor.ivory_pressed, IVORY_PRESSED);
        let mono = Palette::new(ColorMode::Mono);
        assert!(mono.mode.is_mono());
        assert_eq!(mono.accent, Color::Reset);
        let ansi = Palette::new(ColorMode::Ansi256);
        assert_eq!(ansi.surface, Color::Indexed(233));
    }
}
