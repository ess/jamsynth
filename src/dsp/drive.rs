use crate::patch::{DriveConfig, DriveType};

pub struct DriveEffect;

impl DriveEffect {
    #[inline(always)]
    pub fn process(input: f32, config: &DriveConfig) -> f32 {
        if !config.enabled || config.amount <= 0.001 {
            return input;
        }

        let amt = config.amount.clamp(0.0, 1.0);

        match config.drive_type {
            DriveType::SoftSaturation => {
                // Pre-gain boost based on drive amount (1.0x to 6.0x)
                let gain = 1.0 + (amt * 5.0);
                let driven = input * gain;
                // Soft tanh saturation normalized so quiet parts aren't over-attenuated
                driven.tanh() / (1.0 + amt * 0.5)
            }
            DriveType::TapeWarmth => {
                // Subtle asymmetrical curve: introduces 2nd and 3rd harmonics
                let drive_in = input * (1.0 + amt * 3.0);
                let asym = drive_in + 0.15 * amt * drive_in * drive_in;
                asym.tanh()
            }
            DriveType::HardClip => {
                // Hard ceiling threshold
                let threshold = 1.0 - (amt * 0.7);
                input.clamp(-threshold, threshold) / threshold
            }
        }
    }
}
