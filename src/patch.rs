use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VoiceMode {
    MonoLegato,
    MonoRetrigger,
    Polyphonic { max_voices: usize },
}

impl Default for VoiceMode {
    fn default() -> Self {
        Self::MonoLegato
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Waveform {
    Sine,
    Triangle,
    Saw,
    Square,
}

impl Default for Waveform {
    fn default() -> Self {
        Self::Sine
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OscillatorConfig {
    pub enabled: bool,
    pub wave: Waveform,
    pub octave: i32,        // e.g. -2, -1, 0, 1, 2
    pub semitone: i32,      // -12 to +12
    pub fine_cents: f32,    // -100 to +100
    pub gain: f32,          // 0.0 to 1.0
    pub sub_osc_gain: f32,  // Sine wave 1 octave down (0.0 to 1.0)
}

impl Default for OscillatorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            wave: Waveform::Sine,
            octave: -1,
            semitone: 0,
            fine_cents: 0.0,
            gain: 1.0,
            sub_osc_gain: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PitchEnvelopeConfig {
    pub enabled: bool,
    pub start_semitones: f32, // Pitch drop amount at note trigger (e.g. +24 semitones)
    pub decay_ms: f32,        // Drop duration in ms (e.g. 38ms for punchy 808)
}

impl Default for PitchEnvelopeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            start_semitones: 24.0,
            decay_ms: 38.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdsrConfig {
    pub attack_ms: f32,
    pub decay_ms: f32,
    pub sustain: f32, // 0.0 to 1.0
    pub release_ms: f32,
}

impl Default for AdsrConfig {
    fn default() -> Self {
        Self {
            attack_ms: 1.0,
            decay_ms: 1400.0,
            sustain: 0.0,
            release_ms: 100.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FilterType {
    LowPass24, // 24dB 4-pole Moog ladder
    LowPass12, // 12dB 2-pole SVF
    BandPass,
    HighPass,
}

impl Default for FilterType {
    fn default() -> Self {
        Self::LowPass24
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterConfig {
    pub enabled: bool,
    pub filter_type: FilterType,
    pub cutoff_hz: f32,       // 20.0 to 20000.0 Hz
    pub resonance: f32,       // 0.0 to 1.0 (Q factor / self-oscillation)
    pub key_tracking: f32,    // 0.0 to 1.0
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            filter_type: FilterType::LowPass24,
            cutoff_hz: 350.0,
            resonance: 0.2,
            key_tracking: 0.5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterEnvelopeConfig {
    pub enabled: bool,
    pub attack_ms: f32,
    pub decay_ms: f32,
    pub sustain: f32,
    pub release_ms: f32,
    pub amount_octaves: f32, // How many octaves envelope opens filter
}

impl Default for FilterEnvelopeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            attack_ms: 10.0,
            decay_ms: 300.0,
            sustain: 0.0,
            release_ms: 100.0,
            amount_octaves: 2.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DriveType {
    SoftSaturation,
    HardClip,
    TapeWarmth,
}

impl Default for DriveType {
    fn default() -> Self {
        Self::SoftSaturation
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriveConfig {
    pub enabled: bool,
    pub drive_type: DriveType,
    pub amount: f32, // 0.0 to 1.0
}

impl Default for DriveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            drive_type: DriveType::SoftSaturation,
            amount: 0.35,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JamSynthPatch {
    pub name: String,
    pub voice_mode: VoiceMode,
    pub glide_ms: f32, // Portamento glide speed in ms for 808 slides
    pub osc1: OscillatorConfig,
    pub osc2: OscillatorConfig,
    pub pitch_env: PitchEnvelopeConfig,
    pub amp_env: AdsrConfig,
    pub filter: FilterConfig,
    pub filter_env: FilterEnvelopeConfig,
    pub drive: DriveConfig,
    pub master_gain: f32,
}

impl Default for JamSynthPatch {
    fn default() -> Self {
        Self {
            name: "Default 808 Sub".to_string(),
            voice_mode: VoiceMode::MonoLegato,
            glide_ms: 45.0,
            osc1: OscillatorConfig::default(),
            osc2: OscillatorConfig {
                enabled: false,
                wave: Waveform::Saw,
                octave: 0,
                semitone: 0,
                fine_cents: 7.0,
                gain: 0.3,
                sub_osc_gain: 0.0,
            },
            pitch_env: PitchEnvelopeConfig::default(),
            amp_env: AdsrConfig::default(),
            filter: FilterConfig::default(),
            filter_env: FilterEnvelopeConfig::default(),
            drive: DriveConfig::default(),
            master_gain: 0.9,
        }
    }
}

impl JamSynthPatch {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    pub fn preset_808_sub() -> Self {
        Self::default()
    }

    pub fn preset_reese_bass() -> Self {
        Self {
            name: "Dark Reese Bass".to_string(),
            voice_mode: VoiceMode::MonoLegato,
            glide_ms: 60.0,
            osc1: OscillatorConfig {
                enabled: true,
                wave: Waveform::Saw,
                octave: -1,
                semitone: 0,
                fine_cents: -8.0,
                gain: 0.7,
                sub_osc_gain: 0.4,
            },
            osc2: OscillatorConfig {
                enabled: true,
                wave: Waveform::Saw,
                octave: -1,
                semitone: 0,
                fine_cents: 8.0,
                gain: 0.7,
                sub_osc_gain: 0.0,
            },
            pitch_env: PitchEnvelopeConfig {
                enabled: false,
                start_semitones: 0.0,
                decay_ms: 0.0,
            },
            amp_env: AdsrConfig {
                attack_ms: 15.0,
                decay_ms: 2000.0,
                sustain: 0.8,
                release_ms: 200.0,
            },
            filter: FilterConfig {
                enabled: true,
                filter_type: FilterType::LowPass24,
                cutoff_hz: 480.0,
                resonance: 0.3,
                key_tracking: 0.5,
            },
            filter_env: FilterEnvelopeConfig::default(),
            drive: DriveConfig {
                enabled: true,
                drive_type: DriveType::TapeWarmth,
                amount: 0.45,
            },
            master_gain: 0.85,
        }
    }

    pub fn preset_cold_pad() -> Self {
        Self {
            name: "Cold Gothic Pad".to_string(),
            voice_mode: VoiceMode::Polyphonic { max_voices: 8 },
            glide_ms: 0.0,
            osc1: OscillatorConfig {
                enabled: true,
                wave: Waveform::Saw,
                octave: 0,
                semitone: 0,
                fine_cents: -5.0,
                gain: 0.5,
                sub_osc_gain: 0.0,
            },
            osc2: OscillatorConfig {
                enabled: true,
                wave: Waveform::Saw,
                octave: 0,
                semitone: 7, // Fifth interval
                fine_cents: 5.0,
                gain: 0.4,
                sub_osc_gain: 0.0,
            },
            pitch_env: PitchEnvelopeConfig {
                enabled: false,
                start_semitones: 0.0,
                decay_ms: 0.0,
            },
            amp_env: AdsrConfig {
                attack_ms: 350.0,
                decay_ms: 3000.0,
                sustain: 0.7,
                release_ms: 1200.0,
            },
            filter: FilterConfig {
                enabled: true,
                filter_type: FilterType::LowPass24,
                cutoff_hz: 1200.0,
                resonance: 0.15,
                key_tracking: 0.8,
            },
            filter_env: FilterEnvelopeConfig {
                enabled: true,
                attack_ms: 500.0,
                decay_ms: 2500.0,
                sustain: 0.3,
                release_ms: 1000.0,
                amount_octaves: 1.5,
            },
            drive: DriveConfig {
                enabled: true,
                drive_type: DriveType::TapeWarmth,
                amount: 0.15,
            },
            master_gain: 0.8,
        }
    }
}
