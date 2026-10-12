use jamsynth::patch::JamSynthPatch;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = Path::new("presets");
    fs::create_dir_all(out_dir).expect("Failed to create presets directory");

    let p808 = JamSynthPatch::preset_808_sub();
    let preese = JamSynthPatch::preset_reese_bass();
    let ppad = JamSynthPatch::preset_cold_pad();

    fs::write(
        out_dir.join("808_sub.json"),
        p808.to_json().unwrap(),
    ).expect("Failed to write 808_sub.json");

    fs::write(
        out_dir.join("dark_reese.json"),
        preese.to_json().unwrap(),
    ).expect("Failed to write dark_reese.json");

    fs::write(
        out_dir.join("cold_gothic_pad.json"),
        ppad.to_json().unwrap(),
    ).expect("Failed to write cold_gothic_pad.json");

    println!("✓ Successfully exported default JSON presets to jamsynth/presets/:");
    println!("  • 808_sub.json");
    println!("  • dark_reese.json");
    println!("  • cold_gothic_pad.json");
}
