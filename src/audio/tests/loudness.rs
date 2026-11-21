//! The loudness of the whole mix, measured: every scene is rendered through the real mixer at
//! the default volumes and read by the meter. Its true peak stays under the ceiling and its
//! integrated loudness within the window of its target.
//!
//! `LOUDNESS_DIR=/tmp/abstractions-loudness cargo test --features dev --lib -- audio::tests::loudness --nocapture`
//! prints the table and writes each scene as a WAV file in that folder.

use std::path::Path;

use super::meter::{Loudness, measure};
use super::render::write;
use super::scenes::{SCENES, Scene};
use super::{RATE, frames, mixer};
use crate::audio::tune::{LOUDEST_MOMENT, TRUE_PEAK, WINDOW};
use crate::settings::{Audio as Volumes, Range};

/// Frames between two frames of the game.
const GAME_FRAME: usize = 800;

/// Seconds at the start of a scene the meter does not hear: the fade in of the output.
const LEAD: f32 = 1.0;

/// Renders a scene through the mixer at `volumes`, the scene moving once per game frame.
fn record((_, seconds, _, at, cues): &Scene, volumes: &Volumes) -> Vec<[f32; 2]> {
    let (mut mixer, shared) = mixer(&at(0.0));
    let total = frames(*seconds + LEAD);
    let mut sound = Vec::with_capacity(total);
    for i in 0..total {
        if i % GAME_FRAME == 0 {
            shared.publish(&at((i as f32 / RATE - LEAD).max(0.0)), volumes);
            if let Some(cue) = cues(i / GAME_FRAME) {
                shared.push(cue);
            }
        }
        sound.push(mixer.frame());
    }
    sound
}

/// Prints one row of the table.
fn row(name: &str, target: f32, read: &Loudness) {
    println!(
        "{name:<22} {target:>6.1} {:>7.1} {:>7.1} {:>7.1} {:>7.1} {:>6.1}",
        read.integrated, read.short_term, read.momentary, read.true_peak, read.range
    );
}

/// Renders and measures every scene at `volumes`, prints the table under `title`, writes the
/// sounds when `LOUDNESS_DIR` names a folder, and returns the readings in the order of the scenes.
fn table(title: &str, volumes: &Volumes) -> Vec<Loudness> {
    let dir = std::env::var_os("LOUDNESS_DIR");
    println!("{title}");
    println!("scene                  target   integ   short  moment    peak  range");
    let measured = |scene: &Scene| {
        let sound = record(scene, volumes);
        let read = measure(&sound[frames(LEAD)..]);
        row(scene.0, scene.2, &read);
        if let Some(dir) = &dir {
            std::fs::create_dir_all(dir).expect("the folder can be made");
            let path = Path::new(dir).join(format!("{title}_{}.wav", scene.0));
            write(&path, &sound).expect("the file can be written");
        }
        read
    };
    SCENES.iter().map(measured).collect()
}

/// At the default volumes every scene keeps its true peak under the ceiling, no moment is
/// louder than the loudest an event may be, and each scene sits within the window of its target.
#[test]
fn the_mix_meets_its_targets() {
    let reads = table("full", &Volumes::default());
    for (scene, read) in SCENES.iter().zip(&reads) {
        let name = scene.0;
        assert!(
            read.true_peak <= f64::from(TRUE_PEAK),
            "{name}: peak {}",
            read.true_peak
        );
        assert!(
            read.momentary <= f64::from(LOUDEST_MOMENT),
            "{name}: {}",
            read.momentary
        );
        let off = read.integrated - f64::from(scene.2);
        assert!(
            off.abs() <= f64::from(WINDOW),
            "{name}: {off:+.1} LU from its target"
        );
    }
}

/// The reduced dynamic range brings the scenes closer together: the gap between the quietest
/// and the loudest scene shrinks, and the true peak stays under the ceiling.
#[test]
fn the_reduced_range_is_narrower() {
    let reduced = Volumes {
        range: Range::Reduced,
        ..Volumes::default()
    };
    let spread = |reads: &[Loudness]| {
        let loud = reads
            .iter()
            .map(|read| read.integrated)
            .fold(f64::MIN, f64::max);
        let quiet = reads
            .iter()
            .map(|read| read.integrated)
            .fold(f64::MAX, f64::min);
        loud - quiet
    };
    let (full, narrow) = (
        table("full", &Volumes::default()),
        table("reduced", &reduced),
    );
    assert!(spread(&narrow) < spread(&full) - 2.0);
    assert!(
        narrow
            .iter()
            .all(|read| read.true_peak <= f64::from(TRUE_PEAK))
    );
}
