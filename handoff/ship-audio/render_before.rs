//! The ship sound of `staging` before the 0.11 voice, rendered to WAV files for comparison.
//!
//! This file is not part of the game. Copy it to `src/audio/tests/render_before.rs` in a
//! checkout of the commit before the new voice, add `mod render_before;` to
//! `src/audio/tests.rs`, and run it with `SHIP_AUDIO_DIR` set:
//! `SHIP_AUDIO_DIR=/tmp/ship-audio/before cargo test --features dev --lib -- audio::tests::render_before`.
//!
//! The old voice had three inputs only, the throttle, the dynamic pressure and the cabin, so
//! there are four takes. Outside the cabin the old engine was silent.

use std::io::{Result, Write};
use std::path::Path;

use super::{RATE, frames, mixer};
use crate::audio::scene::AudioScene;
use crate::settings::Audio as Volumes;

/// Returns how far `t` is from `from` to `to`, held between 0 and 1.
fn ramp(t: f32, from: f32, to: f32) -> f32 {
    ((t - from) / (to - from)).clamp(0.0, 1.0)
}

/// Returns the scene of the old voice in the cabin at a throttle and a dynamic pressure, each
/// from 0 to 1, with no music.
fn cabin(throttle: f32, pressure: f32, in_space: bool) -> AudioScene {
    AudioScene {
        altitude_fade: 0.0,
        cabin: 1.0,
        throttle,
        dynamic_pressure: pressure,
        in_space,
        ..AudioScene::default()
    }
}

/// The lever of the throttle take of the new renderer: idle, full from 2 to 4 seconds, back to
/// idle from 9.5 to 11.
fn lever(t: f32) -> f32 {
    ramp(t, 2.0, 4.0) - ramp(t, 9.5, 11.0)
}

/// One take: its file name, its length in seconds and the scene at each moment.
type Take = (&'static str, f32, fn(f32) -> AudioScene);

/// Every take of the old voice.
const TAKES: [Take; 4] = [
    ("before_01_idle_inside", 9.0, |_| cabin(0.0, 0.0, false)),
    ("before_05_throttle_inside", 12.0, |t| {
        cabin(lever(t), 0.3 * lever(t), false)
    }),
    ("before_10_vacuum_inside", 10.0, |t| {
        cabin(ramp(t, 2.0, 3.5) - ramp(t, 6.5, 8.0), 0.0, true)
    }),
    ("before_06_throttle_outside", 12.0, |t| AudioScene {
        cabin: 0.0,
        ..cabin(lever(t), 0.3 * lever(t), false)
    }),
];

/// Writes `sound` as a stereo WAV file of 16-bit samples at the synthesiser rate.
fn write(path: &Path, sound: &[[f32; 2]]) -> Result<()> {
    let bytes = (sound.len() * 4) as u32;
    let rate = RATE as u32;
    let mut file = std::io::BufWriter::new(std::fs::File::create(path)?);
    file.write_all(b"RIFF")?;
    file.write_all(&(36 + bytes).to_le_bytes())?;
    file.write_all(b"WAVEfmt ")?;
    file.write_all(&16_u32.to_le_bytes())?;
    file.write_all(&1_u16.to_le_bytes())?;
    file.write_all(&2_u16.to_le_bytes())?;
    file.write_all(&rate.to_le_bytes())?;
    file.write_all(&(rate * 4).to_le_bytes())?;
    file.write_all(&4_u16.to_le_bytes())?;
    file.write_all(&16_u16.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&bytes.to_le_bytes())?;
    for sample in sound.iter().flatten() {
        let word = (sample.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        file.write_all(&word.to_le_bytes())?;
    }
    file.flush()
}

/// Renders every take into the folder `SHIP_AUDIO_DIR` names, and does nothing without it.
#[test]
fn old_ship_takes_render() {
    let Some(dir) = std::env::var_os("SHIP_AUDIO_DIR") else {
        return;
    };
    std::fs::create_dir_all(&dir).expect("the folder can be made");
    for (name, seconds, at) in &TAKES {
        let (mut mixer, shared) = mixer(&at(0.0));
        let mut sound = Vec::with_capacity(frames(*seconds));
        for i in 0..frames(*seconds) {
            if i % 800 == 0 {
                shared.publish(&at(i as f32 / RATE), &Volumes::default());
            }
            sound.push(mixer.frame());
        }
        let path = Path::new(&dir).join(format!("{name}.wav"));
        write(&path, &sound).expect("the file can be written");
        println!("{}", path.display());
    }
}
