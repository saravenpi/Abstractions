//! Occlusion: fast in, slow out, narrow things ignored, and never inside geometry.

use bevy::prelude::*;

use super::world::Boxes;
use super::{moment, run, third};
use crate::view::tune::{EASE_SPEED, HEAD_RADIUS, RADIUS, SKIN};
use crate::view::{Kind, Moment, Rig, ViewPrefs};

/// Returns the floor with a wall 1.5 m behind a body that stands at the origin and faces -Z.
fn walled() -> Boxes {
    Boxes::floor().with([-4.0, 0.0, 1.5], [4.0, 4.0, 1.7])
}

/// The camera is in front of a wall that appears behind the body within 0.1 s, and eases back
/// out no faster than the documented speed once the wall is gone.
#[test]
fn pulls_in_fast_and_eases_out_slowly() {
    let mut rig = Rig::default();
    let mut body = moment(Kind::OnFoot, third());
    let open = run(&mut rig, &mut body, &Boxes::floor(), 60)
        .pose
        .position
        .z;
    let blocked = run(&mut rig, &mut body, &walled(), 6).pose.position.z;
    assert!(blocked <= 1.5 - RADIUS && open > 3.0);
    let mut last = blocked;
    for _ in 0..240 {
        let now = run(&mut rig, &mut body, &Boxes::floor(), 1).pose.position.z;
        assert!(now >= last - 1.0e-5 && now - last <= EASE_SPEED * body.dt + 1.0e-4);
        last = now;
    }
    assert!((last - open).abs() < 0.02);
}

/// The whiskers shorten the boom before a wall beside it cuts the view, and they do it fast.
#[test]
fn whiskers_anticipate_a_wall() {
    let beside = Boxes::floor().with([1.2, 0.0, 1.0], [1.4, 4.0, 6.0]);
    let mut body = moment(Kind::OnFoot, third());
    let mut rig = Rig::default();
    let open = run(&mut rig, &mut body, &Boxes::floor(), 60).pose.position;
    let soon = run(&mut rig, &mut body, &beside, 6).pose.position;
    let settled = run(&mut rig, &mut body, &beside, 120).pose.position;
    assert!(
        settled.z < open.z - 0.2,
        "the whiskers did not shorten the boom"
    );
    assert!(
        beside.clearance(open) > RADIUS,
        "the wall cut the view itself"
    );
    assert!((soon.z - settled.z).abs() < 0.1 * (open.z - settled.z));
}

/// A post narrower than the size rule between the body and the camera changes nothing; a
/// pillar wider than the rule pulls the camera in.
#[test]
fn narrow_obstacles_are_ignored() {
    let view = |world: &Boxes| {
        let mut body = moment(Kind::OnFoot, third());
        run(&mut Rig::default(), &mut body, world, 90).pose.position
    };
    let open = view(&Boxes::floor());
    let post = view(&Boxes::floor().with([0.22, 0.0, 1.97], [0.3, 4.0, 2.03]));
    let pillar = view(&Boxes::floor().with([-0.2, 0.0, 1.8], [0.7, 4.0, 2.2]));
    assert!(post.distance(open) < 1.0e-4, "a thin post moved the camera");
    assert!(pillar.z < 1.8 - RADIUS + 1.0e-3);
}

/// Walks a body through the building while the player swings the camera, and asserts on
/// every frame that the camera keeps `margin` metres from every surface and never
/// jumps by more than 0.3 m relative to the body.
fn tour(prefs: ViewPrefs, margin: f32) {
    let world = Boxes::building();
    let mut rig = Rig::default();
    let mut body: Moment = moment(Kind::OnFoot, prefs);
    body.target.position = Vec3::new(0.0, 0.9, 8.0);
    let legs = [
        (Vec3::NEG_Z * 4.0, 330),
        (Vec3::X * 2.0, 60),
        (Vec3::NEG_X * 2.0, 60),
        (Vec3::Z * 4.0, 240),
    ];
    let (mut frame, mut last) = (0.0_f32, None);
    for (velocity, frames) in legs {
        body.target.velocity = velocity;
        for _ in 0..frames {
            frame += 1.0;
            body.look.yaw_delta = 1.5 * (frame * 0.021).cos() * body.dt;
            body.look.pitch_delta = 0.9 * (frame * 0.013).sin() * body.dt;
            let at = run(&mut rig, &mut body, &world, 1).pose.position;
            let gap = world.clearance(at);
            assert!(gap >= margin - 1.0e-3, "{gap} m from a surface at {at}");
            let beside: Vec3 = at - body.target.position;
            let jump = last.map_or(0.0, |was| beside.distance(was));
            assert!(jump <= 0.3, "the camera jumped by {jump} m at {at}");
            last = Some(beside);
        }
    }
}

/// The camera never ends inside geometry or within the near plane margin, and never jumps:
/// past walls, along a corridor, through a doorway and under a low ceiling, in third and in
/// first person.
#[test]
fn never_inside_geometry() {
    tour(third(), RADIUS);
    tour(ViewPrefs::default(), HEAD_RADIUS - SKIN);
}

/// Under a low ceiling the rig is shorter and lower than in the open.
#[test]
fn shortens_and_lowers_indoors() {
    let roofed = Boxes::floor().with([-30.0, 2.4, -30.0], [30.0, 2.6, 30.0]);
    let view = |world: &Boxes| {
        let mut body = moment(Kind::OnFoot, third());
        run(&mut Rig::default(), &mut body, world, 240)
            .pose
            .position
    };
    let (open, inside) = (view(&Boxes::floor()), view(&roofed));
    assert!(inside.z < open.z - 0.8 && inside.y < open.y - 0.2);
    assert!(roofed.clearance(inside) >= RADIUS - 1.0e-3);
}
