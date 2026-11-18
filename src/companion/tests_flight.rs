//! Flying with the player: the formation at speed, an abrupt stop, and clearing the ground.

use bevy::prelude::*;

use super::brain::Brain;
use super::motion::{Float, follow};
use super::spot::{Probe, Stance};
use super::tune::FLIGHT_CLEARANCE;
use crate::space::Anchor;
use crate::universe::BodyId;
use crate::world::Terrain;

/// Frame time of the simulated world.
const DT: f32 = 1.0 / 60.0;

/// Height of the flight above the ground or the sea, in metres.
const ALOFT: f32 = 40.0;

/// A player flying over the flat test world with the companion in formation.
struct Flight {
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
    /// Position of the player's eyes.
    eye: Vec3,
}

impl Flight {
    /// Returns a flight over the world of `seed`, the companion at the player's eyes.
    fn new(seed: u64) -> Self {
        let terrain = Terrain::new(seed);
        let ground = terrain.height(0.0, 0.0).max(0.0);
        let eye = Vec3::new(0.0, ground + ALOFT, 0.0);
        let mut brain = Brain::new(seed);
        brain.flight = 1.0;
        Self {
            anchor: Anchor::surface(BodyId::default(), &terrain, Vec3::ZERO),
            terrain,
            brain,
            at: Transform::from_translation(eye),
            float: Float::default(),
            eye,
        }
    }

    /// Returns the reader of the ground.
    fn probe(&self) -> Probe<'_> {
        Probe {
            anchor: &self.anchor,
            terrain: &self.terrain,
        }
    }

    /// Moves the player by `velocity` for one frame and lets the companion follow.
    fn step(&mut self, velocity: Vec3) {
        self.eye += velocity * DT;
        let forward = velocity
            .try_normalize()
            .unwrap_or(self.brain.stance.forward);
        let stance = Stance {
            eye: self.eye,
            forward,
            up: Vec3::Y,
        };
        self.brain.observe(&self.anchor, stance, DT);
        self.brain.look = self.eye + forward * 10.0;
        let probe = Probe {
            anchor: &self.anchor,
            terrain: &self.terrain,
        };
        follow(&probe, &self.brain, &mut self.at, &mut self.float, DT);
    }

    /// Returns where the companion is from its formation spot, cleared of the ground.
    fn off(&self) -> Vec3 {
        let (goal, clearance) = self.brain.goal();
        self.at.translation - self.probe().clear(goal, clearance)
    }
}

/// At 90 m/s it keeps its place beside the player, out of the line of a camera over the right
/// shoulder, and never touches the ground or the sea.
#[test]
fn keeps_formation_at_ninety_metres_a_second() {
    let mut flight = Flight::new(3);
    let course = Vec3::new(0.6, 0.0, -0.8);
    let mut worst: f32 = 0.0;
    for tick in 0..(12.0 / DT) as usize {
        let speed = (tick as f32 * DT * 30.0).min(90.0);
        flight.step(course * speed);
        let gap = flight.probe().gap(flight.at.translation);
        assert!(gap > 0.5, "{gap} m over the ground at {speed} m/s");
        if speed >= 90.0 {
            worst = worst.max(flight.off().length());
        }
    }
    assert!(worst < 3.0, "{worst} m off its place at 90 m/s");
    let right = course.cross(Vec3::Y);
    let beside = flight.at.translation - flight.eye;
    assert!(beside.dot(right) < -1.5, "not on the left: {beside}");
    assert!(beside.dot(course) < 0.0, "not behind: {beside}");
}

/// When the player stops dead from 90 m/s it closes on its spot without swinging back and forth:
/// it goes at most a little past it, the distance along the course changes sign at most once,
/// and it ends at rest.
#[test]
fn settles_without_swinging_after_an_abrupt_stop() {
    let mut flight = Flight::new(5);
    let course = Vec3::NEG_Z;
    for _ in 0..(8.0 / DT) as usize {
        flight.step(course * 90.0);
    }
    let (mut crossings, mut last) = (0, flight.off().dot(course));
    let (mut fastest, mut past): (f32, f32) = (0.0, 0.0);
    for tick in 0..(6.0 / DT) as usize {
        let speed = (90.0 - tick as f32 * DT * 180.0).max(0.0);
        let was = flight.float.vel;
        flight.step(course * speed);
        fastest = fastest.max((flight.float.vel - was).length() / DT);
        let along = flight.off().dot(course);
        past = past.max(along);
        crossings += usize::from(along * last < 0.0 && along.abs() > 0.05);
        last = along;
    }
    assert!(
        crossings <= 1,
        "it swung {crossings} times through its spot"
    );
    assert!(past < 2.5, "it flew {past} m past its spot");
    assert!(
        flight.off().length() < 0.1,
        "still {} m off",
        flight.off().length()
    );
    assert!(flight.float.vel.length() < 0.1);
    assert!(fastest < 400.0, "braked at {fastest} m/s2");
}

/// Flying fast and low toward rising ground, it climbs before the ground reaches it.
#[test]
fn looks_ahead_for_the_ground_at_speed() {
    let mut flight = Flight::new(9);
    let course = Vec3::new(1.0, 0.0, 0.3).normalize();
    let mut lowest = f32::MAX;
    for _ in 0..(20.0 / DT) as usize {
        let ground = flight.terrain.height(flight.eye.x, flight.eye.z).max(0.0);
        flight.eye.y = ground + FLIGHT_CLEARANCE + 1.0;
        flight.step(course * 60.0);
        lowest = lowest.min(flight.probe().gap(flight.at.translation));
    }
    assert!(lowest > 0.2, "it came within {lowest} m of the ground");
}

/// Skimming four metres over hills at 60 m/s, the player climbing over every rise, the body
/// never moves against the player by more than the ground moved under the two of them, plus
/// what its spring does in a frame. The ground ahead only moves the place it aims for: it used
/// to lift the body itself, in one step, by whatever rose 48 m ahead.
#[test]
fn never_jumps_skimming_the_ground() {
    let mut flight = Flight::new(9);
    let course = Vec3::new(1.0, 0.0, 0.3).normalize();
    let floor = |terrain: &Terrain, at: Vec3| terrain.height(at.x, at.z).max(0.0);
    let (mut worst, mut was) = (f32::MIN, None::<(Vec3, f32)>);
    for _ in 0..(20.0 / DT) as usize {
        let under = floor(&flight.terrain, flight.eye);
        flight.eye.y = under + 4.0;
        flight.step(course * 60.0);
        let beside = flight.at.translation - flight.eye;
        let ground = under + floor(&flight.terrain, flight.at.translation);
        if let Some((last, level)) = was.replace((beside, ground)) {
            worst = worst.max(last.distance(beside) - 2.0 * (ground - level).abs());
        }
    }
    assert!(
        worst < 4.0,
        "it jumps {worst} m more than the ground in a frame"
    );
}
