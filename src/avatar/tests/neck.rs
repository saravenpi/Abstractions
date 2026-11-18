//! Looking at the robot from every side in third person: the camera orbits a standing body all
//! the way round, the body keeps facing where it faced, and the head follows the view inside the
//! reach of its neck without a snap, so the face shows when the camera comes round to the front.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::FRAME;
use crate::avatar::animate::Anim;
use crate::avatar::dress::{dress, looked};
use crate::avatar::pose::Core;
use crate::avatar::rig::{Seg, globals};
use crate::avatar::tune::NECK_REACH;
use crate::avatar::{AvatarPose, Mode};
use crate::gear::FlightState;
use crate::player::wrap;
use crate::view::{Look, Moment, Rig, Shown, Target, ViewMode, ViewPrefs};

/// Heading of the standing body around the local up.
const BODY_YAW: f32 = 0.4;

/// Frames the camera takes to orbit once round the body: eight seconds.
const ORBIT: usize = 480;

/// Frames the camera rests behind the body before it orbits.
const REST: usize = 120;

/// Largest turn of the drawn head in one frame, in radians.
const HEAD_STEP: f32 = 0.05;

/// One frame of the orbit, every angle relative to the facing of the body.
struct Seen {
    /// Where the camera stands around the body.
    camera: f32,
    /// Where the view looks.
    view: f32,
    /// Where the neck turns the head, yaw and pitch.
    neck: Vec2,
    /// The yaw of the drawn head, glances included.
    head: f32,
    /// The yaw of the drawn chest.
    chest: f32,
    /// What the body is doing.
    mode: Mode,
}

/// Returns the yaw of the direction `to` around the local up, from -Z, positive to the left.
fn yaw_of(to: Vec3) -> f32 {
    (-to.x).atan2(-to.z)
}

/// Returns what one frame of the rig shows of a body posed by `anim` and dressed in `input`.
fn seen(shown: &Shown, target: &Target, anim: &mut Anim, input: &mut AvatarPose) -> Seen {
    let look = looked(&shown.heading, BODY_YAW);
    dress(input, &FlightState::default(), look, false);
    let pose = anim.step(input, FRAME);
    let chest = globals(&pose)[Seg::Torso as usize].rotation * Vec3::NEG_Z;
    Seen {
        camera: wrap(yaw_of(shown.pose.position - target.position) - BODY_YAW),
        view: look.x,
        neck: anim.neck.look,
        head: pose.core(Core::HeadYaw),
        chest: yaw_of(chest),
        mode: input.mode,
    }
}

/// Returns every frame of a third person camera that starts behind a standing body, rests,
/// then orbits it once to the left and a quarter turn more.
fn orbit() -> Vec<Seen> {
    let mut moment = Moment {
        target: Target {
            position: Vec3::Y * 0.9,
            ..default()
        },
        look: Look::default(),
        prefs: ViewPrefs {
            on_foot: ViewMode::Third,
            ..default()
        },
        fov: 1.309,
        dt: FRAME,
    };
    let mut rig = Rig::default();
    rig.face(crate::view::Heading {
        yaw: BODY_YAW,
        pitch: -0.2,
    });
    let (mut anim, mut input) = (Anim::default(), AvatarPose::default());
    let nothing = |_: Vec3, _: Vec3, _: f32, _: f32| None;
    let mut frames = Vec::new();
    for frame in 0..REST + ORBIT + ORBIT / 4 {
        let turning = frame >= REST;
        moment.look.yaw_delta = f32::from(u8::from(turning)) * TAU / ORBIT as f32;
        let shown = rig.step(&moment, &nothing);
        frames.push(seen(&shown, &moment.target, &mut anim, &mut input));
    }
    frames
}

/// Asserts what the head does on frame `index` of the orbit: it looks where the view looks
/// while the neck reaches, and toward the camera once the camera stands in front.
fn assert_aimed(index: usize, now: &Seen) {
    if index > REST / 2 && now.view.abs() < 0.8 {
        let off = (now.neck.x - now.view).abs();
        assert!(off < 0.3, "frame {index}: looks {off} away from the view");
    }
    if now.camera.abs() < 0.4 {
        let off = (now.neck.x - now.camera).abs();
        assert!(off < 0.5, "frame {index}: the face is {off} off the camera");
        let behind = now.view.abs() > PI - 0.5;
        assert!(behind, "frame {index}: the camera is not in front");
    }
}

/// The camera orbits all the way round a standing body: the drawn body never turns, the head
/// never jumps and never leaves the reach of the neck, follows the view where the neck reaches,
/// and faces the camera once it stands in front.
#[test]
fn the_camera_orbits_round_to_the_face() {
    let frames = orbit();
    let first = frames[0].chest;
    let front = frames.iter().filter(|seen| seen.camera.abs() < 0.05);
    assert!(
        front.count() > 0,
        "the camera never comes round to the front"
    );
    for (index, pair) in frames.windows(2).enumerate() {
        let (was, now) = (&pair[0], &pair[1]);
        assert_eq!(now.mode, Mode::Idle);
        let jump = (now.head - was.head).abs();
        assert!(jump < HEAD_STEP, "frame {index}: the head jumps {jump}");
        let turned = wrap(now.chest - first).abs();
        assert!(turned < 0.02, "frame {index}: the body turns {turned}");
        let reach = now.neck.abs();
        let inside = reach.x <= NECK_REACH.0 && reach.y <= NECK_REACH.1;
        assert!(inside, "frame {index}: the neck turns {reach}");
        assert_aimed(index, now);
    }
}
