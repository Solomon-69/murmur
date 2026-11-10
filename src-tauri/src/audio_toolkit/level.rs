//! Brings quiet recordings up to a level the speech model can hear.
//!
//! Some microphones (and the built-in one at a low input volume) deliver speech
//! around -45 dBFS. Parakeet-style models return no tokens at that level, so the
//! dictation comes back empty. Raising the gain before decoding fixes that
//! without touching recordings that are already loud enough.

/// Peak level a quiet clip is raised to (about -3 dBFS).
const TARGET_PEAK: f32 = 0.7;
/// Clips whose peak is already at or above this are left alone.
const QUIET_PEAK: f32 = 0.35;
/// Largest boost applied, so near-silent noise is not blown up into hiss.
const MAX_GAIN: f32 = 30.0;

/// Returns `samples` scaled so its loudest point reaches `TARGET_PEAK`, when the
/// clip is quiet. Loud clips and pure silence come back unchanged.
pub fn level_quiet_audio(samples: Vec<f32>) -> Vec<f32> {
    let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if peak <= f32::EPSILON || peak >= QUIET_PEAK {
        return samples;
    }
    let gain = (TARGET_PEAK / peak).min(MAX_GAIN);
    log::debug!("audio level: peak {peak:.4}, applying gain {gain:.1}x before decoding");
    samples
        .into_iter()
        .map(|s| (s * gain).clamp(-1.0, 1.0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raises_quiet_clip_to_target_peak() {
        // Peak 0.2 needs 3.5x, under MAX_GAIN, so it lands exactly on target.
        // (A 0.02 peak would need 35x and is capped at 30x: see caps_gain_for_near_silence.)
        let out = level_quiet_audio(vec![0.1, -0.2, 0.15]);
        let peak = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!((peak - TARGET_PEAK).abs() < 1e-4);
    }

    #[test]
    fn leaves_loud_clip_unchanged() {
        let input = vec![0.5, -0.6, 0.1];
        assert_eq!(level_quiet_audio(input.clone()), input);
    }

    #[test]
    fn leaves_silence_unchanged() {
        assert_eq!(level_quiet_audio(vec![0.0; 4]), vec![0.0; 4]);
    }

    #[test]
    fn caps_gain_for_near_silence() {
        let out = level_quiet_audio(vec![0.001]);
        assert!((out[0] - 0.001 * MAX_GAIN).abs() < 1e-6);
    }
}
