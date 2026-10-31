use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use crate::i18n::Language;
use crate::state::{DEFAULT_BASE_OCTAVE, DEFAULT_VOLUME};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum Backend {
    #[default]
    Auto,
    Tui,
    Fb,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum ColorMode {
    #[default]
    Auto,
    Truecolor,
    #[value(name = "256")]
    Color256,
    #[value(name = "16")]
    Color16,
    Never,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum LangArg {
    #[default]
    Pt,
    En,
}

impl From<LangArg> for Language {
    fn from(value: LangArg) -> Self {
        match value {
            LangArg::Pt => Self::Pt,
            LangArg::En => Self::En,
        }
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "piano-duas-oitavas",
    version,
    about = "Piano de duas oitavas em TUI/fbdev com mapeamento do teclado alfanumérico, síntese polifônica e efeitos visuais."
)]
pub struct Cli {
    /// Initial UI language.
    #[arg(long, value_enum, default_value = "pt")]
    pub lang: LangArg,

    /// Video backend. `auto` never selects the framebuffer.
    #[arg(long, value_enum, default_value = "auto")]
    pub backend: Backend,

    /// Framebuffer device path.
    #[arg(long, value_name = "PATH", default_value = "/dev/fb0")]
    pub fb_device: PathBuf,

    /// Initial base octave (1..=6).
    #[arg(long, value_name = "1..=6", default_value_t = DEFAULT_BASE_OCTAVE, value_parser = parse_octave)]
    pub octave: u8,

    /// Initial volume (0.0..=1.0).
    #[arg(long, value_name = "0.0..=1.0", default_value_t = DEFAULT_VOLUME, value_parser = parse_volume)]
    pub volume: f32,

    /// Deterministic RNG seed. Reserved; has no musical effect in v1.
    #[arg(long)]
    pub seed: Option<u64>,

    /// Color mode. NO_COLOR always wins.
    #[arg(long, value_enum, default_value = "auto")]
    pub color: ColorMode,

    /// Disable animations.
    #[arg(long)]
    pub no_animations: bool,

    /// Start muted.
    #[arg(long)]
    pub no_audio: bool,

    /// Render a deterministic PNG and exit without using fb, tty or audio.
    #[arg(long, value_name = "PATH")]
    pub capture_frame: Option<PathBuf>,
}

fn parse_octave(value: &str) -> Result<u8, String> {
    let octave: u8 = value
        .parse()
        .map_err(|_| format!("oitava inválida: {value}; use um inteiro de 1 a 6"))?;
    if (crate::core::note::MIN_BASE_OCTAVE..=crate::core::note::MAX_BASE_OCTAVE).contains(&octave) {
        Ok(octave)
    } else {
        Err(format!("oitava inválida: {value}; use um inteiro de 1 a 6"))
    }
}

fn parse_volume(value: &str) -> Result<f32, String> {
    let volume: f32 = value
        .parse()
        .map_err(|_| format!("volume inválido: {value}; use um número de 0.0 a 1.0"))?;
    if volume.is_finite() && (0.0..=1.0).contains(&volume) {
        Ok(volume)
    } else {
        Err(format!(
            "volume inválido: {value}; use um número de 0.0 a 1.0"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("piano-duas-oitavas").chain(args.iter().copied()))
    }

    #[test]
    fn provides_documented_defaults() {
        let cli = parse(&[]).unwrap();
        assert_eq!(cli.lang, LangArg::Pt);
        assert_eq!(cli.backend, Backend::Auto);
        assert_eq!(cli.fb_device, PathBuf::from("/dev/fb0"));
        assert_eq!(cli.octave, 3);
        assert!((cli.volume - 0.32).abs() < f32::EPSILON);
        assert_eq!(cli.seed, None);
        assert_eq!(cli.color, ColorMode::Auto);
        assert!(!cli.no_animations);
        assert!(!cli.no_audio);
        assert_eq!(cli.capture_frame, None);
    }

    #[test]
    fn accepts_documented_options() {
        let cli = parse(&[
            "--lang",
            "en",
            "--backend",
            "fb",
            "--fb-device",
            "/dev/fb1",
            "--octave",
            "6",
            "--volume",
            "1",
            "--seed",
            "42",
            "--color",
            "256",
            "--no-animations",
            "--no-audio",
            "--capture-frame",
            "/tmp/frame.png",
        ])
        .unwrap();
        assert_eq!(cli.lang, LangArg::En);
        assert_eq!(cli.backend, Backend::Fb);
        assert_eq!(cli.fb_device, PathBuf::from("/dev/fb1"));
        assert_eq!(cli.octave, 6);
        assert!((cli.volume - 1.0).abs() < f32::EPSILON);
        assert_eq!(cli.seed, Some(42));
        assert_eq!(cli.color, ColorMode::Color256);
        assert!(cli.no_animations);
        assert!(cli.no_audio);
        assert_eq!(cli.capture_frame, Some(PathBuf::from("/tmp/frame.png")));
    }

    #[test]
    fn rejects_out_of_range_octave() {
        assert!(parse(&["--octave", "0"]).is_err());
        assert!(parse(&["--octave", "7"]).is_err());
        assert!(parse(&["--octave", "abc"]).is_err());
    }

    #[test]
    fn rejects_out_of_range_volume() {
        assert!(parse(&["--volume", "-0.1"]).is_err());
        assert!(parse(&["--volume", "1.1"]).is_err());
        assert!(parse(&["--volume", "NaN"]).is_err());
        assert!(parse(&["--volume", "abc"]).is_err());
    }

    #[test]
    fn rejects_unknown_language_and_backend() {
        assert!(parse(&["--lang", "es"]).is_err());
        assert!(parse(&["--backend", "gpu"]).is_err());
    }
}
