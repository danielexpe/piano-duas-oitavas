use std::time::Duration;

use super::note::MidiNote;

pub const VOICE_COUNT: usize = 8;
pub const AUTO_RELEASE: Duration = Duration::from_millis(750);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceAction {
    Started(usize),
    Retriggered(usize),
    Stolen { index: usize, previous: MidiNote },
}

#[derive(Debug, Clone, Copy)]
struct Slot {
    note: Option<MidiNote>,
    started_at: Duration,
    age: u64,
    held: bool,
}

const EMPTY_SLOT: Slot = Slot {
    note: None,
    started_at: Duration::ZERO,
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
            slot.age = age;
            slot.held = true;
            return VoiceAction::Retriggered(index);
        }

        if let Some(index) = self.slots.iter().position(|slot| slot.note.is_none()) {
            self.start(index, note, now, age);
            return VoiceAction::Started(index);
        }

        if let Some(index) = self
            .slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| !slot.held)
            .min_by_key(|(_, slot)| slot.age)
            .map(|(index, _)| index)
        {
            self.start(index, note, now, age);
            return VoiceAction::Started(index);
        }

        let index = self
            .slots
            .iter()
            .enumerate()
            .min_by_key(|(_, slot)| slot.age)
            .map(|(index, _)| index)
            .unwrap_or(0);
        let previous = self.slots[index].note.unwrap_or(note);
        self.start(index, note, now, age);
        VoiceAction::Stolen { index, previous }
    }

    pub fn note_off(&mut self, note: MidiNote) -> Option<usize> {
        let index = self
            .slots
            .iter()
            .position(|slot| slot.note == Some(note) && slot.held)?;
        self.slots[index].held = false;
        Some(index)
    }

    pub fn expire(&mut self, now: Duration, timeout: Duration) -> Vec<MidiNote> {
        let mut expired = Vec::new();
        for slot in &mut self.slots {
            if slot.held && now.saturating_sub(slot.started_at) >= timeout {
                slot.held = false;
                if let Some(note) = slot.note {
                    expired.push(note);
                }
            }
        }
        expired
    }

    pub fn release_all(&mut self) {
        for slot in &mut self.slots {
            slot.held = false;
        }
    }

    pub fn clear(&mut self) {
        self.slots = [EMPTY_SLOT; VOICE_COUNT];
    }

    pub fn is_held(&self, note: MidiNote) -> bool {
        self.slots
            .iter()
            .any(|slot| slot.note == Some(note) && slot.held)
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
            age,
            held: true,
        };
    }

    fn take_age(&mut self) -> u64 {
        let age = self.next_age;
        self.next_age = self.next_age.wrapping_add(1);
        age
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
    fn steals_oldest_voice_on_ninth_note() {
        let mut voices = VoiceAllocator::new();
        for semitone in 0..VOICE_COUNT as u8 {
            voices.note_on(note(semitone), at(u64::from(semitone)));
        }
        assert_eq!(
            voices.note_on(note(11), at(100)),
            VoiceAction::Stolen {
                index: 0,
                previous: note(0)
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
        assert_eq!(voices.note_off(note(0)), Some(0));
        assert!(!voices.is_held(note(0)));
        assert!(voices.is_held(note(1)));
        assert_eq!(voices.note_off(note(0)), None);
    }

    #[test]
    fn expires_held_notes_after_timeout() {
        let mut voices = VoiceAllocator::new();
        voices.note_on(note(0), at(100));
        assert!(voices.expire(at(849), AUTO_RELEASE).is_empty());
        assert_eq!(voices.expire(at(850), AUTO_RELEASE), vec![note(0)]);
        assert_eq!(voices.held_count(), 0);
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
    fn release_all_stops_holds_and_clear_frees_slots() {
        let mut voices = VoiceAllocator::new();
        voices.note_on(note(0), at(0));
        voices.release_all();
        assert_eq!(voices.held_count(), 0);
        assert_eq!(voices.active_count(), 1);
        voices.clear();
        assert_eq!(voices.active_count(), 0);
    }

    #[test]
    fn reuses_released_voice_before_held_voice() {
        let mut voices = VoiceAllocator::new();
        for semitone in 0..VOICE_COUNT as u8 {
            voices.note_on(note(semitone), at(0));
        }
        voices.note_off(note(3));
        let action = voices.note_on(note(11), at(100));
        assert_eq!(action, VoiceAction::Started(3));
        assert!(voices.is_held(note(0)));
    }
}
