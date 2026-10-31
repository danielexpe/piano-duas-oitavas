use std::f32::consts::TAU;

use crate::core::envelope::{Envelope, EnvelopeParams};
use crate::core::voice::{RELEASE, STEAL_FADE, SUSTAIN_RELEASE, VOICE_COUNT};

pub const SAMPLE_RATE: u32 = 48_000;
pub const MAX_VOICES: usize = VOICE_COUNT;
pub const SHUTDOWN_FADE: f32 = 0.120;

const HARMONICS: [f32; 5] = [1.0, 0.42, 0.22, 0.12, 0.07];
const INHARMONICITY: [f32; 5] = [1.0, 1.0005, 1.0012, 1.0021, 1.0035];
const MUTE_FADE: f32 = 0.005;

#[derive(Debug, Clone)]
struct Voice {
    note: u8,
    velocity: f32,
    phases: [f32; 5],
    frequencies: [f32; 5],
    envelope: Envelope,
}

impl Voice {
    fn new(note: u8, velocity: f32) -> Self {
        let base = frequency(note);
        let mut frequencies = [0.0; 5];
        for (frequency, ratio) in frequencies.iter_mut().zip(INHARMONICITY) {
            *frequency = base * ratio;
        }
        let mut envelope = Envelope::new(EnvelopeParams::DEFAULT);
        envelope.trigger();
        Self {
            note,
            velocity: velocity.clamp(0.0, 1.0),
            phases: [0.0; 5],
            frequencies,
            envelope,
        }
    }

    fn retrigger(&mut self, velocity: f32) {
        self.velocity = velocity.clamp(0.0, 1.0);
        self.phases = [0.0; 5];
        self.envelope.trigger();
    }

    fn release(&mut self, seconds: f32) {
        self.envelope.release(seconds);
    }

    fn sample(&mut self, dt: f32) -> f32 {
        let level = self.envelope.advance(dt);
        if level <= 0.0 {
            return 0.0;
        }
        let mut sample = 0.0;
        for (index, phase) in self.phases.iter_mut().enumerate() {
            sample += HARMONICS[index] * phase.sin();
            *phase += TAU * self.frequencies[index] * dt;
            if *phase >= TAU {
                *phase -= TAU;
            }
        }
        sample * level * self.velocity
    }

    fn is_active(&self) -> bool {
        self.envelope.is_active()
    }
}

#[derive(Debug, Clone)]
pub struct Synth {
    voices: Vec<Voice>,
    tails: Vec<Voice>,
    volume: f32,
    gain: f32,
    muted: bool,
    sample_rate: u32,
}

impl Synth {
    pub fn new(volume: f32, muted: bool) -> Self {
        let volume = volume.clamp(0.0, 1.0);
        Self {
            voices: Vec::with_capacity(MAX_VOICES),
            tails: Vec::with_capacity(MAX_VOICES),
            volume,
            gain: if muted { 0.0 } else { volume },
            muted,
            sample_rate: SAMPLE_RATE,
        }
    }

    pub fn with_sample_rate(volume: f32, muted: bool, sample_rate: u32) -> Self {
        let mut synth = Self::new(volume, muted);
        synth.sample_rate = sample_rate.max(1);
        synth
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        if let Some(voice) = self.voices.iter_mut().find(|voice| voice.note == note) {
            voice.retrigger(velocity);
            return;
        }

        if self.voices.len() >= MAX_VOICES {
            let mut stolen = self.voices.remove(0);
            stolen.release(STEAL_FADE.as_secs_f32());
            self.tails.push(stolen);
            if self.tails.len() > MAX_VOICES {
                self.tails.remove(0);
            }
        }
        self.voices.push(Voice::new(note, velocity));
    }

    pub fn note_off(&mut self, note: u8) {
        if let Some(voice) = self.voices.iter_mut().find(|voice| voice.note == note) {
            voice.release(RELEASE.as_secs_f32());
        }
    }

    pub fn all_notes_off(&mut self) {
        self.release_all(SUSTAIN_RELEASE.as_secs_f32());
    }

    pub fn release_all(&mut self, seconds: f32) {
        for voice in &mut self.voices {
            voice.release(seconds);
        }
        for tail in &mut self.tails {
            tail.release(seconds);
        }
    }

    pub fn shutdown(&mut self) {
        self.release_all(SHUTDOWN_FADE);
    }

    pub fn render(&mut self, output: &mut [f32]) {
        let dt = 1.0 / self.sample_rate as f32;
        let target = if self.muted { 0.0 } else { self.volume };
        let step = dt / MUTE_FADE;

        for sample in output.iter_mut() {
            if self.gain < target {
                self.gain = (self.gain + step).min(target);
            } else if self.gain > target {
                self.gain = (self.gain - step).max(target);
            }

            let mut mixed = 0.0;
            for voice in &mut self.voices {
                mixed += voice.sample(dt);
            }
            for tail in &mut self.tails {
                mixed += tail.sample(dt);
            }
            *sample = (mixed * self.gain).tanh();
        }

        self.voices.retain(Voice::is_active);
        self.tails.retain(Voice::is_active);
    }

