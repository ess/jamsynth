use crate::patch::AdsrConfig;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct AdsrEnvelope {
    state: EnvelopeState,
    current_value: f32,
    attack_rate: f32,
    decay_rate: f32,
    release_rate: f32,
    sustain_level: f32,
}

impl AdsrEnvelope {
    pub fn new() -> Self {
        Self {
            state: EnvelopeState::Idle,
            current_value: 0.0,
            attack_rate: 0.0,
            decay_rate: 0.0,
            release_rate: 0.0,
            sustain_level: 0.0,
        }
    }

    pub fn is_active(&self) -> bool {
        self.state != EnvelopeState::Idle
    }

    pub fn trigger(&mut self, config: &AdsrConfig, sample_rate: f32) {
        if sample_rate <= 0.0 {
            return;
        }

        self.sustain_level = config.sustain.clamp(0.0, 1.0);

        let attack_samples = (config.attack_ms.max(0.5) * 0.001 * sample_rate).max(1.0);
        let decay_samples = (config.decay_ms.max(0.5) * 0.001 * sample_rate).max(1.0);
        let release_samples = (config.release_ms.max(0.5) * 0.001 * sample_rate).max(1.0);

        // Linear step per sample for attack
        self.attack_rate = 1.0 / attack_samples;
        // Exponential decay coefficient
        self.decay_rate = (-3.0 / decay_samples).exp();
        // Exponential release coefficient
        self.release_rate = (-4.0 / release_samples).exp();

        self.state = EnvelopeState::Attack;
    }

    pub fn release(&mut self) {
        if self.state != EnvelopeState::Idle {
            self.state = EnvelopeState::Release;
        }
    }

    pub fn reset(&mut self) {
        self.state = EnvelopeState::Idle;
        self.current_value = 0.0;
    }

    #[inline(always)]
    pub fn process(&mut self) -> f32 {
        match self.state {
            EnvelopeState::Idle => 0.0,
            EnvelopeState::Attack => {
                self.current_value += self.attack_rate;
                if self.current_value >= 1.0 {
                    self.current_value = 1.0;
                    self.state = EnvelopeState::Decay;
                }
                self.current_value
            }
            EnvelopeState::Decay => {
                // Decay exponentially towards sustain_level
                self.current_value =
                    self.sustain_level + (self.current_value - self.sustain_level) * self.decay_rate;
                if (self.current_value - self.sustain_level).abs() < 0.001 {
                    self.current_value = self.sustain_level;
                    self.state = EnvelopeState::Sustain;
                }
                self.current_value
            }
            EnvelopeState::Sustain => self.sustain_level,
            EnvelopeState::Release => {
                self.current_value *= self.release_rate;
                if self.current_value < 0.0001 {
                    self.current_value = 0.0;
                    self.state = EnvelopeState::Idle;
                }
                self.current_value
            }
        }
    }
}
