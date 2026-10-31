use std::time::Duration;

use crate::core::note::MidiNote;
use crate::core::voice::AUTO_RELEASE;
use crate::input::Action;
use crate::state::{effects, Effects, KeyUpMode, Session, SustainMode, VOLUME_STEP};

pub const LAST_NOTE_DISPLAY: Duration = Duration::from_millis(effects::FLOAT_MS);

#[derive(Debug, Clone)]
pub struct App {
    session: Session,
    quit: bool,
    now: Duration,
    effects: Effects,
    no_animations: bool,
}

impl App {
    pub const fn new(session: Session) -> Self {
        Self::with_options(session, false)
    }

    pub const fn with_options(session: Session, no_animations: bool) -> Self {
        Self {
            session,
            quit: false,
            now: Duration::ZERO,
            effects: Effects::new(),
            no_animations,
        }
    }

    pub const fn session(&self) -> &Session {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut Session {
        &mut self.session
    }

    pub const fn effects(&self) -> &Effects {
        &self.effects
    }

    pub const fn no_animations(&self) -> bool {
        self.no_animations
    }

    pub const fn should_quit(&self) -> bool {
        self.quit
    }

    pub const fn now(&self) -> Duration {
        self.now
    }

    pub fn last_note_visible(&self) -> Option<MidiNote> {
        if self.no_animations {
            return None;
        }
        self.effects.floating(self.now).map(|(note, _)| note)
    }

    pub fn floating_note(&self) -> Option<(MidiNote, f32)> {
        if self.no_animations {
            return None;
        }
        self.effects.floating(self.now)
    }

    pub fn animating(&self) -> bool {
        !self.no_animations
            && (self.session.sustain() != SustainMode::Off || self.effects.any_active(self.now))
    }

    pub fn advance(&mut self, now: Duration) {
        if now < self.now {
            return;
        }
        self.now = now;
        if self.session.sustain() == SustainMode::Off {
            let expired = self.session.voices_mut().expire(now, AUTO_RELEASE);
            for note in expired {
                self.effects.release(note, now);
            }
        }
    }

    pub fn handle(&mut self, action: Action) {
        match action {
            Action::Quit => self.quit = true,
            Action::CloseOrQuit => {
                if self.session.help_open() {
                    self.session.close_help();
                } else {
                    self.quit = true;
                }
            }
            Action::ToggleLanguage => self.session.toggle_language(),
            Action::ToggleMute => self.session.toggle_mute(),
            Action::ToggleHelp => self.session.toggle_help(),
            _ if self.session.help_open() => {}
            Action::OctaveDown => {
                if self.session.shift_octave(-1) {
                    self.effects.reset();
                }
            }
            Action::OctaveUp => {
                if self.session.shift_octave(1) {
                    self.effects.reset();
                }
            }
            Action::VolumeDown => self.session.adjust_volume(-VOLUME_STEP),
            Action::VolumeUp => self.session.adjust_volume(VOLUME_STEP),
            Action::SustainPress => self.sustain_press(),
            Action::SustainRelease => self.sustain_release(),
            Action::NoteOn(note) => self.note_on(note),
            Action::NoteOff(note) => self.note_off(note),
            Action::Tick => {}
        }
    }

    pub fn release_all(&mut self) {
        self.session.release_all();
        self.effects.release_all(self.now);
    }

    fn note_on(&mut self, note: MidiNote) {
        self.session.voices_mut().note_on(note, self.now);
        self.effects.press(note, self.now);
    }

    fn note_off(&mut self, note: MidiNote) {
        if self.session.voices_mut().note_off(note).is_some() {
            self.effects.release(note, self.now);
        }
    }

    fn sustain_press(&mut self) {
        match self.session.keyup() {
            KeyUpMode::Auto => {
                self.session.toggle_latch();
                if self.session.sustain() == SustainMode::Off {
                    self.session.voices_mut().release_all();
                    self.effects.release_all(self.now);
                }
            }
            KeyUpMode::Real => self.session.set_sustain(SustainMode::On),
        }
    }

    fn sustain_release(&mut self) {
        if self.session.keyup() == KeyUpMode::Real && self.session.sustain() == SustainMode::On {
            self.session.voices_mut().release_all();
            self.session.set_sustain(SustainMode::Off);
            self.effects.release_all(self.now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::note::PitchClass;
    use crate::i18n::Language;

    fn app() -> App {
        App::new(Session::new(3, 0.32, Language::Pt, false).unwrap())
    }

    fn note(pitch: PitchClass) -> MidiNote {
        MidiNote::from_pitch(pitch, 3).unwrap()
    }

    fn at(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    #[test]
    fn note_on_marks_voice_and_floating_note() {
        let mut app = app();
        app.handle(Action::NoteOn(note(PitchClass::C)));
        assert!(app.session().voices().is_held(note(PitchClass::C)));
        assert_eq!(app.last_note_visible(), Some(note(PitchClass::C)));
        app.advance(at(600));
        assert_eq!(app.last_note_visible(), None);
    }

    #[test]
    fn auto_releases_after_750_milliseconds() {
        let mut app = app();
        app.handle(Action::NoteOn(note(PitchClass::C)));
        app.advance(at(749));
        assert!(app.session().voices().is_held(note(PitchClass::C)));
        app.advance(at(750));
        assert!(!app.session().voices().is_held(note(PitchClass::C)));
    }

    #[test]
    fn help_ignores_notes_and_closes_with_escape() {
        let mut app = app();
        app.handle(Action::ToggleHelp);
        assert!(app.session().help_open());
        app.handle(Action::NoteOn(note(PitchClass::C)));
        assert_eq!(app.session().voices().held_count(), 0);
        app.handle(Action::CloseOrQuit);
        assert!(!app.session().help_open());
        assert!(!app.should_quit());
        app.handle(Action::CloseOrQuit);
        assert!(app.should_quit());
    }

    #[test]
    fn language_and_mute_work_with_help_open() {
        let mut app = app();
        app.handle(Action::ToggleHelp);
        app.handle(Action::ToggleLanguage);
        app.handle(Action::ToggleMute);
        assert_eq!(app.session().language(), Language::En);
        assert!(app.session().muted());
        assert!(app.session().help_open());
    }

    #[test]
    fn octave_shift_clears_voices_sustain_and_effects() {
        let mut app = app();
        app.handle(Action::NoteOn(note(PitchClass::C)));
        app.handle(Action::SustainPress);
        app.handle(Action::OctaveUp);
        assert_eq!(app.session().base_octave(), 4);
        assert_eq!(app.session().voices().active_count(), 0);
        assert_eq!(app.session().sustain(), SustainMode::Off);
        assert_eq!(app.last_note_visible(), None);
    }

    #[test]
    fn latch_holds_notes_until_toggled_off() {
        let mut app = app();
        app.handle(Action::NoteOn(note(PitchClass::C)));
        app.handle(Action::SustainPress);
        app.advance(at(2_000));
        assert!(app.session().voices().is_held(note(PitchClass::C)));
        app.handle(Action::SustainPress);
        app.advance(at(2_000));
        assert!(!app.session().voices().is_held(note(PitchClass::C)));
    }

    #[test]
    fn real_keyup_sustain_holds_and_releases() {
        let mut app = app();
        app.session_mut().set_keyup(KeyUpMode::Real);
        app.handle(Action::NoteOn(note(PitchClass::C)));
        app.handle(Action::SustainPress);
        assert_eq!(app.session().sustain(), SustainMode::On);
        app.advance(at(2_000));
        assert!(app.session().voices().is_held(note(PitchClass::C)));
        app.handle(Action::SustainRelease);
        assert_eq!(app.session().sustain(), SustainMode::Off);
        assert_eq!(app.session().voices().held_count(), 0);
    }

    #[test]
    fn volume_and_mute_actions_update_session() {
        let mut app = app();
        app.handle(Action::VolumeUp);
        assert!((app.session().volume() - 0.37).abs() < 1e-6);
        app.handle(Action::VolumeDown);
        assert!((app.session().volume() - 0.32).abs() < 1e-6);
        app.handle(Action::ToggleMute);
        assert!(app.session().muted());
    }

    #[test]
    fn note_off_releases_voice_and_starts_glow() {
        let mut app = app();
        app.handle(Action::NoteOn(note(PitchClass::C)));
        app.handle(Action::NoteOff(note(PitchClass::C)));
        assert!(!app.session().voices().is_held(note(PitchClass::C)));
        app.advance(at(50));
        assert!(app.effects().glow_strength(note(PitchClass::C), app.now()) > 0.0);
    }

    #[test]
    fn no_animations_hides_floating_note_and_idle_frames() {
        let mut app = App::with_options(Session::new(3, 0.32, Language::Pt, false).unwrap(), true);
        app.handle(Action::NoteOn(note(PitchClass::C)));
        assert_eq!(app.last_note_visible(), None);
        assert!(!app.animating());
        app.advance(at(750));
        assert!(!app.session().voices().is_held(note(PitchClass::C)));
    }

    #[test]
    fn sustain_keeps_animating_frames() {
        let mut app = app();
        app.handle(Action::SustainPress);
        assert_eq!(app.session().sustain(), SustainMode::Latch);
        assert!(app.animating());
    }
}
