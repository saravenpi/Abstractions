//! Player tests on a sphere: walking around the planet, rebasing the view, and the ship hand-over.

mod airframe;
mod basin;
mod climbing;
mod crouching;
mod deck;
mod eyes;
mod flight;
mod floor;
mod glider;
mod intents;
mod interact;
mod jet;
mod jumps;
mod lead;
mod lost;
mod opener;
mod respread;
mod sends;
mod spawn;
mod stage;
mod steer;
mod steps;
mod wings;

use std::f64::consts::TAU;

use avian3d::prelude::*;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::controller::Motion;
use super::spawn::Home;
use super::upright::facing;
use super::{BODY_LENGTH, BODY_RADIUS, Player, place, seat};
use crate::space::{Anchor, Frame, Shift, altitude, rebased, up};
use crate::universe::{STANDARD_GRAVITY, Universe};
use crate::world::Globe;

/// Sea level radius of the test planet in metres.
const RADIUS: f64 = 3000.0;

/// Returns the upright anchor whose origin is the sea level point along `dir`.
fn tangent_anchor(dir: DVec3) -> Anchor {
    Anchor {
        frame: Frame::Surface(Universe::new(4).home()),
        origin: dir * RADIUS,
        rotation: DQuat::from_rotation_arc(dir, DVec3::Y),
        flat: false,
    }
}

/// A body that walks straight ahead, rebased by the game rule.
struct Walker {
    /// Anchor the pose is expressed against.
    anchor: Anchor,
    /// Render position of the body.
    pos: Vec3,
    /// Heading around the local up.
    yaw: f32,
    /// Number of rebases so far.
    rebases: u32,
}

impl Walker {
    /// Steps `length` metres along the heading, the way the controller does: forward in the
    /// upright frame, back onto altitude `height`, then the rebase.
    ///
    /// Returns the altitude reached before the body is put back on `height`.
    fn step(&mut self, globe: &Globe, length: f32, height: f32) -> f32 {
        let vertical = up(&self.anchor, self.pos);
        self.pos += facing(vertical, self.yaw) * Vec3::NEG_Z * length;
        let reached = altitude(&self.anchor, globe, self.pos);
        self.pos -= up(&self.anchor, self.pos) * (reached - height);
        if let Some(next) = rebased(&self.anchor, RADIUS, self.pos) {
            self.pos = Shift::between(&self.anchor, &next).point(self.pos);
            self.anchor = next;
            self.rebases += 1;
        }
        reached
    }

    /// Returns the planet-frame position of the body.
    fn planet(&self) -> DVec3 {
        self.anchor.to_frame(self.pos)
    }
}

/// Walking straight on a 3 km globe follows a great circle: after 10 km the body is within 1 m
/// of the great circle point, after a full turn it is back within 1 m of the start, and its
/// altitude never leaves a 1 cm band.
#[test]
fn walking_straight_goes_around_the_planet() {
    let globe = Globe::new(4, RADIUS, STANDARD_GRAVITY);
    let height = 1.5f32;
    let anchor = tangent_anchor(DVec3::new(0.3, 0.8, -0.52).normalize());
    let mut walker = Walker {
        anchor,
        pos: Vec3::Y * height,
        yaw: 0.7,
        rebases: 0,
    };
    let start = walker.planet();
    let ahead = anchor.rotation.inverse() * (facing(Vec3::Y, 0.7) * Vec3::NEG_Z).as_dvec3();
    let orbit = RADIUS + f64::from(height);
    let steps = 80_000;
    let length = TAU * orbit / f64::from(steps);
    for step in 1..=steps {
        let reached = walker.step(&globe, length as f32, height);
        assert!(
            (reached - height).abs() < 0.01,
            "step {step}: altitude {reached}"
        );
        let walked = f64::from(step) * length;
        if (walked - 10_000.0).abs() < length * 0.5 {
            let angle = walked / orbit;
            let expected = start * angle.cos() + ahead * orbit * angle.sin();
            assert!(walker.planet().distance(expected) < 1.0);
        }
    }
    assert!(walker.rebases > 500, "{}", walker.rebases);
    let off = walker.planet().distance(start);
    assert!(off < 1.0, "{off} m from the start after a full turn");
}

/// A rebase carries an upright body onto the orientation its yaw gives in the new frame, so the
/// view does not turn.
#[test]
fn rebase_keeps_the_view() {
    let anchor = tangent_anchor(DVec3::new(-0.6, 0.1, 0.79).normalize());
    for (index, yaw) in [-2.9f32, -1.1, 0.0, 0.4, 2.2].into_iter().enumerate() {
        let pos = Vec3::new(31.0 + index as f32, 12.0, -22.0 * index as f32);
        let vertical = up(&anchor, pos);
        let rot = facing(vertical, yaw);
        let next = rebased(&anchor, RADIUS, pos).unwrap();
        let shift = Shift::between(&anchor, &next);
        let (pos, rot) = (shift.point(pos), shift.turn(rot));
        assert!((up(&next, pos) - Vec3::Y).length() < 1e-6);
        let wanted = facing(up(&next, pos), yaw);
        assert!((rot * Vec3::NEG_Z - wanted * Vec3::NEG_Z).length() < 1e-6);
        assert!((rot * Vec3::Y - wanted * Vec3::Y).length() < 1e-6);
    }
}

/// The ship can stand the body at a spot with a facing, and switch its collider off and on.
#[test]
fn ship_places_and_seats_the_body() {
    let anchor = tangent_anchor(DVec3::new(0.1, -0.7, 0.7).normalize());
    let mut world = World::new();
    world.insert_resource(anchor);
    let falling = Motion {
        velocity: Vec3::new(1.0, -30.0, 2.0),
        ..default()
    };
    let lost = Home(DVec3::ZERO);
    let body = world
        .spawn((Player, Transform::default(), falling, lost))
        .id();
    let feet = Vec3::new(18.0, 0.4, -9.0);
    place(&mut world, feet, Vec3::X);
    let vertical = up(&anchor, feet);
    let transform = *world.get::<Transform>(body).unwrap();
    let lift = (transform.translation - feet).dot(vertical);
    assert!(lift > BODY_LENGTH * 0.5 + BODY_RADIUS && lift < BODY_LENGTH * 0.5 + BODY_RADIUS + 0.1);
    assert!((transform.rotation * Vec3::Y - vertical).length() < 1e-5);
    let level = (Vec3::X - vertical * Vec3::X.dot(vertical)).normalize();
    assert!((transform.rotation * Vec3::NEG_Z - level).length() < 1e-4);
    let motion = *world.get::<Motion>(body).unwrap();
    assert_eq!(motion.velocity, Vec3::ZERO);
    assert!((facing(vertical, motion.yaw) * Vec3::NEG_Z - level).length() < 1e-4);
    let home = anchor.to_render(world.get::<Home>(body).unwrap().0);
    assert!(home.distance(transform.translation) < 1e-3);
    seat(&mut world, true);
    assert!(world.get::<ColliderDisabled>(body).is_some());
    seat(&mut world, false);
    assert!(world.get::<ColliderDisabled>(body).is_none());
}
