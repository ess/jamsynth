use crate::patch::PitchEnvelopeConfig;

pub struct PitchEnvelope {
    current_semitones: f32,
    decay_rate: f32,
    active: bool,
}

impl PitchEnvelope {
    pub fn new() -> Self {
        Self {
            current_semitones: 0.0,
            decay_rate: 0.0,
            active: false,
        }
    }

    pub fn trigger(&mut self, config: &PitchEnvelopeConfig, sample_rate: f32) {
        if !config.enabled || config.decay_ms <= 0.5 || sample_rate <= 0.0 {
            self.current_semitones = 0.0;
            self.decay_rate = 0.0;
            self.active = false;
            return;
        }

        self.current_semitones = config.start_semitones;
        // Total decay samples:
        let total_samples = (config.decay_ms * 0.001 * sample_rate).max(1.0);
        // Exponential decay multiplier per sample (drops to ~0.001 at decay_ms):
        self.decay_rate = (-6.907755 / total_samples).exp(); // ln(0.001) = -6.907755
        self.active = true;
    }

    pub fn reset(&mut self) {
        self.current_semitones = 0.0;
        self.active = false;
    }

    #[inline(always)]
    pub fn process(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        let val = self.current_semitones;
        self.current_semitones *= self.decay_rate;
        if self.current_semitones.abs() < 0.01 {
            self.current_semitones = 0.0;
            self.active = false;
        }
        val
    }
}
