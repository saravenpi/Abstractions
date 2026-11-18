//! Tests of where the companion looks: on its own, not where the camera points, turning its
//! head no faster than its neck allows, with the body following the head.

use bevy::prelude::*;

use super::attention::Focus;
use super::brain::Brain;
use super::gaze::Attention;
use super::mind::Input;
use super::motion::{Float, follow};
use super::neck::Neck;
use super::scene::{Scene, course};
use super::spot::{Probe, Stance};
use super::tune::{HEAD_RATE, NECK};
use crate::space::Anchor;
use crate::universe::BodyId;
use crate::world::Terrain;

/// Frame time of the simulated world.
const DT: f32 = 1.0 / 60.0;

/// A player on the flat test world, a camera that turns as it likes, and the companion.
struct Watch {
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

impl Watch {
    /// Returns a player standing on the world of `seed`, the companion beside it.
    fn new(seed: u64) -> Self {
        let terrain = Terrain::new(seed);
        let eye = Vec3::new(0.0, terrain.height(0.0, 0.0).max(0.0) + 1.6, 0.0);
        Self {
            anchor: Anchor::surface(BodyId::default(), &terrain, Vec3::ZERO),
            terrain,
            brain: Brain::new(seed),
            at: Transform::from_translation(eye + Vec3::X * 2.0),
            float: Float::default(),
            eye,
        }
    }

    /// Returns what the companion sees with the camera looking along `view` from `camera`.
    fn scene(&self, view: Vec3, camera: Vec3) -> Scene {
        let (eye, up, drift) = (self.eye, Vec3::Y, self.brain.drift);
        Scene {
            here: self.at.translation,
            heading: self.at.rotation * self.float.neck.rotation() * Vec3::NEG_Z,
            eye,
            up,
            drift,
            course: course(drift, up),
            camera,
            view,
            sights: [None; 3],
        }
    }

