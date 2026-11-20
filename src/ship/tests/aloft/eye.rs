//! The camera through a loop or a roll: checked on every frame for a snap, and at the end for
//! a flight that comes out level with the camera upright behind it.

use bevy::prelude::*;

use super::Game;
use crate::gear::FlightState;
use crate::player::wrap;
use crate::space::{Anchor, up};
use crate::view::{CameraPose, Heading};

/// Largest move of the camera relative to the body in one frame, in metres, and its largest
/// turn in radians: a view that follows smoothly stays far under both, a snap or a flip does not.
const SMOOTH: (f32, f32) = (1.2, 0.35);

/// The camera of the frame before, relative to the body, and the worst frame seen.
///
/// The limbs of the drawn body are not checked here as the stages of the flight check them:
/// a body that loops and rolls swings its wingtips round faster than a glide does.
#[derive(Debug, Default)]
pub struct Eye {
    /// Offset from the body and rotation of the camera on the last frame.
    last: Option<(Vec3, Quat)>,
    /// Largest move and turn in one frame so far.
    pub worst: (f32, f32),
}

impl Eye {
    /// Runs one frame and checks that the camera neither jumped nor snapped round.
    pub fn step(&mut self, game: &mut Game) {
        game.step(1);
        let (pose, body) = (game.get::<CameraPose>(), game.body());
        let now = (pose.position - body.at, pose.rotation);
        if let Some(last) = self.last.replace(now) {
            let (moved, turned) = (now.0.distance(last.0), now.1.angle_between(last.1));
            self.worst = (self.worst.0.max(moved), self.worst.1.max(turned));
            let pitch = body.gear.state.pitch;
            assert!(
                moved < SMOOTH.0,
                "the camera jumped {moved} m at {pitch} rad"
            );
            assert!(
                turned < SMOOTH.1,
                "the camera snapped {turned} rad at {pitch} rad"
            );
        }
    }
}

/// Frames a flight may take after the keys are let go to fly nose first where the view looks.
const RECOVERY: usize = 300;

/// Returns the heading of the nose of a body in `state`, however the gear reads its attitude.
pub fn heading(state: &FlightState) -> f32 {
    let nose = state.nose();
    (-nose.x).atan2(-nose.z)
}

/// Returns how far the nose of the body is from where the view looks, around the vertical,
/// and how far from the way the body moves through the air.
fn astray(game: &mut Game) -> (f32, f32) {
    let state = game.body().gear.state;
    let view = game.get::<Heading>().yaw;
    (wrap(heading(&state) - view).abs(), state.aoa.abs())
}

/// Checks that the flight comes out the way it went in: on its wings, the right way up, flying
/// nose first where the view looks, with the camera upright behind it.
pub fn comes_out_level(game: &mut Game, eye: &mut Eye, what: &str) {
    let mut frames = 0;
    while frames < 75 || astray(game).0 > 0.3 || astray(game).1 > 0.6 {
        eye.step(game);
        frames += 1;
        assert!(
            frames < RECOVERY,
            "{what}: still astray by {:?}",
            astray(game)
        );
    }
    let body = game.body();
    let state = body.gear.state;
    let vertical = up(&game.get::<Anchor>(), body.at);
    let roof = game.get::<CameraPose>().rotation * Vec3::Y;
    assert!(
        roof.dot(vertical) > 0.5,
        "{what}: the camera ends on its side"
    );
    let view = game.get::<Heading>().pitch;
    assert!(view.abs() < 1.2, "{what}: the view ends at {view} rad");
    assert!(
        (state.frame() * Vec3::Y).y > 0.8 && state.mode.winged(),
        "{what}"
    );
    assert!(body.height > 30.0, "{what}: ended at {} m", body.height);
}
