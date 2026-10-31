use super::note::MidiNote;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyBinding {
    pub key: char,
    pub offset: u8,
}

pub const KEY_BINDINGS: [KeyBinding; 24] = [
    KeyBinding {
        key: 'z',
        offset: 0,
    },
    KeyBinding {
        key: 's',
        offset: 1,
    },
    KeyBinding {
        key: 'x',
        offset: 2,
    },
    KeyBinding {
        key: 'd',
        offset: 3,
    },
    KeyBinding {
        key: 'c',
        offset: 4,
    },
    KeyBinding {
        key: 'v',
        offset: 5,
    },
    KeyBinding {
        key: 'g',
        offset: 6,
    },
    KeyBinding {
        key: 'b',
        offset: 7,
    },
    KeyBinding {
        key: 'h',
        offset: 8,
    },
    KeyBinding {
        key: 'n',
        offset: 9,
    },
    KeyBinding {
        key: 'j',
        offset: 10,
    },
    KeyBinding {
        key: 'm',
        offset: 11,
    },
    KeyBinding {
        key: 'q',
        offset: 12,
    },
    KeyBinding {
        key: '2',
        offset: 13,
    },
    KeyBinding {
        key: 'w',
        offset: 14,
    },
    KeyBinding {
        key: '3',
        offset: 15,
    },
    KeyBinding {
        key: 'e',
        offset: 16,
    },
    KeyBinding {
        key: 'r',
        offset: 17,
    },
    KeyBinding {
        key: '5',
        offset: 18,
    },
    KeyBinding {
        key: 't',
        offset: 19,
    },
    KeyBinding {
        key: '6',
        offset: 20,
    },
    KeyBinding {
        key: 'y',
        offset: 21,
    },
    KeyBinding {
        key: '7',
        offset: 22,
    },
    KeyBinding {
        key: 'u',
        offset: 23,
    },
];

pub fn normalize(key: char) -> Option<char> {
    match key {
        'A'..='Z' => Some(key.to_ascii_lowercase()),
        '@' => Some('2'),
        '#' => Some('3'),
        '%' => Some('5'),
        '&' => Some('7'),
        key if key.is_ascii() => Some(key),
        _ => None,
    }
}

pub fn note_for_key(key: char, base_octave: u8) -> Option<MidiNote> {
    let key = normalize(key)?;
    let binding = KEY_BINDINGS.iter().find(|binding| binding.key == key)?;
    let midi = 12u16 * (u16::from(base_octave) + 1) + u16::from(binding.offset);
    u8::try_from(midi).ok().and_then(MidiNote::from_midi)
}

pub fn key_for_note(note: MidiNote, base_octave: u8) -> Option<char> {
    let base = 12u16 * (u16::from(base_octave) + 1);
    let offset = u16::from(note.midi()).checked_sub(base)?;
    let offset = u8::try_from(offset).ok()?;
    KEY_BINDINGS
        .iter()
        .find(|binding| binding.offset == offset)
        .map(|binding| binding.key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::note::PitchClass;
    use std::collections::HashSet;

    #[test]
    fn maps_twenty_four_unique_keys() {
        let keys: HashSet<char> = KEY_BINDINGS.iter().map(|binding| binding.key).collect();
        let offsets: HashSet<u8> = KEY_BINDINGS.iter().map(|binding| binding.offset).collect();
        assert_eq!(keys.len(), 24);
        assert_eq!(offsets.len(), 24);
    }

    #[test]
    fn maps_fourteen_white_and_ten_black_keys() {
        let white = KEY_BINDINGS
            .iter()
            .filter(|binding| offset_note(binding.offset).is_white())
            .count();
        let black = KEY_BINDINGS
            .iter()
            .filter(|binding| !offset_note(binding.offset).is_white())
            .count();
        assert_eq!(white, 14);
        assert_eq!(black, 10);
    }

    #[test]
    fn resolves_base_and_next_octaves() {
        assert_eq!(
            note_for_key('z', 3).unwrap(),
            MidiNote::from_pitch(PitchClass::C, 3).unwrap()
        );
        assert_eq!(
            note_for_key('m', 3).unwrap(),
            MidiNote::from_pitch(PitchClass::B, 3).unwrap()
        );
        assert_eq!(
            note_for_key('q', 3).unwrap(),
            MidiNote::from_pitch(PitchClass::C, 4).unwrap()
        );
        assert_eq!(
            note_for_key('u', 3).unwrap(),
            MidiNote::from_pitch(PitchClass::B, 4).unwrap()
        );
    }

    #[test]
    fn normalizes_letters_and_aliases() {
        assert_eq!(note_for_key('Z', 3), note_for_key('z', 3));
        assert_eq!(note_for_key('Q', 3), note_for_key('q', 3));
        assert_eq!(note_for_key('@', 3), note_for_key('2', 3));
        assert_eq!(note_for_key('#', 3), note_for_key('3', 3));
        assert_eq!(note_for_key('%', 3), note_for_key('5', 3));
        assert_eq!(note_for_key('&', 3), note_for_key('7', 3));
        assert_eq!(note_for_key('¨', 3), None);
    }

    #[test]
    fn rejects_unmapped_keys_and_out_of_range_notes() {
        assert_eq!(note_for_key('a', 3), None);
        assert_eq!(note_for_key('4', 3), None);
        assert_eq!(note_for_key('z', 0).unwrap().midi(), 12);
        assert_eq!(note_for_key('u', 6).unwrap().midi(), 107);
        assert_eq!(note_for_key('u', 8), None);
    }

    #[test]
    fn reverses_note_to_key() {
        assert_eq!(key_for_note(MidiNote::from_midi(60).unwrap(), 3), Some('q'));
        assert_eq!(key_for_note(MidiNote::from_midi(48).unwrap(), 3), Some('z'));
        assert_eq!(key_for_note(MidiNote::from_midi(47).unwrap(), 3), None);
        assert_eq!(key_for_note(MidiNote::from_midi(72).unwrap(), 3), None);
    }

    fn offset_note(offset: u8) -> MidiNote {
        MidiNote::from_midi(48 + offset).unwrap()
    }
}
