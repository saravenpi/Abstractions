//! Tests of following and riding in the ship, without a window.

use bevy::prelude::*;

use super::brain::Brain;
use super::motion::{Float, follow, ride};
use super::ride::{boarding, leaving};
use super::spot::{Probe, Stance};
use super::tune::{CLEARANCE, FLOOR, SEAT};
use crate::space::Anchor;
use crate::universe::BodyId;
use crate::world::Terrain;

/// Frame time of the simulated world.
const DT: f32 = 1.0 / 60.0;

/// A player walking over the flat test world, with the companion following.
struct Walk {
    /// The test world.
    terrain: Terrain,
    /// Its anchor.
    anchor: Anchor,
    /// The companion's brain.
    brain: Brain,
    /// The companion's pose.
    at: Transform,
    /// The companion's motion.
    float: Float,
    /// Position of the player on the ground plane.
    player: Vec2,
}

impl Walk {
    /// Returns a walk starting at `player` on the flat legacy plane of `seed`, which does not move with the
    /// planet the universe chooses as home, the companion at the player's eyes.
    fn new(seed: u64, player: Vec2) -> Self {
        let terrain = Terrain::legacy(seed);
        let mut walk = Self {
            anchor: Anchor::surface(BodyId::default(), &terrain, Vec3::ZERO),
            terrain,
            brain: Brain::new(seed),
            at: Transform::IDENTITY,
            float: Float::default(),
            player,
        };
        walk.at.translation = walk.eye();
        walk
    }

    /// Returns where the player's eyes are: 1.6 m above the ground or the sea.
    fn eye(&self) -> Vec3 {
        let ground = self.terrain.height(self.player.x, self.player.y).max(0.0);
        Vec3::new(self.player.x, ground + 1.6, self.player.y)
    }

    /// Moves the player by `velocity` for one frame, lets the companion follow, and returns its gap to the ground.
    fn step(&mut self, velocity: Vec2) -> f32 {
        self.player += velocity * DT;
        let forward = Vec3::new(velocity.x, 0.0, velocity.y);
        let stance = Stance {
            eye: self.eye(),
            forward: forward.try_normalize().unwrap_or(self.brain.stance.forward),
            up: Vec3::Y,
        };
        self.brain.observe(&self.anchor, stance, DT);
        self.brain.look = stance.eye;
        let probe = Probe {
            anchor: &self.anchor,
            terrain: &self.terrain,
        };
        follow(&probe, &self.brain, &mut self.at, &mut self.float, DT);
        probe.gap(self.at.translation)
    }

    /// Returns how far the companion is from its resting spot, cleared of the ground.
    fn lag(&self) -> f32 {
        let probe = Probe {
            anchor: &self.anchor,
            terrain: &self.terrain,
        };
        let rest = probe.clear(self.brain.stance.rest(self.brain.perch.side), CLEARANCE);
        self.at.translation.distance(rest)
    }
}

/// Walking and sprinting over hills and sea, the body never touches the ground or the water, keeps
/// up with a sprint, and settles on its spot within two seconds of the player stopping.
#[test]
fn follows_softly_and_stays_clear() {
    let mut walk = Walk::new(1, Vec2::new(-300.0, 40.0));
    for (speed, seconds, reach) in [(4.5, 40.0, 2.5), (9.0, 40.0, 4.0)] {
        let mut worst: f32 = 0.0;
        for tick in 0..(seconds / DT) as usize {
            let heading = Vec2::from_angle(tick as f32 * DT * 0.2);
            let gap = walk.step(heading * speed);
            assert!(gap >= FLOOR - 2e-3, "gap {gap} at {}", walk.player);
            worst = worst.max(walk.lag() * f32::from(tick > 120));
        }
        assert!(worst < reach, "fell {worst} m behind at {speed} m/s");
    }
    for _ in 0..120 {
        walk.step(Vec2::ZERO);
    }
    assert!(walk.lag() < 0.05, "still {} m away", walk.lag());
    assert!(walk.float.vel.length() < 0.1);
    let facing = walk
        .at
        .forward()
        .dot((walk.eye() - walk.at.translation).normalize());
    assert!(
        facing > 0.95,
        "the screen faces {facing} away from the player"
    );
}

/// Left very far behind, it starts again just behind the player, while no camera is known,
/// instead of racing across the map, and flies to its place from there.
#[test]
fn starts_again_behind_when_left_far_behind() {
    let mut walk = Walk::new(42, Vec2::new(10.0, 10.0));
    walk.step(Vec2::X);
    walk.player += Vec2::new(900.0, -400.0);
    walk.step(Vec2::X);
    assert!(
        walk.float.vel.length() < 30.0,
        "it races at {}",
        walk.float.vel
    );
    let away = walk.at.translation - walk.eye();
    assert!(away.length() < 12.0, "{away}");
    assert!(away.x < 0.0, "it should be behind a player heading +X");
}

/// In the cockpit it settles on its seat at the side of the view and turns its screen to the player.
#[test]
fn rides_at_the_side_of_the_cockpit() {
    let eye = Transform::from_xyz(50.0, 900.0, -20.0).with_rotation(Quat::from_rotation_y(1.1));
    let world = Transform::from_translation(eye.translation + Vec3::new(40.0, 3.0, 9.0));
    let mut at = boarding(&eye, &world);
    assert!(at.translation.length() <= 3.0 + 1e-3);
    let back = leaving(
        &eye,
        &boarding(&eye, &Transform::from_xyz(50.5, 900.2, -21.0)),
    );
    assert!(back.translation.distance(Vec3::new(50.5, 900.2, -21.0)) < 1e-3);
    let mut float = Float::default();
    for _ in 0..120 {
        ride(Vec3::ZERO, &mut at, &mut float, DT);
    }
    let seat = Vec3::from_array(SEAT);
    assert!(at.translation.distance(seat) < 0.01);
    assert!(
        seat.x.abs() > 0.25 && seat.z < -0.3,
        "the seat is at the side, ahead"
    );
    assert!(at.forward().dot(-seat.normalize()) > 0.99);
}
