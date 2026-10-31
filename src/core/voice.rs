use std::time::Duration;

use super::note::MidiNote;

pub const VOICE_COUNT: usize = 8;
pub const AUTO_RELEASE: Duration = Duration::from_millis(750);
pub const RELEASE: Duration = Duration::from_millis(180);
pub const SUSTAIN_RELEASE: Duration = Duration::from_millis(450);
pub const STEAL_FADE: Duration = Duration::from_millis(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceAction {
    Started(usize),
    Retriggered(usize),
    Stolen {
        index: usize,
        previous: MidiNote,
        fade: Duration,
    },
}

#[derive(Debug, Clone, Copy)]
struct Slot {
    note: Option<MidiNote>,
    started_at: Duration,
    released_at: Option<Duration>,
    release_duration: Duration,
    age: u64,
    held: bool,
}

const EMPTY_SLOT: Slot = Slot {
    note: None,
    started_at: Duration::ZERO,
    released_at: None,
    release_duration: RELEASE,
    age: 0,
    held: false,
};

#[derive(Debug, Clone)]
pub struct VoiceAllocator {
    slots: [Slot; VOICE_COUNT],
    next_age: u64,
}

impl VoiceAllocator {
    pub const fn new() -> Self {
        Self {
            slots: [EMPTY_SLOT; VOICE_COUNT],
            next_age: 1,
        }
    }

    pub fn note_on(&mut self, note: MidiNote, now: Duration) -> VoiceAction {
        let age = self.take_age();

        if let Some(index) = self.slots.iter().position(|slot| slot.note == Some(note)) {
            let slot = &mut self.slots[index];
            slot.started_at = now;
            slot.released_at = None;
            slot.release_duration = RELEASE;
            slot.age = age;
            slot.held = true;
            self.assert_invariants();
            return VoiceAction::Retriggered(index);
        }

        if let Some(index) = self.slots.iter().position(|slot| slot.note.is_none()) {
            self.start(index, note, now, age);
            self.assert_invariants();
            return VoiceAction::Started(index);
        }

        let index = self
            .slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.released_at.is_none())
            .min_by_key(|(_, slot)| slot.age)
            .or_else(|| {
                self.slots
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, slot)| slot.age)
            })
            .map(|(index, _)| index)
            .unwrap_or(0);
        let previous = self.slots[index].note.unwrap_or(note);
        self.start(index, note, now, age);
        self.assert_invariants();
        VoiceAction::Stolen {
            index,
            previous,
            fade: STEAL_FADE,
        }
    }

    pub fn note_off(&mut self, note: MidiNote, now: Duration) -> Option<usize> {
        let index = self
            .slots
            .iter()
            .position(|slot| slot.note == Some(note) && slot.held)?;
        self.begin_release(index, now, RELEASE);
        self.assert_invariants();
        Some(index)
    }

    pub fn defer_release(&mut self, note: MidiNote) -> Option<usize> {
        let index = self
            .slots
            .iter()
            .position(|slot| slot.note == Some(note) && slot.held)?;
        self.slots[index].held = false;
        self.assert_invariants();
        Some(index)
    }

    pub fn release_unheld(&mut self, now: Duration, duration: Duration) -> Vec<MidiNote> {
        let mut released = Vec::new();
        for index in 0..VOICE_COUNT {
            let slot = self.slots[index];
            if slot.note.is_some() && !slot.held && slot.released_at.is_none() {
                if let Some(note) = slot.note {
                    released.push(note);
                }
                self.begin_release(index, now, duration);
            }
        }
        self.assert_invariants();
        released
    }

    pub fn release_all(&mut self, now: Duration, duration: Duration) -> Vec<MidiNote> {
        let mut released = Vec::new();
        for index in 0..VOICE_COUNT {
            let slot = self.slots[index];
            if let Some(note) = slot.note {
                if slot.released_at.is_none() {
                    released.push(note);
                    self.begin_release(index, now, duration);
                }
            }
        }
        self.assert_invariants();
        released
    }

    pub fn expire(&mut self, now: Duration, timeout: Duration) -> Vec<MidiNote> {
        let mut expired = Vec::new();
        for index in 0..VOICE_COUNT {
            let slot = self.slots[index];
            if !slot.held {
                continue;
            }
            let Some(note) = slot.note else {
                continue;
            };
            if now.saturating_sub(slot.started_at) >= timeout {
                expired.push(note);
                self.begin_release(index, now, RELEASE);
            }
        }
        self.assert_invariants();
        expired
    }

    pub fn expire_releases(&mut self, now: Duration) -> Vec<MidiNote> {
        let mut finished = Vec::new();
        for index in 0..VOICE_COUNT {
            let slot = self.slots[index];
            let Some(released_at) = slot.released_at else {
                continue;
            };
            if now >= released_at.saturating_add(slot.release_duration) {
                if let Some(note) = slot.note {
                    finished.push(note);
                }
                self.slots[index] = EMPTY_SLOT;
            }
        }
        self.assert_invariants();
        finished
    }

    pub fn clear(&mut self) {
        self.slots = [EMPTY_SLOT; VOICE_COUNT];
    }

    pub fn is_held(&self, note: MidiNote) -> bool {
        self.slots
            .iter()
            .any(|slot| slot.note == Some(note) && slot.held)
    }

    pub fn is_releasing(&self, note: MidiNote) -> bool {
        self.slots
            .iter()
            .any(|slot| slot.note == Some(note) && slot.released_at.is_some())
    }

    pub fn held_notes(&self) -> impl Iterator<Item = MidiNote> + '_ {
        self.slots
            .iter()
            .filter(|slot| slot.held)
            .filter_map(|slot| slot.note)
    }

    pub fn active_notes(&self) -> impl Iterator<Item = MidiNote> + '_ {
        self.slots.iter().filter_map(|slot| slot.note)
    }

    pub fn held_count(&self) -> usize {
        self.slots.iter().filter(|slot| slot.held).count()
    }

    pub fn active_count(&self) -> usize {
        self.slots.iter().filter(|slot| slot.note.is_some()).count()
    }

    fn start(&mut self, index: usize, note: MidiNote, now: Duration, age: u64) {
        self.slots[index] = Slot {
            note: Some(note),
            started_at: now,
            released_at: None,
            release_duration: RELEASE,
            age,
            held: true,
        };
    }

    fn begin_release(&mut self, index: usize, now: Duration, duration: Duration) {
        let slot = &mut self.slots[index];
        slot.held = false;
        slot.released_at = Some(now);
        slot.release_duration = duration;
    }

    fn take_age(&mut self) -> u64 {
        let age = self.next_age;
        self.next_age = self.next_age.wrapping_add(1);
        age
    }

    fn assert_invariants(&self) {
        debug_assert!(self.active_count() <= VOICE_COUNT);
        let mut seen = [false; 128];
        for slot in &self.slots {
            let Some(note) = slot.note else {
                continue;
            };
            let midi = usize::from(note.midi());
            debug_assert!(!seen[midi], "duplicate voice for {note}");
            seen[midi] = true;
            debug_assert!(
                !slot.held || slot.released_at.is_none(),
                "held voice cannot be releasing"
            );
        }
    }
}

