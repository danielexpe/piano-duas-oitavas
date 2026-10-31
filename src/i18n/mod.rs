#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    Pt,
    En,
}

impl Language {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Pt => "pt",
            Self::En => "en",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Pt => "PT-BR",
            Self::En => "EN",
        }
    }

    pub const fn toggle(self) -> Self {
        match self {
            Self::Pt => Self::En,
            Self::En => Self::Pt,
        }
    }
}

macro_rules! catalog {
    ($($variant:ident => { pt: $pt:expr, en: $en:expr }),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Key {
            $($variant),+
        }

        impl Key {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            pub const fn text(self, language: Language) -> &'static str {
                match (self, language) {
                    $((Self::$variant, Language::Pt) => $pt,)+
                    $((Self::$variant, Language::En) => $en),+
                }
            }
        }
    };
}

catalog! {
    AppTitle => { pt: "Piano Duas Oitavas", en: "Piano Duas Oitavas" },
    AppSubtitle => { pt: "Duas oitavas, 24 teclas, um teclado.", en: "Two octaves, 24 keys, one keyboard." },
    HudOctave => { pt: "Oitava {base} (C{base}–B{next})", en: "Octave {base} (C{base}–B{next})" },
    HudVolume => { pt: "Vol {pct}%", en: "Vol {pct}%" },
    HudSound => { pt: "Som: {state}", en: "Sound: {state}" },
    HudMuted => { pt: "MUDO", en: "MUTED" },
    HudAudioUnavailable => { pt: "Sem áudio", en: "No audio" },
    HudSustain => { pt: "Sustain: {state}", en: "Sustain: {state}" },
    HudLanguage => { pt: "Idioma: {lang}", en: "Language: {lang}" },
    HudKeyup => { pt: "Key-up: {state}", en: "Key-up: {state}" },
    StateOn => { pt: "ligado", en: "on" },
    StateOff => { pt: "desligado", en: "off" },
    StateLatch => { pt: "latch", en: "latch" },
    StateReal => { pt: "real", en: "real" },
    StateAuto => { pt: "auto", en: "auto" },
    LangPt => { pt: "PT-BR", en: "PT-BR" },
    LangEn => { pt: "EN", en: "EN" },
    KeyboardTitle => { pt: "Teclado físico", en: "Computer keyboard" },
    HelpTitle => { pt: "Ajuda", en: "Help" },
    HelpClose => { pt: "F1 ou Esc para fechar", en: "F1 or Esc to close" },
    HelpNotesTitle => { pt: "Notas e teclas", en: "Notes and keys" },
    HelpNotesWhite => { pt: "Brancas: Z X C V B N M / Q W E R T Y U", en: "White: Z X C V B N M / Q W E R T Y U" },
    HelpNotesBlack => { pt: "Pretas: S D G H J / 2 3 5 6 7", en: "Black: S D G H J / 2 3 5 6 7" },
    HelpSolfeges => { pt: "Solfejo: Dó Ré Mi Fá Sol Lá Si", en: "Letters: C D E F G A B" },
    HelpControlsTitle => { pt: "Controles", en: "Controls" },
    HelpSustain => { pt: "Espaço: sustain (segurar; no fallback, alterna)", en: "Space: sustain (hold; toggles in fallback)" },
    HelpOctave => { pt: "Esq./Dir.: muda a oitava base", en: "Left/Right: shift base octave" },
    HelpVolume => { pt: "+ ou -: volume em passos de 5%", en: "+ or -: volume in 5% steps" },
    HelpMute => { pt: "F3: liga/desliga o som", en: "F3: toggle sound" },
    HelpLanguage => { pt: "F2: português/inglês", en: "F2: Portuguese/English" },
    HelpHelp => { pt: "F1: abre/fecha esta ajuda", en: "F1: open/close this help" },
    HelpQuit => { pt: "Esc: fecha a ajuda ou sai", en: "Esc: close help or quit" },
    HelpKeyupReal => { pt: "Key-up real: soltar a tecla encerra a nota.", en: "Real key-up: releasing a key ends the note." },
    HelpKeyupFallback => { pt: "Sem key-up: cada nota dura 750 ms; Espaço alterna o sustain.", en: "No key-up: each note lasts 750 ms; Space toggles sustain." },
    HelpNoPersistence => { pt: "Preferências não são salvas.", en: "Preferences are not saved." },
    FooterHelp => { pt: "F1 ajuda", en: "F1 help" },
    FooterLanguage => { pt: "F2 idioma", en: "F2 language" },
    FooterMute => { pt: "F3 mudo", en: "F3 mute" },
    FooterQuit => { pt: "Esc sair", en: "Esc quit" },
    ErrorNoTty => { pt: "Terminal interativo indisponível; use um terminal local ou --backend fb.", en: "Interactive terminal unavailable; use a local terminal or --backend fb." },
    ErrorFbPermission => { pt: "Sem permissão para {path}; entre no grupo video ou ajuste as permissões.", en: "No permission for {path}; join the video group or fix permissions." },
    ErrorFbVt => { pt: "O backend fb exige um VT de texto local; tty atual: {tty}.", en: "The fb backend requires a local text VT; current tty: {tty}." },
    ErrorAudioUnavailable => { pt: "Áudio indisponível; o piano continua sem som.", en: "Audio unavailable; the piano continues silently." },
}

pub fn t(language: Language, key: Key) -> &'static str {
    key.text(language)
}

pub fn fill(template: &str, arguments: &[(&str, &str)]) -> String {
    let mut text = template.to_string();
    for (name, value) in arguments {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_has_both_languages() {
        for key in Key::ALL {
            let pt = key.text(Language::Pt);
            let en = key.text(Language::En);
            assert!(!pt.is_empty(), "{key:?} missing pt");
            assert!(!en.is_empty(), "{key:?} missing en");
        }
    }

    #[test]
    fn brand_is_identical_in_both_languages() {
        assert_eq!(
            Key::AppTitle.text(Language::Pt),
            Key::AppTitle.text(Language::En)
        );
    }

    #[test]
    fn fills_placeholders() {
        let text = fill(
            Key::HudOctave.text(Language::Pt),
            &[("base", "3"), ("next", "4")],
        );
        assert_eq!(text, "Oitava 3 (C3–B4)");
        let text = fill(Key::HudVolume.text(Language::En), &[("pct", "32")]);
        assert_eq!(text, "Vol 32%");
        let text = fill(Key::HudSustain.text(Language::Pt), &[("state", "latch")]);
        assert_eq!(text, "Sustain: latch");
    }

    #[test]
    fn toggles_between_languages() {
        assert_eq!(Language::Pt.toggle(), Language::En);
        assert_eq!(Language::En.toggle(), Language::Pt);
        assert_eq!(Language::Pt.code(), "pt");
        assert_eq!(Language::En.code(), "en");
    }

    #[test]
    fn help_lines_fit_compact_width() {
        for key in [
            Key::HelpNotesWhite,
            Key::HelpNotesBlack,
            Key::HelpSolfeges,
            Key::HelpSustain,
            Key::HelpOctave,
            Key::HelpVolume,
            Key::HelpMute,
            Key::HelpLanguage,
            Key::HelpHelp,
            Key::HelpQuit,
            Key::HelpKeyupReal,
            Key::HelpKeyupFallback,
            Key::HelpNoPersistence,
            Key::HelpClose,
        ] {
            for language in [Language::Pt, Language::En] {
                assert!(
                    key.text(language).chars().count() <= 72,
                    "{key:?} in {language:?} exceeds 72 characters"
                );
            }
        }
    }
}
