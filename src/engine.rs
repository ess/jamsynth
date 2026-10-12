use crate::patch::{JamSynthPatch, VoiceMode};
use crate::voice::SynthVoice;

const MAX_VOICES: usize = 16;

pub struct JamSynthEngine {
    pub patch: JamSynthPatch,
    voices: Vec<SynthVoice>,
    sample_rate: f32,
    pitch_bend_semitones: f32,
    cc74_cutoff_mult: f32, // Controlled by CC 74 (0.25 to 2.5)
    sustain_pedal: bool,
    held_keys: Vec<(u8, f32)>, // Key stack for monophonic legato handling
}

impl JamSynthEngine {
    pub fn new(sample_rate: f32) -> Self {
        let mut voices = Vec::with_capacity(MAX_VOICES);
        for _ in 0..MAX_VOICES {
            voices.push(SynthVoice::new());
        }

        Self {
            patch: JamSynthPatch::default(),
            voices,
            sample_rate: if sample_rate > 0.0 { sample_rate } else { 44100.0 },
            pitch_bend_semitones: 0.0,
            cc74_cutoff_mult: 1.0,
            sustain_pedal: false,
            held_keys: Vec::new(),
        }
    }

    pub fn set_sample_rate(&mut self, rate: f32) {
        if rate > 0.0 {
            self.sample_rate = rate;
        }
    }

    pub fn load_patch(&mut self, patch: JamSynthPatch) {
        self.patch = patch;
    }

    pub fn load_patch_json(&mut self, json_str: &str) -> Result<(), serde_json::Error> {
        let patch = JamSynthPatch::from_json(json_str)?;
        self.load_patch(patch);
        Ok(())
    }

    pub fn all_notes_off(&mut self) {
        for voice in &mut self.voices {
            voice.reset();
        }
        self.held_keys.clear();
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        let vel = velocity.clamp(0.0, 1.0);
        self.held_keys.retain(|(k, _)| *k != note);
        self.held_keys.push((note, vel));

        match self.patch.voice_mode {
            VoiceMode::MonoLegato => {
                let v = &mut self.voices[0];
                let is_legato = v.is_active;
                v.note_on(note, vel, &self.patch, self.sample_rate, is_legato);
            }
            VoiceMode::MonoRetrigger => {
                let v = &mut self.voices[0];
                v.note_on(note, vel, &self.patch, self.sample_rate, false);
            }
            VoiceMode::Polyphonic { max_voices } => {
                let limit = max_voices.clamp(1, MAX_VOICES);
                // 1. Find free voice
                let mut target_idx = None;
                for i in 0..limit {
                    if !self.voices[i].is_active {
                        target_idx = Some(i);
                        break;
                    }
                }

                // 2. If no free voice, steal the oldest
                let idx = target_idx.unwrap_or_else(|| {
                    let mut oldest_idx = 0;
                    let mut oldest_age = 0;
                    for i in 0..limit {
                        if self.voices[i].age >= oldest_age {
                            oldest_age = self.voices[i].age;
                            oldest_idx = i;
                        }
                    }
                    oldest_idx
                });

                self.voices[idx].note_on(note, vel, &self.patch, self.sample_rate, false);
            }
        }
    }

    pub fn note_off(&mut self, note: u8) {
        self.held_keys.retain(|(k, _)| *k != note);

        match self.patch.voice_mode {
            VoiceMode::MonoLegato | VoiceMode::MonoRetrigger => {
                if let Some(&(prev_note, prev_vel)) = self.held_keys.last() {
                    // Legato glide back to the previous note in the stack
                    let v = &mut self.voices[0];
                    v.note_on(prev_note, prev_vel, &self.patch, self.sample_rate, true);
                } else if !self.sustain_pedal {
                    self.voices[0].note_off();
                }
            }
            VoiceMode::Polyphonic { .. } => {
                if !self.sustain_pedal {
                    for v in &mut self.voices {
                        if v.is_active && v.note == note {
                            v.note_off();
                        }
                    }
                }
            }
        }
    }

    pub fn set_pitch_bend(&mut self, bend_normalized: f32) {
        // Range: -1.0 to +1.0 -> maps to -2.0 to +2.0 semitones (or up to 12)
        self.pitch_bend_semitones = bend_normalized * 2.0;
    }

    pub fn handle_cc(&mut self, controller: u8, value: u8) {
        match controller {
            1 => {
                // Mod Wheel: could modulate LFO or vibrato
            }
            64 => {
                // Sustain pedal
                self.sustain_pedal = value >= 64;
                if !self.sustain_pedal && self.held_keys.is_empty() {
                    for v in &mut self.voices {
                        if v.is_active {
                            v.note_off();
                        }
                    }
                }
            }
            74 => {
                // Brightness / Cutoff frequency modulation:
                // Maps 0..127 to multiplier 0.25..2.5
                let norm = value as f32 / 127.0;
                self.cc74_cutoff_mult = 0.25 + (norm * 2.25);
            }
            120 | 123 => {
                // All Sound Off / All Notes Off
                self.all_notes_off();
            }
            _ => {}
        }
    }

    /// Process a block of stereo audio samples.
    pub fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        let num_samples = left.len().min(right.len());
        let active_voices = match self.patch.voice_mode {
            VoiceMode::MonoLegato | VoiceMode::MonoRetrigger => 1,
            VoiceMode::Polyphonic { max_voices } => max_voices.clamp(1, MAX_VOICES),
        };

        for i in 0..num_samples {
            let mut sample_out = 0.0;
            for v_idx in 0..active_voices {
                let v = &mut self.voices[v_idx];
                sample_out += v.process(&self.patch, self.sample_rate, self.cc74_cutoff_mult);
            }

            // Output mono synth to stereo channels
            left[i] = sample_out;
            right[i] = sample_out;
        }
    }
}
