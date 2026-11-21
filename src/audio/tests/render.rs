//! The offline renderer: it plays named takes of the ship through the real mixer, with no audio
//! device and no window, and writes each one as a WAV file, so that someone can listen.
//!
//! It runs only when `SHIP_AUDIO_DIR` names the folder to write to:
//! `SHIP_AUDIO_DIR=/tmp/ship-audio cargo test --features dev --lib -- audio::tests::render`.

use std::io::{Result, Write};
use std::path::Path;

use super::rig::{alone, seated};
use super::takes::{flyby, thin, throttle, vacuum, warp};
use super::takes_ground::{canopy, gear, landing, liftoff, start};
use super::{RATE, frames, mixer};
use crate::audio::cue::AudioCue;
use crate::audio::dsp::limit::CEILING;
use crate::audio::ship::feed::HullSound;
use crate::audio::spatial::Spot;
use crate::settings::Audio as Volumes;

/// The state of the ship at each moment of a take, and the cues that fire in it.
type Script = (fn(f32) -> HullSound, &'static [(f32, AudioCue)]);

/// One take: its file name, its length in seconds, whether it is heard from the seat, and its script.
type Take = (&'static str, f32, bool, Script);

/// The moments of a warp: the drive starts to spool, engages and drops out.
const WARP: [(f32, AudioCue); 3] = [
    (1.0, AudioCue::WarpSpool),
    (5.0, AudioCue::WarpEngage),
    (10.0, AudioCue::WarpExit),
];

/// The touchdown of the landing take.
const TOUCH: [(f32, AudioCue); 1] = [(7.0, AudioCue::Land(0.45))];

/// Every take, in the order the files are numbered.
const TAKES: [Take; 16] = [
    ("01_cold_start_inside", 9.0, true, (start, &[])),
    ("02_cold_start_outside_20m", 9.0, false, (start, &[])),
    ("03_liftoff_inside", 13.0, true, (liftoff, &[])),
    ("04_liftoff_outside_20m", 13.0, false, (liftoff, &[])),
    ("05_throttle_thick_air_inside", 12.0, true, (throttle, &[])),
    (
        "06_throttle_thick_air_outside_60m",
        12.0,
        false,
        (throttle, &[]),
    ),
    ("07_flyby_150ms_doppler", 8.0, false, (flyby, &[])),
    ("08_thin_air_inside", 8.0, true, (thin, &[])),
    ("09_thin_air_outside_60m", 8.0, false, (thin, &[])),
    ("10_vacuum_inside", 10.0, true, (vacuum, &[])),
    ("11_vacuum_outside_silent", 4.0, false, (vacuum, &[])),
    ("12_gear_cycle_inside", 8.0, true, (gear, &[])),
    ("13_canopy_inside", 8.0, true, (canopy, &[])),
    ("14_warp_inside", 13.0, true, (warp, &WARP)),
    ("15_landing_inside", 12.0, true, (landing, &TOUCH)),
    ("16_landing_outside_20m", 12.0, false, (landing, &TOUCH)),
];

/// Frames between two frames of the game, which sends the state of the ship 60 times per second.
const GAME_FRAME: usize = 800;

/// Renders a take through the mixer, the state of the ship moving once per game frame as it
/// does in play.
fn record((_, seconds, seat, (at, cues)): &Take) -> Vec<[f32; 2]> {
    let scene = |t: f32| {
        let hull = at(t);
        let mut scene = alone(if *seat { seated(hull) } else { hull });
        scene.cabin = f32::from(u8::from(*seat));
        scene
    };
    let (mut mixer, shared) = mixer(&scene(0.0));
    let mut out = Vec::with_capacity(frames(*seconds));
    for i in 0..frames(*seconds) {
        let (t, next) = (i as f32 / RATE, (i + 1) as f32 / RATE);
        if i % GAME_FRAME == 0 {
            shared.publish(&scene(t), &Volumes::default());
        }
        for (_, cue) in cues.iter().filter(|(due, _)| (t..next).contains(due)) {
            shared.push(*cue, &Spot::FLAT);
        }
        out.push(mixer.frame());
    }
    out
}

/// Writes `sound` as a stereo WAV file of 16-bit samples at the synthesiser rate.
pub(super) fn write(path: &Path, sound: &[[f32; 2]]) -> Result<()> {
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
/// Every take stays within full scale, and only the take outside a ship in a vacuum is silent.
#[test]
fn ship_takes_render() {
    let Some(dir) = std::env::var_os("SHIP_AUDIO_DIR") else {
        return;
    };
    std::fs::create_dir_all(&dir).expect("the folder can be made");
    for take in &TAKES {
        let sound = record(take);
        let peak = sound.iter().flatten().fold(0.0_f32, |p, x| p.max(x.abs()));
        let silent = take.0.contains("silent");
        assert!(
            peak <= CEILING && (peak > 0.01) != silent,
            "{}: {peak}",
            take.0
        );
        let path = Path::new(&dir).join(format!("{}.wav", take.0));
        write(&path, &sound).expect("the file can be written");
        println!("{}: peak {peak:.3}", path.display());
    }
}
