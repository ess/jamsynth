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
                "Filter Cutoff",
                350.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
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
