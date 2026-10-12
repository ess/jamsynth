use crate::patch::{OscillatorConfig, Waveform};
use std::f32::consts::PI;

/// PolyBLEP residual function for bandlimited step transitions.
/// Corrects discontinuities within dt of the phase boundary.
#[inline(always)]
fn poly_blep(t: f32, dt: f32) -> f32 {
    if dt <= 0.0 {
        return 0.0;
    }
    // Discontinuity is at t = 0
    if t < dt {
        let t_norm = t / dt;
        t_norm + t_norm - t_norm * t_norm - 1.0
    } else if t > 1.0 - dt {
        let t_norm = (t - 1.0) / dt;
        t_norm * t_norm + t_norm + t_norm + 1.0
    } else {
        0.0
    }
}

pub struct PolyBlepOscillator {
    phase: f32,     // 0.0 to 1.0
    sub_phase: f32, // 0.0 to 1.0 (1 octave lower)
}

impl PolyBlepOscillator {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            sub_phase: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.sub_phase = 0.0;
    }

    /// Process one sample given the fundamental frequency, sample rate, and oscillator config.
    pub fn process(&mut self, base_freq: f32, sample_rate: f32, config: &OscillatorConfig) -> f32 {
        if !config.enabled || sample_rate <= 0.0 {
            return 0.0;
        }

        // Calculate pitch modified by octave, semitones, and fine cents:
        // freq = base_freq * 2^(octave + semitone/12 + cents/1200)
        let semitone_shift = (config.octave as f32 * 12.0)
            + (config.semitone as f32)
            + (config.fine_cents / 100.0);
        let freq = base_freq * (2.0f32).powf(semitone_shift / 12.0);

        let dt = (freq / sample_rate).clamp(0.0, 0.5);
        let sub_dt = (dt * 0.5).clamp(0.0, 0.5);

        // Compute raw oscillator sample based on waveform
        let osc_out = match config.wave {
            Waveform::Sine => (self.phase * 2.0 * PI).sin(),
            Waveform::Triangle => {
                // Derived from integrated square or triangle wave with polyblep
                let mut tri = -1.0 + (2.0 * self.phase);
                tri = 2.0 * (tri.abs() - 0.5);
                tri
            }
            Waveform::Saw => {
                // Naive saw: 2 * t - 1
                let mut saw = (2.0 * self.phase) - 1.0;
                saw -= poly_blep(self.phase, dt);
                saw
            }
            Waveform::Square => {
                // Naive square: 1 if phase < 0.5 else -1
                let mut sqr = if self.phase < 0.5 { 1.0 } else { -1.0 };
                sqr += poly_blep(self.phase, dt);
                sqr -= poly_blep((self.phase + 0.5).fract(), dt);
                sqr
            }
        };

        // Compute sub-oscillator sample (pure sine 1 octave down)
        let sub_out = if config.sub_osc_gain > 0.001 {
            (self.sub_phase * 2.0 * PI).sin() * config.sub_osc_gain
        } else {
            0.0
        };

        // Advance phase
        self.phase = (self.phase + dt).fract();
        self.sub_phase = (self.sub_phase + sub_dt).fract();

        (osc_out * config.gain) + sub_out
    }
}
