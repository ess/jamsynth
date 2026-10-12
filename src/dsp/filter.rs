use crate::patch::{FilterConfig, FilterType};
use std::f32::consts::PI;

pub struct MoogLadderFilter {
    // 4 cascaded 1-pole filter state variables
    stage: [f32; 4],
    delay: [f32; 4],
}

impl MoogLadderFilter {
    pub fn new() -> Self {
        Self {
            stage: [0.0; 4],
            delay: [0.0; 4],
        }
    }

    pub fn reset(&mut self) {
        self.stage = [0.0; 4];
        self.delay = [0.0; 4];
    }

    /// Process a single sample through a 24dB non-linear Moog Ladder model.
    #[inline(always)]
    pub fn process(&mut self, input: f32, cutoff_hz: f32, resonance: f32, sample_rate: f32) -> f32 {
        if sample_rate <= 0.0 {
            return input;
        }

        // Clamp cutoff to safe Nyquist boundary
        let cutoff = cutoff_hz.clamp(20.0, sample_rate * 0.45);
        let res = resonance.clamp(0.0, 0.98);

        // Tuning coefficient: g = 1 - exp(-2 * pi * cutoff / sample_rate)
        let wc = 2.0 * PI * cutoff / sample_rate;
        let g = 1.0 - (-wc).exp();
        let k = 4.0 * res; // Feedback resonance gain

        // Feedback loop with soft clipping to prevent harsh blowups
        let feedback = (self.delay[3] * k).tanh();
        let u = input - feedback;

        // 4 cascaded one-pole sections
        self.stage[0] = self.delay[0] + g * (u.tanh() - self.delay[0]);
        self.delay[0] = self.stage[0];

        self.stage[1] = self.delay[1] + g * (self.stage[0] - self.delay[1]);
        self.delay[1] = self.stage[1];

        self.stage[2] = self.delay[2] + g * (self.stage[1] - self.delay[2]);
        self.delay[2] = self.stage[2];

        self.stage[3] = self.delay[3] + g * (self.stage[2] - self.delay[3]);
        self.delay[3] = self.stage[3];

        self.stage[3]
    }
}

pub struct StateVariableFilter {
    ic1eq: f32,
    ic2eq: f32,
}

impl StateVariableFilter {
    pub fn new() -> Self {
        Self {
            ic1eq: 0.0,
            ic2eq: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    /// Process a sample through a 12dB 2-pole SVF (LowPass, BandPass, HighPass).
    #[inline(always)]
    pub fn process(
        &mut self,
        input: f32,
        cutoff_hz: f32,
        resonance: f32,
        filter_type: FilterType,
        sample_rate: f32,
    ) -> f32 {
        if sample_rate <= 0.0 {
            return input;
        }

        let cutoff = cutoff_hz.clamp(20.0, sample_rate * 0.45);
        // Q factor derived from resonance: 0.5 to ~25.0
        let q = 0.5 + (resonance * 15.0);

        let g = (PI * cutoff / sample_rate).tan();
        let k = 1.0 / q;
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;

        let v3 = input - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + a2 * self.ic1eq + a3 * v3;

        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        match filter_type {
            FilterType::LowPass12 | FilterType::LowPass24 => v2,
            FilterType::BandPass => v1,
            FilterType::HighPass => input - k * v1 - v2,
        }
    }
}

pub struct SynthFilter {
    ladder: MoogLadderFilter,
    svf: StateVariableFilter,
}

impl SynthFilter {
    pub fn new() -> Self {
        Self {
            ladder: MoogLadderFilter::new(),
            svf: StateVariableFilter::new(),
        }
    }

    pub fn reset(&mut self) {
        self.ladder.reset();
        self.svf.reset();
    }

    #[inline(always)]
    pub fn process(
        &mut self,
        input: f32,
        config: &FilterConfig,
        effective_cutoff_hz: f32,
        sample_rate: f32,
    ) -> f32 {
        if !config.enabled {
            return input;
        }

        match config.filter_type {
            FilterType::LowPass24 => self.ladder.process(
                input,
                effective_cutoff_hz,
                config.resonance,
                sample_rate,
            ),
            _ => self.svf.process(
                input,
                effective_cutoff_hz,
                config.resonance,
                config.filter_type,
                sample_rate,
            ),
        }
    }
}
