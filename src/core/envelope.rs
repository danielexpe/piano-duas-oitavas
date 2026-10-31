#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnvelopeParams {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl EnvelopeParams {
    pub const DEFAULT: Self = Self {
        attack: 0.008,
        decay: 0.350,
        sustain: 0.25,
        release: 0.180,
    };

    pub const SUSTAIN_RELEASE: f32 = 0.450;
}

impl Default for EnvelopeParams {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone, Copy)]
pub struct Envelope {
    params: EnvelopeParams,
    stage: Stage,
    level: f32,
    elapsed: f32,
    release_level: f32,
    release_duration: f32,
}

impl Envelope {
    pub fn new(params: EnvelopeParams) -> Self {
        Self {
            params,
            stage: Stage::Idle,
            level: 0.0,
            elapsed: 0.0,
            release_level: 0.0,
            release_duration: params.release,
        }
    }

    pub fn trigger(&mut self) {
        self.stage = Stage::Attack;
        self.level = 0.0;
        self.elapsed = 0.0;
        self.release_level = 0.0;
        self.release_duration = self.params.release;
    }

    pub fn release(&mut self, duration: f32) {
        if self.stage == Stage::Idle {
            return;
        }
        self.release_level = self.level;
        self.release_duration = duration.max(0.001);
        self.elapsed = 0.0;
        self.stage = Stage::Release;
    }

    pub fn advance(&mut self, mut dt: f32) -> f32 {
        if !dt.is_finite() || dt <= 0.0 || self.stage == Stage::Idle {
            return self.level;
        }

        while dt > 0.0 {
            let duration = match self.stage {
                Stage::Attack => self.params.attack,
                Stage::Decay => self.params.decay,
                Stage::Sustain => return self.params.sustain,
                Stage::Release => self.release_duration,
                Stage::Idle => return 0.0,
            }
            .max(0.001);

            let step = dt.min(duration - self.elapsed);
            self.elapsed += step;
            dt -= step;

            self.level = match self.stage {
                Stage::Attack => (self.elapsed / duration).clamp(0.0, 1.0),
                Stage::Decay => {
                    let drop = 1.0 - self.params.sustain;
                    (1.0 - drop * (self.elapsed / duration)).clamp(0.0, 1.0)
                }
                Stage::Sustain => self.params.sustain,
                Stage::Release => {
                    (self.release_level * (1.0 - self.elapsed / duration)).clamp(0.0, 1.0)
                }
                Stage::Idle => 0.0,
            };

            if self.elapsed + 1e-6 < duration {
                break;
            }

            match self.stage {
                Stage::Attack => {
                    self.stage = Stage::Decay;
                    self.level = 1.0;
                }
                Stage::Decay => {
                    self.stage = Stage::Sustain;
                    self.level = self.params.sustain;
                }
                Stage::Release => {
                    self.stage = Stage::Idle;
                    self.level = 0.0;
                    return 0.0;
                }
                Stage::Sustain | Stage::Idle => {}
            }
            self.elapsed = 0.0;
        }

        self.level
    }

    pub fn level(&self) -> f32 {
        self.level
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn is_active(&self) -> bool {
        self.stage != Stage::Idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STEP: f32 = 1.0 / 48_000.0;

    fn run(envelope: &mut Envelope, seconds: f32) -> f32 {
        let mut elapsed = 0.0;
        let mut level = envelope.level();
        while elapsed < seconds {
            level = envelope.advance(STEP);
            elapsed += STEP;
        }
        level
    }

    #[test]
    fn follows_attack_and_decay_to_sustain() {
        let mut envelope = Envelope::new(EnvelopeParams::DEFAULT);
        envelope.trigger();
        let peak = run(&mut envelope, 0.008);
        assert!(peak > 0.99, "peak was {peak}");
        let sustained = run(&mut envelope, 0.350);
        assert!((sustained - 0.25).abs() < 1e-3, "level was {sustained}");
        assert_eq!(envelope.stage(), Stage::Sustain);
    }

    #[test]
    fn keeps_sustain_level_while_held() {
        let mut envelope = Envelope::new(EnvelopeParams::DEFAULT);
        envelope.trigger();
        run(&mut envelope, 1.0);
        assert!((envelope.advance(1.0) - 0.25).abs() < 1e-6);
        assert_eq!(envelope.stage(), Stage::Sustain);
    }

    #[test]
    fn releases_in_180_milliseconds() {
        let mut envelope = Envelope::new(EnvelopeParams::DEFAULT);
        envelope.trigger();
        run(&mut envelope, 0.400);
        envelope.release(EnvelopeParams::DEFAULT.release);
        let level = run(&mut envelope, 0.181);
        assert!(level.abs() < 1e-3, "level was {level}");
        assert_eq!(envelope.stage(), Stage::Idle);
        assert!(!envelope.is_active());
    }

    #[test]
    fn releases_sustain_in_450_milliseconds() {
        let mut envelope = Envelope::new(EnvelopeParams::DEFAULT);
        envelope.trigger();
        run(&mut envelope, 0.400);
        envelope.release(EnvelopeParams::SUSTAIN_RELEASE);
        let level = run(&mut envelope, 0.449);
        assert!(level > 0.0);
        let level = run(&mut envelope, 0.002);
        assert!(level.abs() < 1e-3, "level was {level}");
        assert_eq!(envelope.stage(), Stage::Idle);
    }

    #[test]
    fn stays_finite_and_bounded() {
        let mut envelope = Envelope::new(EnvelopeParams::DEFAULT);
        envelope.trigger();
        let mut peak: f32 = 0.0;
        for _ in 0..48_000 {
            let level = envelope.advance(STEP);
            assert!(level.is_finite());
            assert!((0.0..=1.0).contains(&level));
            peak = peak.max(level);
        }
        assert!(peak <= 1.0);
        envelope.release(EnvelopeParams::SUSTAIN_RELEASE);
        for _ in 0..48_000 {
            let level = envelope.advance(STEP);
            assert!(level.is_finite());
            assert!((0.0..=1.0).contains(&level));
        }
    }

    #[test]
    fn retrigger_restarts_from_zero() {
        let mut envelope = Envelope::new(EnvelopeParams::DEFAULT);
        envelope.trigger();
        run(&mut envelope, 0.1);
        envelope.trigger();
        assert_eq!(envelope.stage(), Stage::Attack);
        assert!(envelope.level() < 0.01);
    }

    #[test]
    fn ignores_inactive_release() {
        let mut envelope = Envelope::new(EnvelopeParams::DEFAULT);
        envelope.release(0.1);
        assert_eq!(envelope.stage(), Stage::Idle);
        assert_eq!(envelope.advance(0.1), 0.0);
    }
}
