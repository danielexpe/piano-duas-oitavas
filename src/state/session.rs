use crate::core::note::{MidiNote, MAX_BASE_OCTAVE, MIN_BASE_OCTAVE};
use crate::core::voice::VoiceAllocator;
use crate::i18n::{Key, Language};

pub const DEFAULT_BASE_OCTAVE: u8 = 3;
pub const DEFAULT_VOLUME: f32 = 0.32;
pub const VOLUME_STEP: f32 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Piano,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SustainMode {
    Off,
    On,
    Latch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyUpMode {
    Real,
    Auto,
}

#[derive(Debug, Clone)]
pub struct Session {
    base_octave: u8,
    volume: f32,
    muted: bool,
    language: Language,
    screen: Screen,
    sustain: SustainMode,
    keyup: KeyUpMode,
    voices: VoiceAllocator,
}

impl Session {
    pub fn new(base_octave: u8, volume: f32, language: Language, muted: bool) -> Option<Self> {
        if !(MIN_BASE_OCTAVE..=MAX_BASE_OCTAVE).contains(&base_octave) {
            return None;
        }
        if !volume.is_finite() || !(0.0..=1.0).contains(&volume) {
            return None;
        }
        Some(Self {
            base_octave,
            volume,
            muted,
            language,
            screen: Screen::Piano,
            sustain: SustainMode::Off,
            keyup: KeyUpMode::Auto,
            voices: VoiceAllocator::new(),
        })
    }

    pub const fn base_octave(&self) -> u8 {
        self.base_octave
    }

    pub const fn next_octave(&self) -> u8 {
        self.base_octave + 1
    }

    pub const fn volume(&self) -> f32 {
        self.volume
    }

    pub fn volume_percent(&self) -> u8 {
        (self.volume * 100.0).round() as u8
    }

    pub const fn muted(&self) -> bool {
        self.muted
    }

    pub const fn language(&self) -> Language {
        self.language
    }

    pub const fn screen(&self) -> Screen {
        self.screen
    }

    pub fn help_open(&self) -> bool {
        self.screen == Screen::Help
    }

    pub const fn sustain(&self) -> SustainMode {
        self.sustain
    }

    pub const fn keyup(&self) -> KeyUpMode {
        self.keyup
    }

    pub const fn voices(&self) -> &VoiceAllocator {
        &self.voices
    }

    pub fn voices_mut(&mut self) -> &mut VoiceAllocator {
        &mut self.voices
    }

    pub fn notes(&self) -> impl Iterator<Item = MidiNote> + '_ {
        let base = u16::from(self.base_octave);
        (0..24u16).filter_map(move |offset| {
            let midi = 12 * (base + 1) + offset;
            u8::try_from(midi).ok().and_then(MidiNote::from_midi)
        })
    }

    pub fn shift_octave(&mut self, delta: i16) -> bool {
        let next = i16::from(self.base_octave) + delta;
        let next = next.clamp(i16::from(MIN_BASE_OCTAVE), i16::from(MAX_BASE_OCTAVE));
        let next = next as u8;
        if next == self.base_octave {
            return false;
        }
        self.base_octave = next;
        self.sustain = SustainMode::Off;
        self.voices.clear();
        true
    }

    pub fn adjust_volume(&mut self, delta: f32) {
        let volume = (self.volume + delta).clamp(0.0, 1.0);
        self.volume = (volume * 100.0).round() / 100.0;
    }

    pub const fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    pub const fn toggle_language(&mut self) {
        self.language = self.language.toggle();
    }

    pub fn toggle_help(&mut self) {
        self.screen = match self.screen {
            Screen::Piano => Screen::Help,
            Screen::Help => Screen::Piano,
        };
    }

    pub fn close_help(&mut self) {
        self.screen = Screen::Piano;
    }

    pub const fn set_sustain(&mut self, mode: SustainMode) {
        self.sustain = mode;
    }

    pub fn toggle_latch(&mut self) {
        self.sustain = match self.sustain {
            SustainMode::Off => SustainMode::Latch,
            SustainMode::On | SustainMode::Latch => SustainMode::Off,
        };
    }

    pub fn set_keyup(&mut self, mode: KeyUpMode) {
        self.keyup = mode;
    }

    pub fn release_all(&mut self) {
        self.voices.release_all();
        self.sustain = SustainMode::Off;
    }

    pub fn sound_state_key(&self) -> Key {
        if self.muted {
            Key::StateOff
        } else {
            Key::StateOn
        }
    }

    pub fn sustain_state_key(&self) -> Key {
        match self.sustain {
            SustainMode::Off => Key::StateOff,
            SustainMode::On => Key::StateOn,
            SustainMode::Latch => Key::StateLatch,
        }
    }

    pub fn keyup_state_key(&self) -> Key {
        match self.keyup {
            KeyUpMode::Real => Key::StateReal,
            KeyUpMode::Auto => Key::StateAuto,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::note::PitchClass;

    fn session() -> Session {
        Session::new(3, 0.32, Language::Pt, false).unwrap()
    }

    #[test]
    fn rejects_invalid_options() {
        assert!(Session::new(0, 0.32, Language::Pt, false).is_none());
        assert!(Session::new(7, 0.32, Language::Pt, false).is_none());
        assert!(Session::new(3, -0.1, Language::Pt, false).is_none());
        assert!(Session::new(3, 1.1, Language::Pt, false).is_none());
        assert!(Session::new(3, f32::NAN, Language::Pt, false).is_none());
    }

    #[test]
    fn lists_two_chromatic_octaves() {
        let session = session();
        let notes: Vec<MidiNote> = session.notes().collect();
        assert_eq!(notes.len(), 24);
        assert_eq!(notes.first().unwrap().midi(), 48);
        assert_eq!(notes.last().unwrap().midi(), 71);
        assert_eq!(notes.iter().filter(|note| note.is_white()).count(), 14);
        assert_eq!(notes.iter().filter(|note| !note.is_white()).count(), 10);
    }

    #[test]
    fn shifts_octave_within_limits_and_releases_notes() {
        let mut session = session();
        session.voices_mut().note_on(
            MidiNote::from_pitch(PitchClass::C, 3).unwrap(),
            std::time::Duration::ZERO,
        );
        session.set_sustain(SustainMode::Latch);
        assert!(session.shift_octave(-1));
        assert_eq!(session.base_octave(), 2);
        assert_eq!(session.sustain(), SustainMode::Off);
        assert_eq!(session.voices().active_count(), 0);
        assert!(session.shift_octave(-1));
        assert!(!session.shift_octave(-1));
        assert_eq!(session.base_octave(), 1);
        assert!(session.shift_octave(5));
        assert!(!session.shift_octave(1));
        assert_eq!(session.base_octave(), 6);
    }

    #[test]
    fn steps_and_clamps_volume() {
        let mut session = session();
        session.adjust_volume(VOLUME_STEP);
        assert!((session.volume() - 0.37).abs() < 1e-6);
        for _ in 0..100 {
            session.adjust_volume(VOLUME_STEP);
        }
        assert!((session.volume() - 1.0).abs() < f32::EPSILON);
        for _ in 0..100 {
            session.adjust_volume(-VOLUME_STEP);
        }
        assert!(session.volume().abs() < f32::EPSILON);
        assert_eq!(session.volume_percent(), 0);
    }

    #[test]
    fn toggles_mute_language_and_help() {
        let mut session = session();
        assert!(!session.muted());
        session.toggle_mute();
        assert!(session.muted());
        assert_eq!(session.language(), Language::Pt);
        session.toggle_language();
        assert_eq!(session.language(), Language::En);
        assert!(!session.help_open());
        session.toggle_help();
        assert!(session.help_open());
        session.toggle_help();
        assert!(!session.help_open());
    }

    #[test]
    fn latch_toggles_and_sound_state_reflects_mute() {
        let mut session = session();
        assert_eq!(session.sustain(), SustainMode::Off);
        session.toggle_latch();
        assert_eq!(session.sustain(), SustainMode::Latch);
        session.toggle_latch();
        assert_eq!(session.sustain(), SustainMode::Off);
        assert_eq!(session.sound_state_key(), Key::StateOn);
        session.toggle_mute();
        assert_eq!(session.sound_state_key(), Key::StateOff);
    }
}
