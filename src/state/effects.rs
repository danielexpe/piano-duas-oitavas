use std::time::Duration;

use crate::core::note::MidiNote;

pub const PRESS_MS: u64 = 90;
pub const GLOW_MS: u64 = 260;
pub const RIPPLE_MS: u64 = 220;
pub const FLOAT_MS: u64 = 600;

const NOTE_COUNT: usize = 128;

#[derive(Debug, Clone, Copy)]
struct NoteEffect {
    pressed_at: Duration,
    released_at: Option<Duration>,
}

#[derive(Debug, Clone, Copy)]
struct Floating {
    note: MidiNote,
    started_at: Duration,
}

#[derive(Debug, Clone)]
pub struct Effects {
    notes: [Option<NoteEffect>; NOTE_COUNT],
    floating: Option<Floating>,
    ripple: Option<(MidiNote, Duration)>,
}

impl Effects {
    pub const fn new() -> Self {
        Self {
            notes: [None; NOTE_COUNT],
            floating: None,
            ripple: None,
        }
    }

    pub fn press(&mut self, note: MidiNote, now: Duration) {
        self.notes[usize::from(note.midi())] = Some(NoteEffect {
            pressed_at: now,
            released_at: None,
        });
        self.floating = Some(Floating {
            note,
            started_at: now,
        });
        self.ripple = Some((note, now));
    }

    pub fn release(&mut self, note: MidiNote, now: Duration) {
        if let Some(effect) = self.notes[usize::from(note.midi())].as_mut() {
            if effect.released_at.is_none() {
                effect.released_at = Some(now);
            }
        }
    }

    pub fn release_all(&mut self, now: Duration) {
        for effect in self.notes.iter_mut().flatten() {
            if effect.released_at.is_none() {
                effect.released_at = Some(now);
            }
        }
    }

    pub fn reset(&mut self) {
        self.notes = [None; NOTE_COUNT];
        self.floating = None;
        self.ripple = None;
    }

    pub fn press_strength(&self, note: MidiNote, now: Duration) -> f32 {
        let Some(effect) = self.notes[usize::from(note.midi())] else {
            return 0.0;
        };
        if effect.released_at.is_some() {
            return 0.0;
        }
        attenuation(now.saturating_sub(effect.pressed_at), PRESS_MS)
    }

    pub fn glow_strength(&self, note: MidiNote, now: Duration) -> f32 {
        let Some(effect) = self.notes[usize::from(note.midi())] else {
            return 0.0;
        };
        let Some(released_at) = effect.released_at else {
            return 0.0;
        };
        attenuation(now.saturating_sub(released_at), GLOW_MS)
    }

    pub fn floating(&self, now: Duration) -> Option<(MidiNote, f32)> {
        let floating = self.floating?;
        let progress = progress(now.saturating_sub(floating.started_at), FLOAT_MS)?;
        Some((floating.note, progress))
    }

    pub fn ripple(&self, now: Duration) -> Option<f32> {
        let (_, started_at) = self.ripple?;
        progress(now.saturating_sub(started_at), RIPPLE_MS)
    }

    pub fn any_active(&self, now: Duration) -> bool {
        self.floating(now).is_some()
            || self.notes.iter().enumerate().any(|(midi, effect)| {
                if effect.is_none() {
                    return false;
                }
                let Some(note) = u8::try_from(midi).ok().and_then(MidiNote::from_midi) else {
                    return false;
                };
                self.press_strength(note, now) > 0.0 || self.glow_strength(note, now) > 0.0
            })
    }
}

impl Default for Effects {
    fn default() -> Self {
        Self::new()
    }
}

fn attenuation(elapsed: Duration, window_ms: u64) -> f32 {
    if window_ms == 0 {
        return 0.0;
    }
    let elapsed = elapsed.as_millis().min(u128::from(u64::MAX)) as u64;
    if elapsed >= window_ms {
        return 0.0;
    }
    let remaining = 1.0 - elapsed as f32 / window_ms as f32;
    remaining * remaining
}

