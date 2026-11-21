//! Tests of a blow meeting the world: an overhead chop aimed at the ground stops on it with the
//! tool never under it, a stab into a wall stops at the wall and staggers back off it, and a
//! swing through open air meets nothing.

use bevy::prelude::*;

use super::super::pose::Pose;
use super::super::rig::{Seg, globals};
use super::act::{Act, Timing};
use super::bump::{stopped, swept};
use super::dir::Dir;
use super::feel::bend::Bend;
use super::feel::tune::STEP;
use super::held::Held;
use super::key::Key;
use super::layer::Layer;
use super::path::Path;
use super::solid::Meets;
use super::stage::{Plane, club, spear};
use super::tune::{HAND_GRIP, SAMPLES};

/// Frames a blow is cut into.
const FRAMES: usize = 30;

/// Returns the grip and the tip of `held` posed at share `s` of its path from `dir`, bent by `bend`,
/// in the frame of the body, through the rig the body is drawn with.
fn blade(held: &Held, (dir, bend): (Dir, &Bend), s: f32) -> [Vec3; 2] {
    let path = Path::of(held.style, dir);
    let key: Key = bend.turn(&path.at(s), 1.0);
    let layer = Layer::held(&key, held.build.hands, [true; 2]).with_body(
        (1.0, 2.0 * s - 1.0, true),
        bend,
        (0.5, STEP),
    );
    let bones = globals(&layer.apply(Pose::default()));
    let (torso, hand) = (bones[Seg::Torso as usize], bones[Seg::HandR as usize]);
    let grip = hand.transform_point(Vec3::new(0.0, -HAND_GRIP, 0.0));
    [
        grip,
        grip + (torso.rotation * layer.axis).normalize() * held.build.reach(),
    ]
}

/// Returns how far into `solid` the deepest point of [`SAMPLES`] along `blade` is: 0 when none is.
fn inside(solid: &Plane, blade: [Vec3; 2]) -> f32 {
    SAMPLES
        .iter()
        .map(|s| (blade[0].lerp(blade[1], *s) - solid.at).dot(solid.normal))
        .fold(0.0, |most: f32, d| most.max(-d))
}

/// Sweeps the blow of `held` from `dir` bent by `bend` against `solid` frame by frame, asserting
/// that no frame before the contact is inside it, and returns the act it stops in and the share of
/// the path it met it at.
fn blow(held: &Held, (dir, bend): (Dir, &Bend), solid: &Plane) -> Option<(Act, f32)> {
    let timing = Timing::of(held, &Path::of(held.style, dir));
    let dt = timing.swing / FRAMES as f32;
    for n in 1..=FRAMES {
        let (s0, s1) = ((n - 1) as f32 / FRAMES as f32, n as f32 / FRAMES as f32);
        let (was, now) = (blade(held, (dir, bend), s0), blade(held, (dir, bend), s1));
        assert!(
            inside(solid, was) < 1e-3,
            "frame {n} starts inside: {}",
            inside(solid, was)
        );
        if let Some(touch) = swept(solid, (was, now)) {
            let act = stopped((s1 * timing.swing, dt), &timing, touch.share, (50.0, 50.0));
            return Some((act, s0 + (s1 - s0) * touch.share));
        }
    }
    None
}

/// An overhead chop aimed at the ground at the feet stops on the ground, and the tool rests where it
/// stopped no deeper in it than a couple of centimetres, off it by the stagger.
#[test]
fn a_chop_at_the_ground_stops_on_it() {
    let held = club();
    let bend = Bend::of(Some(Vec3::new(0.05, -0.85, -0.7)), 1.0);
    let floor = Plane {
        at: Vec3::new(0.0, -0.5, 0.0),
        normal: Vec3::Y,
    };
    let (act, met) = blow(&held, (Dir::Up, &bend), &floor).expect("the chop met the ground");
    let Act::Stun(_, at) = act else {
        panic!("{act:?}");
    };
    assert!(at <= met + 1e-4, "{at} {met}");
    let rests = blade(&held, (Dir::Up, &bend), at);
    assert!(inside(&floor, rests) < 0.02, "{}", inside(&floor, rests));
}

/// A stab straight ahead into a wall stops at the wall and staggers back off it; with the wall out
/// of reach it meets nothing.
#[test]
fn a_stab_into_a_wall_stops_and_recoils() {
    let held = spear();
    let still = Bend::default();
    let wall = Plane {
        at: Vec3::new(0.0, 0.0, -1.35),
        normal: Vec3::Z,
    };
    let (act, met) = blow(&held, (Dir::Right, &still), &wall).expect("the stab met the wall");
    let Act::Stun(_, at) = act else {
        panic!("{act:?}");
    };
    assert!(at < met, "it did not stagger back: {at} {met}");
    assert!(inside(&wall, blade(&held, (Dir::Right, &still), at)) < 0.02);
    let far = Plane {
        at: Vec3::new(0.0, 0.0, -4.0),
        normal: Vec3::Z,
    };
    assert!(blow(&held, (Dir::Right, &still), &far).is_none());
    assert!(far.meet(Vec3::ZERO, Vec3::NEG_Z).is_none());
}
