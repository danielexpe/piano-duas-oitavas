use crate::app::App;
use crate::core::keymap;
use crate::core::note::MidiNote;
use crate::i18n::{self, Key, Language};

pub const KEYBOARD_ROWS: [&str; 4] = ["1234567890", "QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"];
pub const BASE_OCTAVE_KEYCAPS: [char; 12] =
    ['Z', 'S', 'X', 'D', 'C', 'V', 'G', 'B', 'H', 'N', 'J', 'M'];

#[derive(Debug, Clone, PartialEq)]
pub struct Hud {
    pub octave: String,
    pub volume: String,
    pub sound: String,
    pub sustain: String,
    pub language: String,
    pub keyup: String,
    pub muted: bool,
    pub sustain_active: bool,
    pub audio_unavailable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoteView {
    pub note: MidiNote,
    pub key: char,
    pub white: bool,
    pub held: bool,
    pub active: bool,
    pub press: f32,
    pub glow: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Keycap {
    pub label: char,
    pub note: Option<MidiNote>,
    pub held: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatingNote {
    pub note: MidiNote,
    pub progress: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub language: Language,
    pub hud: Hud,
    pub notes: Vec<NoteView>,
    pub keyboard: Vec<Vec<Keycap>>,
    pub help_open: bool,
    pub floating: Option<FloatingNote>,
}

impl Scene {
    pub fn from_app(app: &App) -> Self {
        let session = app.session();
        let language = session.language();
        let base = session.base_octave();
        let voices = session.voices();
        let now = app.now();
        let animations = !app.no_animations();

        let notes = session
            .notes()
            .map(|note| NoteView {
                note,
                key: keymap::key_for_note(note, base).unwrap_or(' '),
                white: note.is_white(),
                held: voices.is_held(note),
                active: voices.active_notes().any(|active| active == note),
                press: if animations {
                    app.effects().press_strength(note, now)
                } else {
                    0.0
                },
                glow: if animations {
                    app.effects().glow_strength(note, now)
                } else {
                    0.0
                },
            })
            .collect();

        let keyboard = KEYBOARD_ROWS
            .iter()
            .map(|row| {
                row.chars()
                    .map(|label| {
                        let note = keymap::note_for_key(label, base);
                        Keycap {
                            label,
                            note,
                            held: note.is_some_and(|note| voices.is_held(note)),
                        }
                    })
                    .collect()
            })
            .collect();

        let hud = Hud {
            octave: i18n::fill(
                Key::HudOctave.text(language),
                &[
                    ("base", &base.to_string()),
                    ("next", &session.next_octave().to_string()),
                ],
            ),
            volume: i18n::fill(
                Key::HudVolume.text(language),
                &[("pct", &session.volume_percent().to_string())],
            ),
            sound: i18n::fill(
                Key::HudSound.text(language),
                &[("state", session.sound_state_key().text(language))],
            ),
            sustain: i18n::fill(
                Key::HudSustain.text(language),
                &[("state", session.sustain_state_key().text(language))],
            ),
            language: i18n::fill(
                Key::HudLanguage.text(language),
                &[("lang", language.label())],
            ),
            keyup: i18n::fill(
                Key::HudKeyup.text(language),
                &[("state", session.keyup_state_key().text(language))],
            ),
            muted: session.muted(),
            sustain_active: session.sustain() != crate::state::SustainMode::Off,
            audio_unavailable: app.audio_unavailable(),
        };

        Self {
            language,
            hud,
            notes,
            keyboard,
            help_open: session.help_open(),
            floating: app
                .floating_note()
                .map(|(note, progress)| FloatingNote { note, progress }),
        }
    }

    pub fn white_keys(&self) -> impl Iterator<Item = &NoteView> {
        self.notes.iter().filter(|note| note.white)
    }

    pub fn black_keys(&self) -> impl Iterator<Item = &NoteView> {
        self.notes.iter().filter(|note| !note.white)
    }

    pub fn keycap(&self, label: char) -> Option<&Keycap> {
        self.keyboard
            .iter()
            .flatten()
            .find(|keycap| keycap.label == label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::input::Action;
    use crate::state::Session;

    fn app() -> App {
        App::new(Session::new(3, 0.32, Language::Pt, false).unwrap())
    }

    #[test]
    fn builds_two_octaves_with_white_and_black_keys() {
        let scene = Scene::from_app(&app());
        assert_eq!(scene.notes.len(), 24);
        assert_eq!(scene.white_keys().count(), 14);
        assert_eq!(scene.black_keys().count(), 10);
        assert_eq!(scene.notes[0].note.name(), "C3");
        assert_eq!(scene.notes[0].key, 'z');
    }

    #[test]
    fn marks_pressed_piano_and_physical_keys() {
        let mut app = app();
        app.handle(Action::NoteOn(MidiNote::from_midi(48).unwrap()));
        let scene = Scene::from_app(&app);
        assert!(scene.notes[0].held);
        assert!(scene.notes[0].active);
        assert!(scene.notes[0].press > 0.0);
        assert_eq!(
            scene.floating.map(|floating| floating.note),
            Some(MidiNote::from_midi(48).unwrap())
        );
        let keycap = scene.keycap('Z').unwrap();
        assert!(keycap.held);
        assert_eq!(keycap.note, Some(MidiNote::from_midi(48).unwrap()));
    }

    #[test]
    fn keyboard_has_exactly_twenty_four_mapped_keycaps() {
        let scene = Scene::from_app(&app());
        let mapped = scene
            .keyboard
            .iter()
            .flatten()
            .filter(|keycap| keycap.note.is_some())
            .count();
        assert_eq!(mapped, 24);
    }

    #[test]
    fn hud_uses_catalog_language_and_state() {
        let mut app = app();
        let scene = Scene::from_app(&app);
        assert_eq!(scene.hud.octave, "Oitava 3 (C3–B4)");
        assert_eq!(scene.hud.volume, "Vol 32%");
        assert!(scene.hud.sound.contains("ligado"));
        app.handle(Action::ToggleLanguage);
        let scene = Scene::from_app(&app);
        assert_eq!(scene.hud.octave, "Octave 3 (C3–B4)");
        assert_eq!(scene.hud.volume, "Vol 32%");
        assert!(scene.hud.sound.contains("on"));
    }

    #[test]
    fn exposes_last_note_while_visible() {
        let mut app = app();
        app.handle(Action::NoteOn(MidiNote::from_midi(60).unwrap()));
        let scene = Scene::from_app(&app);
        assert_eq!(
            scene.floating.map(|floating| floating.note),
            Some(MidiNote::from_midi(60).unwrap())
        );
    }

    #[test]
    fn hides_floating_note_when_animations_are_off() {
        let mut app = App::with_options(Session::new(3, 0.32, Language::Pt, false).unwrap(), true);
        app.handle(Action::NoteOn(MidiNote::from_midi(60).unwrap()));
        let scene = Scene::from_app(&app);
        assert!(scene.floating.is_none());
        assert_eq!(scene.notes[12].press, 0.0);
    }

    #[test]
    fn base_octave_keycaps_cover_every_note_of_the_base_octave() {
        let scene = Scene::from_app(&app());
        let notes: Vec<MidiNote> = BASE_OCTAVE_KEYCAPS
            .iter()
            .filter_map(|label| scene.keycap(*label))
            .filter_map(|keycap| keycap.note)
            .collect();
        assert_eq!(notes.len(), 12);
        assert!(notes.iter().all(|note| (48..60).contains(&note.midi())));
    }
}
