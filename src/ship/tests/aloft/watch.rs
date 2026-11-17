//! What is checked on every frame of the flight: the camera, the body and the companion.

use bevy::prelude::*;

use super::Game;
use crate::avatar::{AvatarPose, Marks, Mode};
use crate::companion::Companion;
use crate::player::Player;
use crate::view::CameraPose;

/// Farthest the camera may move relative to the body in one frame, in metres.
const CAMERA_STEP: f32 = 1.2;

/// Largest turn of the camera in one frame, in radians.
const CAMERA_TURN: f32 = 0.6;

/// Farthest a wingtip or a sole of the drawn body may move in the frame of the body in one
/// frame, in metres.
const LIMB_STEP: f32 = 0.45;

/// Farthest the companion may be from the body once it caught up, in metres.
const COMPANION_REACH: f32 = 15.0;

/// What one frame showed.
#[derive(Debug, Clone, Copy)]
struct Seen {
    /// The camera relative to the body: offset and rotation.
    camera: (Vec3, Quat),
    /// The marks of the drawn body.
    marks: Marks,
}

/// The checks that run on every frame, with what the last frame showed.
#[derive(Debug, Default)]
pub struct Watch {
    /// The last frame, unless the next one is a cut.
    last: Option<Seen>,
    /// The modes of the body in the order they appeared.
    pub modes: Vec<Mode>,
    /// Whether the companion is held to its reach.
    pub near: bool,
    /// Largest move and turn of the camera seen in one frame.
    worst: (f32, f32),
}

/// Returns what the game shows this frame.
fn seen(game: &mut Game) -> Seen {
    let pose = game.get::<CameraPose>();
    let world = game.app.world_mut();
    let mut bodies = world.query_filtered::<&Transform, With<Player>>();
    let body = *bodies.single(world).expect("one player");
    Seen {
        camera: (pose.position - body.translation, pose.rotation),
        marks: game.get::<Marks>(),
    }
}

/// Returns how far the companion is from the body.
fn companion_gap(game: &mut Game) -> f32 {
    let world = game.app.world_mut();
    let mut bodies = world.query_filtered::<&GlobalTransform, With<Player>>();
    let body = bodies.single(world).expect("one player").translation();
    let mut bots = world.query_filtered::<&GlobalTransform, With<Companion>>();
    bots.single(world)
        .map_or(0.0, |bot| bot.translation().distance(body))
}

impl Watch {
    /// Makes the next frame a cut: nothing is compared with the frame before it.
    pub fn cut(&mut self) {
        self.last = None;
    }

    /// Runs `frames` frames and checks each one.
    pub fn step(&mut self, game: &mut Game, frames: usize) {
        for _ in 0..frames {
            game.step(1);
            self.check(game);
        }
    }

    /// Checks one frame against the one before it.
    fn check(&mut self, game: &mut Game) {
        let now = seen(game);
        let mode = game.get::<AvatarPose>().mode;
        if self.modes.last() != Some(&mode) {
            self.modes.push(mode);
        }
        if let Some(last) = self.last {
            self.compare(&last, &now, mode);
        }
        if self.near {
            let gap = companion_gap(game);
            assert!(
                gap < COMPANION_REACH,
                "the companion is {gap} m away in {mode:?}"
            );
        }
        self.last = Some(now);
    }

    /// Checks that the camera and the limbs moved no more than a frame allows since `last`.
    fn compare(&mut self, last: &Seen, now: &Seen, mode: Mode) {
        let moved = now.camera.0.distance(last.camera.0);
        let turned = now.camera.1.angle_between(last.camera.1);
        self.worst = (self.worst.0.max(moved), self.worst.1.max(turned));
        assert!(
            moved < CAMERA_STEP,
            "the camera jumped {moved} m in {mode:?}"
        );
        assert!(
            turned < CAMERA_TURN,
            "the camera turned {turned} rad in {mode:?}"
        );
        let limbs = now.marks.tips.iter().chain(&now.marks.soles);
        let before = last.marks.tips.iter().chain(&last.marks.soles);
        let jerk = limbs
            .zip(before)
            .map(|(a, b)| a.distance(*b))
            .fold(0.0, f32::max);
        assert!(jerk < LIMB_STEP, "a limb jumped {jerk} m in {mode:?}");
    }

    /// Prints what the run showed.
    pub fn report(&self) {
        println!("aloft: modes {:?}", self.modes);
        println!(
            "aloft: worst camera step {:.3} m, {:.3} rad",
            self.worst.0, self.worst.1
        );
    }
}