impl Default for VoiceAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::note::PitchClass;

    fn note(semitone: u8) -> MidiNote {
        MidiNote::from_pitch(PitchClass::from_semitone(semitone).unwrap(), 3).unwrap()
    }

    fn at(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            self.0
        }

        fn range(&mut self, limit: u64) -> u64 {
            self.next() % limit
        }
    }

    fn note_from_index(index: u64) -> MidiNote {
        MidiNote::from_midi(48 + u8::try_from(index).unwrap()).unwrap()
    }

    #[test]
    fn allocates_up_to_eight_voices() {
        let mut voices = VoiceAllocator::new();
        for semitone in 0..VOICE_COUNT as u8 {
            assert!(matches!(
                voices.note_on(note(semitone), at(0)),
                VoiceAction::Started(_)
            ));
        }
        assert_eq!(voices.active_count(), VOICE_COUNT);
        assert_eq!(voices.held_count(), VOICE_COUNT);
    }

    #[test]
    fn steals_oldest_held_voice_on_ninth_note() {
        let mut voices = VoiceAllocator::new();
        for semitone in 0..VOICE_COUNT as u8 {
            voices.note_on(note(semitone), at(u64::from(semitone)));
        }
        assert_eq!(
            voices.note_on(note(11), at(100)),
            VoiceAction::Stolen {
                index: 0,
                previous: note(0),
                fade: STEAL_FADE
            }
        );
        assert_eq!(voices.active_count(), VOICE_COUNT);
        assert!(voices.is_held(note(11)));
        assert!(!voices.is_held(note(0)));
    }

    #[test]
    fn retrigger_does_not_duplicate_voice() {
        let mut voices = VoiceAllocator::new();
        voices.note_on(note(0), at(0));
        assert_eq!(
            voices.note_on(note(0), at(100)),
            VoiceAction::Retriggered(0)
        );
        assert_eq!(voices.active_count(), 1);
        assert_eq!(voices.held_count(), 1);
    }

    #[test]
    fn releases_the_matching_voice() {
        let mut voices = VoiceAllocator::new();
        voices.note_on(note(0), at(0));
        voices.note_on(note(1), at(0));
        assert_eq!(voices.note_off(note(0), at(10)), Some(0));
        assert!(!voices.is_held(note(0)));
        assert!(voices.is_releasing(note(0)));
        assert!(voices.is_held(note(1)));
        assert_eq!(voices.note_off(note(0), at(20)), None);
    }

    #[test]
    fn expires_held_notes_and_finishes_their_release() {
        let mut voices = VoiceAllocator::new();
        voices.note_on(note(0), at(100));
        assert!(voices.expire(at(849), AUTO_RELEASE).is_empty());
        assert_eq!(voices.expire(at(850), AUTO_RELEASE), vec![note(0)]);
        assert_eq!(voices.held_count(), 0);
        assert!(voices.is_releasing(note(0)));
        assert!(voices.expire_releases(at(1_029)).is_empty());
        assert_eq!(voices.expire_releases(at(1_030)), vec![note(0)]);
        assert_eq!(voices.active_count(), 0);
    }

    #[test]
    fn retrigger_restarts_auto_release_timer() {
        let mut voices = VoiceAllocator::new();
        voices.note_on(note(0), at(0));
        voices.note_on(note(0), at(500));
        assert!(voices.expire(at(1000), AUTO_RELEASE).is_empty());
        assert_eq!(voices.expire(at(1250), AUTO_RELEASE), vec![note(0)]);
    }

    #[test]
    fn retrigger_cancels_a_pending_release() {
        let mut voices = VoiceAllocator::new();
        voices.note_on(note(0), at(0));
        voices.note_off(note(0), at(100));
        voices.note_on(note(0), at(150));
        assert!(!voices.is_releasing(note(0)));
        assert!(voices.expire_releases(at(1_000)).is_empty());
        assert!(voices.is_held(note(0)));
    }

    #[test]
    fn release_all_returns_notes_and_clear_frees_slots() {
        let mut voices = VoiceAllocator::new();
        voices.note_on(note(0), at(0));
        voices.note_on(note(1), at(0));
        assert_eq!(
            voices.release_all(at(10), SUSTAIN_RELEASE),
            vec![note(0), note(1)]
        );
        assert_eq!(voices.held_count(), 0);
        assert_eq!(voices.active_count(), 2);
        voices.clear();
        assert_eq!(voices.active_count(), 0);
    }

    #[test]
    fn deferred_release_waits_for_sustain_release() {
        let mut voices = VoiceAllocator::new();
        voices.note_on(note(0), at(0));
        voices.note_on(note(1), at(0));
        assert_eq!(voices.defer_release(note(0)), Some(0));
        assert!(!voices.is_held(note(0)));
        assert!(!voices.is_releasing(note(0)));
        assert!(voices.is_held(note(1)));

        assert_eq!(
            voices.release_unheld(at(100), SUSTAIN_RELEASE),
            vec![note(0)]
        );
        assert!(voices.is_releasing(note(0)));
        assert!(voices.is_held(note(1)));
        assert!(voices.expire_releases(at(549)).is_empty());
        assert_eq!(voices.expire_releases(at(550)), vec![note(0)]);
        assert_eq!(voices.held_count(), 1);
    }

    #[test]
    fn prefers_non_releasing_voice_before_releasing_ones() {
        let mut voices = VoiceAllocator::new();
        for semitone in 0..VOICE_COUNT as u8 {
            voices.note_on(note(semitone), at(0));
        }
        voices.note_off(note(3), at(10));
        let action = voices.note_on(note(11), at(100));
        assert_eq!(
            action,
            VoiceAction::Stolen {
                index: 0,
                previous: note(0),
                fade: STEAL_FADE
            }
        );
        assert!(!voices.is_held(note(3)));
        assert!(voices.is_releasing(note(3)));
    }

    #[test]
    fn steals_oldest_releasing_voice_when_all_are_releasing() {
        let mut voices = VoiceAllocator::new();
        for semitone in 0..VOICE_COUNT as u8 {
            voices.note_on(note(semitone), at(0));
            voices.note_off(note(semitone), at(10 * u64::from(semitone) + 20));
        }
        let action = voices.note_on(note(11), at(200));
        assert_eq!(
            action,
            VoiceAction::Stolen {
                index: 0,
                previous: note(0),
                fade: STEAL_FADE
            }
        );
    }

    #[test]
    fn invariants_hold_across_deterministic_sequences() {
        let mut voices = VoiceAllocator::new();
        let mut rng = Lcg(0x5eed);
        let mut now = 0u64;

        for _ in 0..20_000 {
            now += rng.range(17);
            let candidate = note_from_index(rng.range(24));
            match rng.range(6) {
                0..=2 => {
                    voices.note_on(candidate, at(now));
                }
                3 => {
                    voices.note_off(candidate, at(now));
                }
                4 => {
                    voices.defer_release(candidate);
                }
                _ => {
                    voices.expire(at(now), AUTO_RELEASE);
                    voices.expire_releases(at(now));
                }
            }

            let active: Vec<MidiNote> = voices.active_notes().collect();
            assert!(active.len() <= VOICE_COUNT);
            let mut unique = active.clone();
            unique.sort();
            unique.dedup();
            assert_eq!(unique.len(), active.len());
            assert!(voices.held_count() <= voices.active_count());
            for note in voices.held_notes() {
                assert!(!voices.is_releasing(note));
            }
        }
    }
}
