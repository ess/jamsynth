pub mod dsp;
pub mod engine;
pub mod patch;
pub mod voice;

pub use engine::JamSynthEngine;
pub use patch::JamSynthPatch;

use nih_plug::prelude::*;
use std::sync::Arc;

#[derive(Params)]
pub struct JamSynthParams {
    #[id = "master_gain"]
    pub master_gain: FloatParam,

    #[id = "cutoff"]
    pub cutoff: FloatParam,

    #[id = "resonance"]
    pub resonance: FloatParam,

    #[id = "glide"]
    pub glide: FloatParam,

    #[id = "drive"]
    pub drive: FloatParam,

    #[id = "attack"]
    pub attack: FloatParam,

    #[id = "decay"]
    pub decay: FloatParam,

    #[id = "sustain"]
    pub sustain: FloatParam,

    #[id = "release"]
    pub release: FloatParam,
}

impl Default for JamSynthParams {
    fn default() -> Self {
        Self {
            master_gain: FloatParam::new(
                "Master Gain",
                0.9,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            cutoff: FloatParam::new(
                "Cutoff",
                350.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            ),
            resonance: FloatParam::new(
                "Resonance",
                0.2,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            glide: FloatParam::new(
                "Glide",
                45.0,
                FloatRange::Linear { min: 0.0, max: 1000.0 },
            ),
            drive: FloatParam::new(
                "Drive",
                0.35,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            attack: FloatParam::new(
                "Attack",
                1.0,
                FloatRange::Linear { min: 0.1, max: 5000.0 },
            ),
            decay: FloatParam::new(
                "Decay",
                1400.0,
                FloatRange::Linear { min: 1.0, max: 5000.0 },
            ),
            sustain: FloatParam::new(
                "Sustain",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            release: FloatParam::new(
                "Release",
                100.0,
                FloatRange::Linear { min: 1.0, max: 5000.0 },
            ),
        }
    }
}

pub struct JamSynth {
    params: Arc<JamSynthParams>,
    engine: JamSynthEngine,
}

impl Default for JamSynth {
    fn default() -> Self {
        Self {
            params: Arc::new(JamSynthParams::default()),
            engine: JamSynthEngine::new(44100.0),
        }
    }
}

impl Plugin for JamSynth {
    const NAME: &'static str = "JamSynth";
    const VENDOR: &'static str = "JamBand";
    const URL: &'static str = "https://codeberg.org/ess/jamband";
    const EMAIL: &'static str = "info@codeberg.org";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: std::num::NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.engine.set_sample_rate(buffer_config.sample_rate);
        true
    }

    fn reset(&mut self) {
        self.engine.all_notes_off();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Sync parameters
        self.engine.patch.master_gain = self.params.master_gain.value();
        self.engine.patch.filter.cutoff_hz = self.params.cutoff.value();
        self.engine.patch.filter.resonance = self.params.resonance.value();
        self.engine.patch.glide_ms = self.params.glide.value();
        self.engine.patch.drive.amount = self.params.drive.value();
        self.engine.patch.amp_env.attack_ms = self.params.attack.value();
        self.engine.patch.amp_env.decay_ms = self.params.decay.value();
        self.engine.patch.amp_env.sustain = self.params.sustain.value();
        self.engine.patch.amp_env.release_ms = self.params.release.value();

        // Process incoming MIDI events
        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOn { note, velocity, .. } => {
                    self.engine.note_on(note, velocity);
                }
                NoteEvent::NoteOff { note, .. } => {
                    self.engine.note_off(note);
                }
                NoteEvent::MidiChannelPressure { .. } => {}
                NoteEvent::MidiPitchBend { value, .. } => {
                    // value is 0.0 to 1.0 with 0.5 centered -> normalize to -1.0 to 1.0
                    let bend = (value - 0.5) * 2.0;
                    self.engine.set_pitch_bend(bend);
                }
                NoteEvent::MidiCC { cc, value, .. } => {
                    let val_u8 = (value * 127.0).round() as u8;
                    self.engine.handle_cc(cc, val_u8);
                }
                _ => {}
            }
        }

        // Process audio buffer
        let num_samples = buffer.samples();
        let mut left_scratch = vec![0.0f32; num_samples];
        let mut right_scratch = vec![0.0f32; num_samples];

        self.engine.process_block(&mut left_scratch, &mut right_scratch);

        let output_slices = buffer.as_slice();
        if output_slices.len() >= 2 {
            output_slices[0].copy_from_slice(&left_scratch);
            output_slices[1].copy_from_slice(&right_scratch);
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for JamSynth {
    const CLAP_ID: &'static str = "org.codeberg.ess.jamsynth";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Procedural JSON-native wavetable/sub/poly synthesizer for JamBand");
    const CLAP_MANUAL_URL: Option<&'static str> = Some("https://codeberg.org/ess/jamband");
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for JamSynth {
    const VST3_CLASS_ID: [u8; 16] = *b"JamSynth_JamBand";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}

nih_export_clap!(JamSynth);
nih_export_vst3!(JamSynth);
