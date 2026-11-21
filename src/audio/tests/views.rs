//! The seat and the chase view: what the ship sounds like depends on where the ears are. Moving
//! between them fades over about 0.3 seconds with no click, in a vacuum the chase view hears no
//! engine at all, and the seat still hears it through the frame.

use super::rig::{alone, parked, rms, seated};
use super::{RATE, frames, mixer};
use crate::audio::scene::AudioScene;
use crate::audio::ship::feed::{Ch, HullSound};
use crate::settings::Audio as Volumes;

/// Frames between two frames of the game.
const GAME_FRAME: usize = 800;

/// Returns the ship of this machine at full burn heard from a chase view 12 metres behind it.
fn chase(hull: HullSound) -> HullSound {
    hull.with(Ch::Riding, 1.0)
        .with(Ch::Distance, 12.0)
        .with(Ch::Aspect, 0.9)
        .with(Ch::Speed, 90.0)
}

/// Returns the scene of `hull` alone, with the ears in the cabin when `cabin`.
fn scene(hull: HullSound, cabin: bool) -> AudioScene {
    AudioScene {
        cabin: f32::from(u8::from(cabin)),
        ..alone(hull)
    }
}

/// Renders four seconds of `first`, then two of `then`, the scene moving once per game frame,
/// and returns the left channel.
fn switched(first: AudioScene, then: AudioScene) -> Vec<f32> {
    let (mut mixer, shared) = mixer(&first);
    (0..frames(6.0))
        .map(|i| {
            if i == frames(4.0) / GAME_FRAME * GAME_FRAME {
                shared.publish(&then, &Volumes::default());
            }
            mixer.frame()[0]
        })
        .collect()
}

/// Returns the root mean square of `sound` over the 25 milliseconds from `seconds`.
fn at(sound: &[f32], seconds: f32) -> f64 {
    let start = (seconds * RATE) as usize;
    rms(&sound[start..start + frames(0.025)])
}

/// Getting into the seat or out to the chase view fades the sound: 25 milliseconds after the
/// switch it is still near where it was, by half a second it is where it goes, and no sample
/// jumps further than the burn does on its own.
#[test]
fn switching_views_crossfades() {
    let burn = parked().with(Ch::Thrust, 1.0).with(Ch::Power, 1.0);
    for (first, then) in [
        (scene(seated(burn), true), scene(chase(burn), false)),
        (scene(chase(burn), false), scene(seated(burn), true)),
    ] {
        let sound = switched(first, then);
        let switch = (frames(4.0) / GAME_FRAME * GAME_FRAME) as f32 / RATE;
        let (before, early, after) = (
            at(&sound, switch - 0.05),
            at(&sound, switch),
            at(&sound, switch + 1.0),
        );
        assert!(
            (early - before).abs() < 0.5 * (after - before).abs(),
            "{before} {early} {after}"
        );
        let jump = |part: &[f32]| {
            part.windows(2)
                .fold(0.0_f32, |j, w| j.max((w[1] - w[0]).abs()))
        };
        let edge = (switch * RATE) as usize;
        let steady = jump(&sound[edge - frames(1.0)..edge]).max(jump(&sound[edge + frames(1.0)..]));
        assert!(jump(&sound[edge..edge + frames(0.5)]) <= 1.5 * steady);
    }
}

/// In a vacuum the chase view hears nothing of the engine, nor any wind, and the seat hears it
/// through the frame.
#[test]
fn a_vacuum_leaves_only_the_frame() {
    let burn = parked().with(Ch::Thrust, 1.0).with(Ch::Air, 0.0);
    let outside = switched(scene(chase(burn), false), scene(chase(burn), false));
    assert!(outside.iter().all(|x| *x == 0.0));
    let inside = switched(scene(seated(burn), true), scene(seated(burn), true));
    assert!(rms(&inside[frames(3.0)..]) > 0.005);
}
