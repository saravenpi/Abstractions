//! The flying pose, the trail and the puppet: what a companion shows of its own motion.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::motion::Float;
use super::puppet::{moved, tilt};
use super::ribbon::{Look, ROWS, Strip, fill, glow};
use super::soar::soar;
use super::tail::heat;
use super::tune::{SOAR_LIMIT, TAIL_LIFE, TAIL_SPEED, TAIL_WIDTH};
use super::wake::{POINTS, SPACING, Wake};
use crate::space::{Anchor, Frame};
use crate::universe::BodyId;

/// Returns where the top of a body flying at `local` and gaining `gain` points, in its own axes.
fn top(local: Vec3, gain: Vec3) -> Vec3 {
    soar(local, gain) * Vec3::Y
}

/// At rest it stands upright; the faster it flies ahead the further its nose drops, never past
/// the limit; a turn banks it into the turn and braking tips it back.
#[test]
fn the_flying_pose_leans_the_way_the_thruster_must_push() {
    assert!(top(Vec3::ZERO, Vec3::ZERO).distance(Vec3::Y) < 1e-5);
    let mut last = 0.0;
    for speed in [10.0, 30.0, 60.0, 90.0, 300.0] {
        let nose = -top(Vec3::NEG_Z * speed, Vec3::ZERO).z;
        assert!(nose > last, "the nose rises at {speed} m/s");
        last = nose;
    }
    let steep = soar(Vec3::NEG_Z * 300.0, Vec3::ZERO).angle_between(Quat::IDENTITY);
    assert!(steep <= SOAR_LIMIT + 1e-4, "tipped {steep} rad");
    let cruise = Vec3::NEG_Z * 40.0;
    assert!(top(cruise, Vec3::X * 15.0).x > 0.3, "no bank to the right");
    assert!(
        top(cruise, Vec3::NEG_X * 15.0).x < -0.3,
        "no bank to the left"
    );
    assert!(
        top(cruise, Vec3::Z * 30.0).z > 0.0,
        "braking does not tip it back"
    );
    let plunge = soar(Vec3::NEG_Y * 80.0, Vec3::NEG_Y * 30.0);
    assert!((plunge * Vec3::Y).is_finite());
}

/// The lean drawn goes from the soft hover lean to the flying pose with how far it flies, and
/// a seated companion only hovers.
#[test]
fn the_pose_follows_how_far_it_flies() {
    let at = Transform::IDENTITY;
    let mut float = Float {
        vel: Vec3::NEG_Z * 60.0,
        ..default()
    };
    let hover = tilt(&at, &float).angle_between(Quat::IDENTITY);
    float.flight = 0.5;
    let half = tilt(&at, &float).angle_between(Quat::IDENTITY);
    float.flight = 1.0;
    let full = tilt(&at, &float).angle_between(Quat::IDENTITY);
    assert!(hover < half && half < full, "{hover} {half} {full}");
    float.seated = true;
    assert!((tilt(&at, &float).angle_between(Quat::IDENTITY) - hover).abs() < 1e-4);
}

/// A puppet's velocity comes from its pose in the frame of the anchor: true along a straight
/// run, and zero on the frame the frame itself changes.
#[test]
fn a_puppet_reads_its_velocity_from_its_pose() {
    let anchor = Anchor {
        frame: Frame::Surface(BodyId::default()),
        origin: DVec3::new(10.0, 0.0, 0.0),
        rotation: bevy::math::DQuat::from_rotation_y(0.5),
        flat: false,
    };
    let was = DVec3::new(100.0, 5.0, 0.0);
    let now = was + DVec3::new(0.0, 0.0, -0.5);
    let vel = moved(&anchor, Some((anchor.frame, was)), now, 1.0 / 60.0);
    assert!((vel.length() - 30.0).abs() < 1e-2, "{vel}");
    assert_eq!(moved(&anchor, None, now, 1.0 / 60.0), Vec3::ZERO);
}

/// The trail glows only in flight and at speed, and is laid by distance whatever the frame
/// rate: a third of the length of a wingtip trail, each point where the companion really was.
#[test]
fn the_trail_is_laid_where_it_flew() {
    assert_eq!(heat(1.0, 0.0), 0.0);
    assert_eq!(heat(0.0, 80.0), 0.0);
    assert_eq!(heat(1.0, TAIL_SPEED.1 + 1.0), 1.0);
    assert!(heat(0.5, 80.0) > 0.4 && heat(0.5, 80.0) < 0.6);
    for fps in [30.0_f64, 144.0] {
        let mut wake = Wake::default();
        let frames = (2.0 * fps) as usize;
        for frame in 0..=frames {
            let at = DVec3::new(0.0, 100.0, -60.0 * frame as f64 / fps);
            wake.feed(at, 1.0, (1.0 / fps) as f32);
        }
        assert_eq!(wake.len(), POINTS);
        for index in 1..POINTS {
            let gap = wake.point(index - 1).at.distance(wake.point(index).at);
            assert!((gap - SPACING).abs() < 1e-6, "at {fps} fps: {gap} m apart");
            assert!(wake.point(index).at.x == 0.0 && wake.point(index).at.y == 100.0);
            assert!(wake.point(index).age > wake.point(index - 1).age);
        }
        assert!(POINTS as f64 * SPACING < 112.0 / 2.5);
    }
}

/// The ribbon is thin, soft at both ends and gone with age: nothing glows at the tip or past
/// the life of a point, no row is wider than the trail, and a jump starts it over.
#[test]
fn the_ribbon_fades_and_never_streaks() {
    let mut wake = Wake::default();
    for frame in 0..40 {
        wake.feed(DVec3::new(0.0, 0.0, -f64::from(frame)), 1.0, 1.0 / 60.0);
    }
    assert_eq!(glow(&wake, 0, TAIL_LIFE), 0.0, "the tip is lit");
    assert!(glow(&wake, 3, TAIL_LIFE) > 0.1);
    assert_eq!(glow(&wake, ROWS - 1, TAIL_LIFE), 0.0, "the end is lit");
    let look = Look {
        eye: Vec3::new(4.0, 1.0, -20.0),
        width: TAIL_WIDTH,
        life: TAIL_LIFE,
        tint: Vec3::ONE,
    };
    let mut strip = Strip::default();
    fill(&wake, &look, |point| point.as_vec3(), &mut strip);
    for row in 0..ROWS {
        let [left, right] = [0, 2].map(|slot| Vec3::from_array(strip.positions[row * 3 + slot]));
        assert!(left.distance(right) <= 2.0 * TAIL_WIDTH + 1e-5);
        assert_eq!(strip.colors[row * 3][3], 0.0, "an edge is lit");
    }
    assert!(wake.live(TAIL_LIFE));
    (0..60).for_each(|_| wake.feed(DVec3::new(0.0, 0.0, -39.0), 0.0, 1.0 / 60.0));
    assert!(!wake.live(TAIL_LIFE), "it glows after its life");
    wake.feed(DVec3::new(0.0, 5000.0, 0.0), 1.0, 1.0 / 60.0);
    assert_eq!(wake.len(), 1, "a jump streaked across the world");
}
