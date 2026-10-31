use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::core::keymap;
use crate::core::note::MidiNote;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    NoteOn(MidiNote),
    NoteOff(MidiNote),
    SustainPress,
    SustainRelease,
    OctaveDown,
    OctaveUp,
    VolumeDown,
    VolumeUp,
    ToggleMute,
    ToggleLanguage,
    ToggleHelp,
    CloseOrQuit,
    Quit,
    Tick,
}

pub fn map_key(event: &KeyEvent, base_octave: u8) -> Option<Action> {
    if event.kind == KeyEventKind::Repeat {
        return None;
    }

    let release = event.kind == KeyEventKind::Release;
    let control = event.modifiers.contains(KeyModifiers::CONTROL);
    let alt = event.modifiers.contains(KeyModifiers::ALT);

    if !release && control && matches!(event.code, KeyCode::Char('c' | 'C')) {
        return Some(Action::Quit);
    }

    if release {
        return match event.code {
            KeyCode::Char(' ') => Some(Action::SustainRelease),
            KeyCode::Char(key) if !control && !alt => {
                keymap::note_for_key(key, base_octave).map(Action::NoteOff)
            }
            _ => None,
        };
    }

    match event.code {
        KeyCode::Esc => Some(Action::CloseOrQuit),
        KeyCode::F(1) => Some(Action::ToggleHelp),
        KeyCode::F(2) => Some(Action::ToggleLanguage),
        KeyCode::F(3) => Some(Action::ToggleMute),
        KeyCode::Left => Some(Action::OctaveDown),
        KeyCode::Right => Some(Action::OctaveUp),
        KeyCode::Char(' ') => Some(Action::SustainPress),
        KeyCode::Char('+' | '=') => Some(Action::VolumeUp),
        KeyCode::Char('-' | '_') => Some(Action::VolumeDown),
        KeyCode::Char(key) if !control && !alt => {
            keymap::note_for_key(key, base_octave).map(Action::NoteOn)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::note::PitchClass;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn note(pitch: PitchClass, octave: u8) -> MidiNote {
        MidiNote::from_pitch(pitch, octave).unwrap()
    }

    #[test]
    fn maps_note_keys() {
        assert_eq!(
            map_key(&key(KeyCode::Char('z')), 3),
            Some(Action::NoteOn(note(PitchClass::C, 3)))
        );
        assert_eq!(
            map_key(&key(KeyCode::Char('q')), 3),
            Some(Action::NoteOn(note(PitchClass::C, 4)))
        );
        assert_eq!(
            map_key(&key(KeyCode::Char('M')), 3),
            Some(Action::NoteOn(note(PitchClass::B, 3)))
        );
        assert_eq!(map_key(&key(KeyCode::Char('a')), 3), None);
    }

    #[test]
    fn maps_aliases_and_shifted_keys() {
        assert_eq!(
            map_key(&key(KeyCode::Char('@')), 3),
            Some(Action::NoteOn(note(PitchClass::CSharp, 4)))
        );
        assert_eq!(
            map_key(&key(KeyCode::Char('#')), 3),
            Some(Action::NoteOn(note(PitchClass::DSharp, 4)))
        );
    }

    #[test]
    fn maps_global_shortcuts() {
        assert_eq!(map_key(&key(KeyCode::F(1)), 3), Some(Action::ToggleHelp));
        assert_eq!(
            map_key(&key(KeyCode::F(2)), 3),
            Some(Action::ToggleLanguage)
        );
        assert_eq!(map_key(&key(KeyCode::F(3)), 3), Some(Action::ToggleMute));
        assert_eq!(map_key(&key(KeyCode::Left), 3), Some(Action::OctaveDown));
        assert_eq!(map_key(&key(KeyCode::Right), 3), Some(Action::OctaveUp));
        assert_eq!(map_key(&key(KeyCode::Char('=')), 3), Some(Action::VolumeUp));
        assert_eq!(map_key(&key(KeyCode::Char('+')), 3), Some(Action::VolumeUp));
        assert_eq!(
            map_key(&key(KeyCode::Char('-')), 3),
            Some(Action::VolumeDown)
        );
        assert_eq!(map_key(&key(KeyCode::Esc), 3), Some(Action::CloseOrQuit));
    }

    #[test]
    fn ignores_repeat_events() {
        let repeat =
            KeyEvent::new_with_kind(KeyCode::Char('z'), KeyModifiers::NONE, KeyEventKind::Repeat);
        assert_eq!(map_key(&repeat, 3), None);
    }

    #[test]
    fn maps_release_to_note_off_and_sustain_release() {
        let release = KeyEvent::new_with_kind(
            KeyCode::Char('z'),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        );
        assert_eq!(
            map_key(&release, 3),
            Some(Action::NoteOff(note(PitchClass::C, 3)))
        );
        let space = KeyEvent::new_with_kind(
            KeyCode::Char(' '),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        );
        assert_eq!(map_key(&space, 3), Some(Action::SustainRelease));
    }

    #[test]
    fn maps_ctrl_c_to_quit() {
        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(map_key(&event, 3), Some(Action::Quit));
    }

    #[test]
    fn ignores_ctrl_or_alt_note_keys() {
        let event = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::ALT);
        assert_eq!(map_key(&event, 3), None);
        let event = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL);
        assert_eq!(map_key(&event, 3), None);
    }
}
