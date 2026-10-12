use crate::dsp::drive::DriveEffect;
use crate::dsp::env::AdsrEnvelope;
use crate::dsp::filter::SynthFilter;
use crate::dsp::osc::PolyBlepOscillator;
use crate::dsp::pitch_env::PitchEnvelope;
use crate::patch::JamSynthPatch;

pub struct SynthVoice {
    pub osc1: PolyBlepOscillator,
    pub osc2: PolyBlepOscillator,
    pub pitch_env: PitchEnvelope,
    pub amp_env: AdsrEnvelope,
    pub filter_env: AdsrEnvelope,
    pub filter: SynthFilter,

    pub note: u8,
    pub velocity: f32,
    pub current_freq: f32,
    pub target_freq: f32,
    pub glide_coeff: f32,
    pub is_active: bool,
    pub age: u64, // For voice stealing in polyphonic mode
}

impl SynthVoice {
    pub fn new() -> Self {
        Self {
            osc1: PolyBlepOscillator::new(),
            osc2: PolyBlepOscillator::new(),
            pitch_env: PitchEnvelope::new(),
            amp_env: AdsrEnvelope::new(),
            filter_env: AdsrEnvelope::new(),
            filter: SynthFilter::new(),
            note: 0,
            velocity: 0.0,
            current_freq: 440.0,
            target_freq: 440.0,
            glide_coeff: 1.0,
            is_active: false,
            age: 0,
        }
    }

    pub fn note_to_freq(note: u8) -> f32 {
        440.0 * (2.0f32).powf((note as f32 - 69.0) / 12.0)
    }

    /// Trigger a new note. If legato is true, glide from the current pitch without resetting envelopes.
    pub fn note_on(
        &mut self,
        note: u8,
        velocity: f32,
        patch: &JamSynthPatch,
        sample_rate: f32,
        legato: bool,
    ) {
        self.note = note;
        self.velocity = velocity;
        self.target_freq = Self::note_to_freq(note);

        if !legato {
            // Instant pitch jump on fresh note
            self.current_freq = self.target_freq;
            self.osc1.reset();
            self.osc2.reset();
            self.pitch_env.trigger(&patch.pitch_env, sample_rate);
            self.amp_env.trigger(&patch.amp_env, sample_rate);
            if patch.filter_env.enabled {
                self.filter_env.trigger(
                    &crate::patch::AdsrConfig {
                        attack_ms: patch.filter_env.attack_ms,
                        decay_ms: patch.filter_env.decay_ms,
                        sustain: patch.filter_env.sustain,
                        release_ms: patch.filter_env.release_ms,
                    },
                    sample_rate,
                );
            }
        } else {
            // Smooth portamento glide without re-triggering envelopes
            if patch.glide_ms > 0.5 && sample_rate > 0.0 {
                let glide_samples = (patch.glide_ms * 0.001 * sample_rate).max(1.0);
                self.glide_coeff = (-4.0 / glide_samples).exp();
            } else {
                self.current_freq = self.target_freq;
                self.glide_coeff = 0.0;
            }
        }

        self.is_active = true;
    }

    pub fn note_off(&mut self) {
        self.amp_env.release();
        self.filter_env.release();
    }

    pub fn reset(&mut self) {
        self.osc1.reset();
        self.osc2.reset();
        self.pitch_env.reset();
        self.amp_env.reset();
        self.filter_env.reset();
        self.filter.reset();
        self.is_active = false;
        self.velocity = 0.0;
    }

    /// Render one sample from this voice.
    #[inline(always)]
    pub fn process(
        &mut self,
        patch: &JamSynthPatch,
        sample_rate: f32,
        cc74_cutoff_mod: f32, // CC 74 offset multiplier (0.5 to 2.0)
    ) -> f32 {
        if !self.is_active && !self.amp_env.is_active() {
            self.is_active = false;
            return 0.0;
        }

        self.age = self.age.wrapping_add(1);

        // 1. Portamento glide towards target frequency
        if (self.current_freq - self.target_freq).abs() > 0.05 {
            self.current_freq =
                self.target_freq + (self.current_freq - self.target_freq) * self.glide_coeff;
        } else {
            self.current_freq = self.target_freq;
        }

        // 2. Pitch envelope modulation (for 808 transient drops)
        let pitch_mod_semitones = self.pitch_env.process();
        let effective_freq = if pitch_mod_semitones.abs() > 0.01 {
            self.current_freq * (2.0f32).powf(pitch_mod_semitones / 12.0)
        } else {
            self.current_freq
        };

        // 3. Process oscillators
        let s1 = self.osc1.process(effective_freq, sample_rate, &patch.osc1);
        let s2 = self.osc2.process(effective_freq, sample_rate, &patch.osc2);
        let mut audio = s1 + s2;

        // 4. Calculate effective filter cutoff
        let mut cutoff = patch.filter.cutoff_hz;
        if patch.filter.key_tracking > 0.01 {
            let note_diff = self.note as f32 - 60.0;
            cutoff *= (2.0f32).powf(note_diff * patch.filter.key_tracking / 12.0);
        }

        // Apply filter envelope modulation
        if patch.filter_env.enabled {
            let env_val = self.filter_env.process();
            cutoff *= (2.0f32).powf(env_val * patch.filter_env.amount_octaves);
        }

        // Apply real-time MIDI CC 74 cutoff offset
        cutoff *= cc74_cutoff_mod;

        // 5. Filter processing
        audio = self.filter.process(audio, &patch.filter, cutoff, sample_rate);

        // 6. Drive & saturation
        audio = DriveEffect::process(audio, &patch.drive);

        // 7. Amp envelope & velocity scaling
        let amp = self.amp_env.process() * self.velocity;
        if !self.amp_env.is_active() {
            self.is_active = false;
        }

        audio * amp * patch.master_gain
    }
}