    /// Moves the player by `velocity` for one frame with the camera looking along `view` from
    /// `camera`, or from behind the player, lets the companion think, look and follow, as the
    /// systems do, and returns the way its head faces.
    fn step(&mut self, velocity: Vec3, view: Vec3, camera: Option<Vec3>) -> Vec3 {
        self.eye += velocity * DT;
        let (eye, up) = (self.eye, Vec3::Y);
        let stance = Stance {
            eye,
            forward: view,
            up,
        };
        self.brain.observe(&self.anchor, stance, DT);
        self.brain.scene = self.scene(view, camera.unwrap_or(eye - view * 3.0 + up * 0.5));
        let brain = &mut self.brain;
        let (speed, night, seated) = (brain.drift.length(), false, false);
        brain.mind.advance(&Input {
            speed,
            night,
            seated,
            dt: DT,
        });
        let scene = brain.scene;
        brain.attention.advance(&scene, DT);
        brain.look = brain.focus();
        let probe = Probe {
            anchor: &self.anchor,
            terrain: &self.terrain,
        };
        follow(&probe, &self.brain, &mut self.at, &mut self.float, DT);
        self.at.rotation * self.float.neck.rotation() * Vec3::NEG_Z
    }
}

/// A player who stands and swings the camera round in circles for a minute does not drag the
/// companion's eyes along: its head points within 10 degrees of the camera on less than one
/// frame in six, it never turns faster than its neck allows, and it often looks at the player.
#[test]
fn looks_on_its_own_while_the_camera_circles() {
    let mut watch = Watch::new(4);
    let (mut along, mut at_player, mut fastest, frames) = (0, 0, 0f32, 3600);
    let mut was = watch.step(Vec3::ZERO, Vec3::NEG_Z, None);
    for tick in 0..frames {
        let t = tick as f32 * DT;
        let view = Quat::from_rotation_y(t * 1.3 + (t * 0.4).sin()) * Vec3::NEG_Z;
        let head = watch.step(Vec3::ZERO, view, None);
        fastest = fastest.max(head.angle_between(was) / DT * f32::from(tick > 0));
        was = head;
        along += usize::from(
            head.with_y(0.0)
                .normalize_or_zero()
                .dot(view.with_y(0.0).normalize_or_zero())
                > 10f32.to_radians().cos(),
        );
        let toward = (watch.eye - watch.at.translation).normalize();
        at_player += usize::from(head.dot(toward) > 15f32.to_radians().cos());
    }
    let share = along as f32 / frames as f32;
    assert!(
        share < 1.0 / 6.0,
        "the head follows the camera {share} of the time"
    );
    assert!(
        fastest <= HEAD_RATE * 1.05,
        "the head turned at {fastest} rad/s"
    );
    let seen = at_player as f32 / frames as f32;
    assert!(seen > 0.1, "it looked at the player {seen} of the time");
}

/// A player running fast is looked ahead of along its course most of the time, whatever the
/// camera does, and a sudden stop draws a glance at the player within a second and a half.
#[test]
fn looks_where_a_fast_player_runs_and_at_it_when_it_stops() {
    let mut watch = Watch::new(11);
    let (mut ahead, frames) = (0, 1800);
    for tick in 0..frames {
        let view = Quat::from_rotation_y(tick as f32 * DT * 1.7) * Vec3::NEG_Z;
        let head = watch.step(Vec3::X * 8.0, view, None);
        ahead += usize::from(tick > 120 && head.with_y(0.0).normalize_or_zero().dot(Vec3::X) > 0.9);
    }
    let share = ahead as f32 / (frames - 120) as f32;
    assert!(
        share > 0.5,
        "it looked where the player runs {share} of the time"
    );
    let glanced = (0..90).any(|_| {
        watch.step(Vec3::ZERO, Vec3::NEG_Z, None);
        watch.brain.attention.focus == Focus::Player
    });
    assert!(glanced, "a stop did not draw a glance");
}

/// A still player who keeps the camera on the companion gets a look back at the camera within
/// ten seconds, which never lasts three.
#[test]
fn looks_back_briefly_at_a_watching_camera() {
    let mut watch = Watch::new(7);
    let (mut first, mut run, mut longest) = (None, 0, 0);
    for tick in 0..900 {
        let camera = watch.eye + Vec3::new(0.0, 0.5, 3.0);
        let view = (watch.at.translation - camera).normalize();
        watch.step(Vec3::ZERO, view, Some(camera));
        let back = watch.brain.attention.focus == Focus::Camera;
        first = first.or(back.then_some(tick));
        run = if back { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    let first = first.expect("it never looked back at the camera");
    assert!(first < 600, "it took {first} frames to look back");
    assert!(
        longest < 180,
        "it stared at the camera for {longest} frames"
    );
}

/// Seated in the ship it only looks out ahead or at the pilot.
#[test]
fn looks_ahead_or_at_the_pilot_in_the_cockpit() {
    let mut attention = Attention::new(5);
    let scene = Scene {
        up: Vec3::Y,
        course: Some(Vec3::NEG_Z),
        view: Vec3::NEG_Z,
        ..Scene::default()
    };
    let mut seen = [false; 2];
    for _ in 0..3600 {
        attention.advance(&scene, DT);
        let focus = attention.focus;
        assert!(matches!(focus, Focus::Player | Focus::Travel), "{focus:?}");
        seen[usize::from(focus == Focus::Travel)] = true;
    }
    assert_eq!(seen, [true, true]);
}

/// A target that jumps behind the companion is turned to smoothly: the head never turns faster
/// than its top rate nor farther from the body than the neck allows, and both end on target.
#[test]
fn the_head_leads_and_the_body_follows() {
    let (mut neck, mut body) = (Neck::default(), Quat::IDENTITY);
    let target = Vec3::new(0.3, 0.4, 1.0).normalize();
    let reach = Vec2::from(NECK).length() + 1e-3;
    let mut was = Vec3::NEG_Z;
    let mut lagged = false;
    for _ in 0..240 {
        body = neck.turn(body, target, Vec3::Y, DT);
        let head = body * neck.rotation() * Vec3::NEG_Z;
        assert!(head.angle_between(was) <= HEAD_RATE * DT * 1.05);
        assert!(neck.head.length() <= reach, "{}", neck.head);
        lagged |= neck.head.length() > 0.3;
        was = head;
    }
    assert!(lagged, "the body turned with the head like a turret");
    assert!(was.angle_between(target) < 0.02);
    assert!((body * Vec3::NEG_Z).angle_between(target) < 0.05);
}