    pub fn active_voices(&self) -> usize {
        self.voices.len()
    }

    pub fn tails(&self) -> usize {
        self.tails.len()
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }
}

fn frequency(note: u8) -> f32 {
    440.0 * 2f32.powf((f32::from(note) - 69.0) / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAMES: usize = 1024;

    fn render(synth: &mut Synth, seconds: f32) -> Vec<f32> {
        let frames = (seconds * synth.sample_rate as f32) as usize;
        let mut output = vec![0.0; frames];
        synth.render(&mut output);
        output
    }

    fn assert_bounded(samples: &[f32]) {
        for sample in samples {
            assert!(sample.is_finite(), "sample não finito: {sample}");
            assert!(sample.abs() <= 1.0, "sample fora da faixa: {sample}");
        }
    }

    #[test]
    fn renders_finite_bounded_audio() {
        let mut synth = Synth::new(0.32, false);
        synth.note_on(69, 1.0);
        let samples = render(&mut synth, 1.0);
        assert_bounded(&samples);
        assert!(samples.iter().any(|sample| sample.abs() > 0.001));
        assert_eq!(synth.active_voices(), 1);
    }

    #[test]
    fn silence_when_idle() {
        let mut synth = Synth::new(0.32, false);
        let samples = render(&mut synth, 0.1);
        assert!(samples.iter().all(|sample| *sample == 0.0));
    }

    #[test]
    fn key_up_releases_and_frees_the_voice() {
        let mut synth = Synth::new(0.32, false);
        synth.note_on(60, 1.0);
        render(&mut synth, 0.4);
        synth.note_off(60);
        let samples = render(&mut synth, 0.5);
        assert_bounded(&samples);
        assert_eq!(synth.active_voices(), 0);
        assert_eq!(synth.tails(), 0);
    }

    #[test]
    fn all_notes_off_fades_every_voice() {
        let mut synth = Synth::new(0.32, false);
        for note in 48..56 {
            synth.note_on(note, 1.0);
        }
        render(&mut synth, 0.2);
        synth.all_notes_off();
        render(&mut synth, 0.6);
        assert_eq!(synth.active_voices(), 0);
    }

    #[test]
    fn mute_ramps_down_to_silence_and_back() {
        let mut synth = Synth::new(0.32, false);
        synth.note_on(69, 1.0);
        render(&mut synth, 0.2);
        synth.set_muted(true);
        let muted = render(&mut synth, 0.1);
        assert_bounded(&muted);
        let tail: f32 = muted[muted.len() - 256..]
            .iter()
            .fold(0.0, |acc, sample| acc.max(sample.abs()));
        assert!(tail < 1e-4, "mudo ainda emite {tail}");
        synth.set_muted(false);
        let unmuted = render(&mut synth, 0.1);
        assert!(unmuted.iter().any(|sample| sample.abs() > 0.001));
    }

    #[test]
    fn steals_beyond_eight_voices_and_bounds_tails() {
        let mut synth = Synth::new(0.32, false);
        for note in 40..60 {
            synth.note_on(note, 1.0);
        }
        assert!(synth.active_voices() <= MAX_VOICES);
        assert!(synth.tails() <= MAX_VOICES);
        let samples = render(&mut synth, 0.2);
        assert_bounded(&samples);
        synth.release_all(0.01);
        render(&mut synth, 0.1);
        assert_eq!(synth.active_voices(), 0);
        assert_eq!(synth.tails(), 0);
    }

    #[test]
    fn retrigger_does_not_create_duplicate_voice() {
        let mut synth = Synth::new(0.32, false);
        synth.note_on(60, 1.0);
        synth.note_on(60, 0.5);
        assert_eq!(synth.active_voices(), 1);
    }

    #[test]
    fn shutdown_fade_ends_without_nan() {
        let mut synth = Synth::new(0.32, false);
        synth.note_on(48, 1.0);
        render(&mut synth, 0.1);
        synth.shutdown();
        let samples = render(&mut synth, 0.25);
        assert_bounded(&samples);
        assert_eq!(synth.active_voices(), 0);
    }

    #[test]
    fn render_chunking_matches_time() {
        let mut synth = Synth::with_sample_rate(0.32, false, SAMPLE_RATE);
        synth.note_on(69, 1.0);
        let first = render(&mut synth, 0.1);
        let second = render(&mut synth, 0.1);
        assert_bounded(&first);
        assert_bounded(&second);
        assert_eq!(first.len() + second.len(), FRAMES * 9 + 384);
    }
}