fn progress(elapsed: Duration, window_ms: u64) -> Option<f32> {
    let elapsed = elapsed.as_millis().min(u128::from(u64::MAX)) as u64;
    (elapsed < window_ms).then_some(elapsed as f32 / window_ms as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::note::PitchClass;

    fn note(pitch: PitchClass) -> MidiNote {
        MidiNote::from_pitch(pitch, 3).unwrap()
    }

    fn at(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    #[test]
    fn press_strength_ramps_down_after_attack() {
        let mut effects = Effects::new();
        effects.press(note(PitchClass::C), at(0));
        assert_eq!(effects.press_strength(note(PitchClass::C), at(0)), 1.0);
        assert!(effects.press_strength(note(PitchClass::C), at(45)) < 1.0);
        assert_eq!(effects.press_strength(note(PitchClass::C), at(90)), 0.0);
    }

    #[test]
    fn glow_starts_on_release_and_fades() {
        let mut effects = Effects::new();
        effects.press(note(PitchClass::C), at(0));
        assert_eq!(effects.glow_strength(note(PitchClass::C), at(100)), 0.0);
        effects.release(note(PitchClass::C), at(100));
        assert_eq!(effects.glow_strength(note(PitchClass::C), at(100)), 1.0);
        assert!(effects.glow_strength(note(PitchClass::C), at(200)) < 1.0);
        assert_eq!(effects.glow_strength(note(PitchClass::C), at(360)), 0.0);
    }

    #[test]
    fn floating_note_is_visible_for_six_hundred_milliseconds() {
        let mut effects = Effects::new();
        effects.press(note(PitchClass::C), at(0));
        assert_eq!(effects.floating(at(0)), Some((note(PitchClass::C), 0.0)));
        let (_, progress) = effects.floating(at(300)).unwrap();
        assert!((progress - 0.5).abs() < 1e-6);
        assert_eq!(effects.floating(at(600)), None);
    }

    #[test]
    fn release_all_marks_only_once_and_reset_clears() {
        let mut effects = Effects::new();
        effects.press(note(PitchClass::C), at(0));
        effects.release_all(at(100));
        effects.release_all(at(200));
        assert_eq!(effects.glow_strength(note(PitchClass::C), at(100)), 1.0);
        let fading = effects.glow_strength(note(PitchClass::C), at(200));
        assert!(fading > 0.0 && fading < 1.0);
        effects.reset();
        assert_eq!(effects.glow_strength(note(PitchClass::C), at(100)), 0.0);
        assert_eq!(effects.floating(at(100)), None);
    }

    #[test]
    fn retrigger_restarts_press_and_floating_windows() {
        let mut effects = Effects::new();
        effects.press(note(PitchClass::C), at(0));
        effects.release(note(PitchClass::C), at(100));
        effects.press(note(PitchClass::C), at(500));
        assert_eq!(effects.glow_strength(note(PitchClass::C), at(500)), 0.0);
        assert_eq!(effects.floating(at(500)), Some((note(PitchClass::C), 0.0)));
    }

    #[test]
    fn any_active_covers_float_press_and_glow() {
        let mut effects = Effects::new();
        assert!(!effects.any_active(at(0)));
        effects.press(note(PitchClass::C), at(0));
        assert!(effects.any_active(at(0)));
        assert!(effects.any_active(at(300)));
        effects.release(note(PitchClass::C), at(600));
        assert!(effects.any_active(at(700)));
        assert!(!effects.any_active(at(900)));
    }

    #[test]
    fn ripple_is_bounded() {
        let mut effects = Effects::new();
        effects.press(note(PitchClass::E), at(10));
        assert_eq!(effects.ripple(at(10)), Some(0.0));
        assert!(effects.ripple(at(120)).unwrap() > 0.0);
        assert_eq!(effects.ripple(at(230)), None);
    }
}
