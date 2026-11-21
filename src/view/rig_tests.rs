//! The rig heals: a value of its state that is not a finite number, or a moment that holds one,
//! is gone within one frame, and the camera comes back where a rig that never broke has it.

use bevy::prelude::*;

use super::Rig;
use crate::view::{Kind, Look, Moment, Shown, Target, ViewMode, ViewPrefs};

/// Returns a ship of 4.5 m in open space, seen from behind with auto-follow off, flying along
/// its nose at 300 metres per second.
fn ship() -> Moment {
    Moment {
        target: Target {
            position: Vec3::Y * 40.0,
            velocity: Vec3::NEG_Z * 300.0,
            radius: 4.5,
            kind: Kind::Ship,
            ..default()
        },
        look: Look::default(),
        prefs: ViewPrefs {
            ship: ViewMode::Third,
            auto_follow: false,
            ..default()
        },
        fov: 1.308_997,
        dt: 1.0 / 60.0,
    }
}

/// An open world: nothing in the way of the camera.
fn open(_: Vec3, _: Vec3, _: f32, _: f32) -> Option<f32> {
    None
}

/// Returns how far apart two cameras are: in metres, in radians and in field of view.
fn apart(a: &Shown, b: &Shown) -> (f32, f32, f32) {
    let turn = a.pose.rotation.angle_between(b.pose.rotation);
    let wide = (a.pose.fov - b.pose.fov).abs();
    (a.pose.position.distance(b.pose.position), turn, wide)
}

/// Ways to break each value the rig keeps between frames.
const SPOILS: [fn(&mut Rig); 20] = [
    |rig| rig.rel = Quat::NAN,
    |rig| rig.short = f32::NAN,
    |rig| rig.indoor = f32::INFINITY,
    |rig| rig.aim.heading.yaw = f32::NAN,
    |rig| rig.aim.view.pitch = f32::NAN,
    |rig| rig.aim.trail.x = f32::NAN,
    |rig| rig.aim.idle = f32::NAN,
    |rig| rig.aim.flight = f32::NAN,
    |rig| rig.blend.person = f32::NAN,
    |rig| rig.blend.flight = f32::NAN,
    |rig| rig.sway.lag.value.x = f32::NAN,
    |rig| rig.sway.lag.velocity.y = f32::INFINITY,
    |rig| rig.sway.stride = f32::NAN,
    |rig| rig.sway.bob = f32::NAN,
    |rig| rig.sway.fall = f32::NAN,
    |rig| rig.sway.roll.angle = f32::NAN,
    |rig| rig.sway.roll.look = Some(Quat::NAN),
    |rig| rig.sway.lead.y = f32::NAN,
    |rig| rig.sway.zoom = f32::NAN,
    |rig| rig.sway.rush = f32::NAN,
];

/// Whatever value of the state is broken, the next camera is a number, and five seconds later
/// it is where the camera of a twin rig that never broke is.
#[test]
fn a_broken_value_of_the_rig_heals() {
    let moment = ship();
    let mut settled = Rig::default();
    for _ in 0..120 {
        settled.step(&moment, &open);
    }
    for (index, spoil) in SPOILS.iter().enumerate() {
        let (mut rig, mut twin) = (settled, settled);
        spoil(&mut rig);
        let next = rig.step(&moment, &open);
        assert!(next.sound(), "spoil {index}: {next:?}");
        for _ in 0..300 {
            rig.step(&moment, &open);
            twin.step(&moment, &open);
        }
        let gap = apart(&rig.step(&moment, &open), &twin.step(&moment, &open));
        assert!(
            gap.0 < 1.0e-3 && gap.1 < 2.0e-3 && gap.2 < 1.0e-4,
            "spoil {index}: {gap:?}"
        );
    }
}

/// Ways to break what the rig reads on one frame.
const BAD: [fn(&mut Moment); 9] = [
    |moment| moment.target.velocity = Vec3::NAN,
    |moment| moment.target.velocity = Vec3::NEG_Z * 3.4e19,
    |moment| moment.target.spin = Vec3::INFINITY,
    |moment| moment.target.position = Vec3::NAN,
    |moment| moment.target.up = Vec3::NAN,
    |moment| moment.dt = f32::NAN,
    |moment| moment.fov = f32::NAN,
    |moment| moment.look.yaw_delta = f32::NAN,
    |moment| moment.prefs.zoom = f32::NAN,
];

/// One frame that reads a value that is not a number, or a speed whose square is past the range
/// of f32, gives a camera that is a number, and five seconds later the camera is where that of
/// a twin rig that read only good frames is.
#[test]
fn a_bad_frame_never_breaks_the_rig() {
    let moment = ship();
    let mut settled = Rig::default();
    for _ in 0..120 {
        settled.step(&moment, &open);
    }
    for (index, spoil) in BAD.iter().enumerate() {
        let (mut rig, mut twin, mut bad) = (settled, settled, moment);
        spoil(&mut bad);
        let next = rig.step(&bad, &open);
        assert!(next.sound(), "bad frame {index}: {next:?}");
        for _ in 0..300 {
            rig.step(&moment, &open);
            twin.step(&moment, &open);
        }
        let gap = apart(&rig.step(&moment, &open), &twin.step(&moment, &open));
        assert!(
            gap.0 < 1.0e-3 && gap.1 < 2.0e-3 && gap.2 < 1.0e-4,
            "bad frame {index}: {gap:?}"
        );
    }
}
