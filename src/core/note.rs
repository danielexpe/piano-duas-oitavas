use std::fmt;

pub const MIN_BASE_OCTAVE: u8 = 1;
pub const MAX_BASE_OCTAVE: u8 = 6;
pub const MAX_MIDI: u8 = 127;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PitchClass {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

impl PitchClass {
    pub const ALL: [Self; 12] = [
        Self::C,
        Self::CSharp,
        Self::D,
        Self::DSharp,
        Self::E,
        Self::F,
        Self::FSharp,
        Self::G,
        Self::GSharp,
        Self::A,
        Self::ASharp,
        Self::B,
    ];

    pub const fn semitone(self) -> u8 {
        match self {
            Self::C => 0,
            Self::CSharp => 1,
            Self::D => 2,
            Self::DSharp => 3,
            Self::E => 4,
            Self::F => 5,
            Self::FSharp => 6,
            Self::G => 7,
            Self::GSharp => 8,
            Self::A => 9,
            Self::ASharp => 10,
            Self::B => 11,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::CSharp => "C#",
            Self::D => "D",
            Self::DSharp => "D#",
            Self::E => "E",
            Self::F => "F",
            Self::FSharp => "F#",
            Self::G => "G",
            Self::GSharp => "G#",
            Self::A => "A",
            Self::ASharp => "A#",
            Self::B => "B",
        }
    }

    pub const fn is_white(self) -> bool {
        matches!(
            self,
            Self::C | Self::D | Self::E | Self::F | Self::G | Self::A | Self::B
        )
    }

    pub fn from_semitone(semitone: u8) -> Option<Self> {
        Self::ALL.get(usize::from(semitone)).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MidiNote(u8);

impl MidiNote {
    pub fn from_midi(value: u8) -> Option<Self> {
        (value <= MAX_MIDI).then_some(Self(value))
    }

    pub fn from_pitch(pitch: PitchClass, octave: u8) -> Option<Self> {
        let midi = 12u16 * (u16::from(octave) + 1) + u16::from(pitch.semitone());
        u8::try_from(midi).ok().and_then(Self::from_midi)
    }

    pub const fn midi(self) -> u8 {
        self.0
    }

    pub fn pitch_class(self) -> PitchClass {
        PitchClass::ALL[usize::from(self.0 % 12)]
    }

    pub fn octave(self) -> i16 {
        i16::from(self.0 / 12) - 1
    }

    pub fn name(self) -> String {
        format!("{}{}", self.pitch_class().name(), self.octave())
    }

    pub fn frequency(self) -> f64 {
        440.0 * 2f64.powf((f64::from(self.0) - 69.0) / 12.0)
    }

    pub fn is_white(self) -> bool {
        self.pitch_class().is_white()
    }
}

impl fmt::Display for MidiNote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_reference_midi_numbers() {
        assert_eq!(MidiNote::from_pitch(PitchClass::C, 3).unwrap().midi(), 48);
        assert_eq!(MidiNote::from_pitch(PitchClass::B, 4).unwrap().midi(), 71);
        assert_eq!(MidiNote::from_pitch(PitchClass::C, 1).unwrap().midi(), 24);
        assert_eq!(MidiNote::from_pitch(PitchClass::B, 7).unwrap().midi(), 107);
    }

    #[test]
    fn computes_names_and_octaves() {
        let note = MidiNote::from_pitch(PitchClass::CSharp, 3).unwrap();
        assert_eq!(note.name(), "C#3");
        assert_eq!(note.octave(), 3);
        assert_eq!(note.pitch_class(), PitchClass::CSharp);
    }

    #[test]
    fn computes_frequency_for_a4() {
        let a4 = MidiNote::from_midi(69).unwrap();
        assert!((a4.frequency() - 440.0).abs() < f64::EPSILON);
        assert!((a4.frequency() / 2.0 - 220.0).abs() < 1e-9);
    }

    #[test]
    fn identifies_white_and_black_keys() {
        let c = MidiNote::from_pitch(PitchClass::C, 3).unwrap();
        let c_sharp = MidiNote::from_pitch(PitchClass::CSharp, 3).unwrap();
        assert!(c.is_white());
        assert!(!c_sharp.is_white());
    }

    #[test]
    fn rejects_invalid_inputs() {
        assert!(MidiNote::from_midi(128).is_none());
        assert!(PitchClass::from_semitone(12).is_none());
        assert!(MidiNote::from_pitch(PitchClass::B, 10).is_none());
    }
}
