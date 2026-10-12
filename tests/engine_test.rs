use jamsynth::patch::{JamSynthPatch, VoiceMode};
use jamsynth::JamSynthEngine;

#[test]
fn test_patch_json_roundtrip() {
    let patch = JamSynthPatch::preset_808_sub();
    let json = patch.to_json().expect("Failed to serialize to JSON");
    assert!(json.contains("Default 808 Sub"));
    assert!(json.contains("start_semitones"));

    let loaded = JamSynthPatch::from_json(&json).expect("Failed to deserialize from JSON");
    assert_eq!(patch.name, loaded.name);
    assert_eq!(patch.voice_mode, loaded.voice_mode);
    assert_eq!(patch.glide_ms, loaded.glide_ms);
}

#[test]
fn test_reese_and_pad_presets() {
    let reese = JamSynthPatch::preset_reese_bass();
    let pad = JamSynthPatch::preset_cold_pad();

    let reese_json = reese.to_json().unwrap();
    let pad_json = pad.to_json().unwrap();

    let loaded_reese = JamSynthPatch::from_json(&reese_json).unwrap();
    let loaded_pad = JamSynthPatch::from_json(&pad_json).unwrap();

    assert_eq!(loaded_reese.name, "Dark Reese Bass");
    assert_eq!(loaded_pad.name, "Cold Gothic Pad");
    assert!(matches!(loaded_pad.voice_mode, VoiceMode::Polyphonic { .. }));
}

#[test]
fn test_engine_renders_808_without_nan_or_inf() {
    let mut engine = JamSynthEngine::new(44100.0);
    engine.load_patch(JamSynthPatch::preset_808_sub());

    // Trigger D3 (MIDI note 50) at full velocity
    engine.note_on(50, 1.0);

    let mut left = vec![0.0f32; 1024];
    let mut right = vec![0.0f32; 1024];

    engine.process_block(&mut left, &mut right);

    // Verify non-zero output and no NaN/Inf
    let mut max_val = 0.0f32;
    for &sample in &left {
        assert!(!sample.is_nan(), "Output contained NaN!");
        assert!(!sample.is_infinite(), "Output contained Infinity!");
        if sample.abs() > max_val {
            max_val = sample.abs();
        }
    }

    assert!(max_val > 0.05, "Expected audio output from 808 note, got silence (max: {})", max_val);
}

#[test]
fn test_engine_portamento_808_slide() {
    let mut engine = JamSynthEngine::new(44100.0);
    let mut patch = JamSynthPatch::preset_808_sub();
    patch.glide_ms = 40.0;
    engine.load_patch(patch);

    // Trigger D3 (50)
    engine.note_on(50, 1.0);

    let mut left = vec![0.0f32; 512];
    let mut right = vec![0.0f32; 512];
    engine.process_block(&mut left, &mut right);

    // Slide up to D4 (62) - legato trigger
    engine.note_on(62, 1.0);
    engine.process_block(&mut left, &mut right);

    for &sample in &left {
        assert!(!sample.is_nan());
        assert!(!sample.is_infinite());
    }
}
