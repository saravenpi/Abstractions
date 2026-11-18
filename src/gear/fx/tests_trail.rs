//! Tests of the faults that made a trail vanish or cross solid things: a glide too slow to
//! glow, a ribbon sorted from the wrong place, a trail left unfed, and a jump drawn as a line.

use bevy::ecs::system::RunSystemOnce;
use bevy::math::{DQuat, DVec3};
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::scene::Scene;
use super::tests_fx::scene;
use super::trail::{BREAK, POINTS, Trail};
use super::trails::{Ribbon, Source, draw, tip_heat};
use crate::gear::Mode;
use crate::space::{Anchor, Frame};
use crate::universe::Universe;

/// Frame time of the test flights in seconds.
const DT: f32 = 1.0 / 60.0;

/// Returns a world with a surface anchor, one left wingtip ribbon and no scene yet.
fn world() -> World {
    let mut world = World::new();
    let mut meshes = Assets::<Mesh>::default();
    let handle = meshes.add(super::ribbon::mesh());
    world.insert_resource(meshes);
    world.insert_resource(Scene::default());
    world.insert_resource(Anchor {
        frame: Frame::Surface(Universe::new(1).home()),
        origin: DVec3::Y * 3000.0,
        rotation: DQuat::IDENTITY,
        flat: false,
    });
    world.spawn((
        Ribbon {
            source: Source::Tip(0),
            trail: Trail::default(),
            drawn: false,
        },
        Mesh3d(handle),
        Transform::IDENTITY,
    ));
    world
}

/// Flies the body of `world` in a glide at 40 m/s from `from` for `frames` frames, seen from
/// 8 m to its side, and draws the ribbon each frame. Returns where the body ended.
fn fly(world: &mut World, from: Vec3, frames: u32) -> Vec3 {
    let mut body = from;
    for _ in 0..frames {
        body += Vec3::NEG_Z * 40.0 * DT;
        world.insert_resource(Scene {
            body,
            rot: Quat::IDENTITY,
            eye: body + Vec3::X * 8.0,
            dt: DT,
            ..scene(Mode::Glide, 40.0, 1.0)
        });
        world.run_system_once(draw).expect("the trails draw");
    }
    body
}

/// Returns the ribbon of `world`, where its entity is and the positions and the strongest
/// alpha of its mesh.
fn drawn(world: &mut World) -> (Ribbon, Vec3, Vec<[f32; 3]>, f32) {
    let mut ribbons = world.query::<(&Ribbon, &Transform, &Mesh3d)>();
    let (ribbon, transform, mesh) = ribbons.single(world).expect("one ribbon");
    let (ribbon, at, handle) = (ribbon.clone(), transform.translation, mesh.0.clone());
    let mesh = world.resource::<Assets<Mesh>>().get(&handle).expect("mesh");
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("no positions");
    };
    let Some(VertexAttributeValues::Float32x4(colors)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR)
    else {
        panic!("no colours");
    };
    let alpha = colors.iter().map(|color| color[3]).fold(0.0, f32::max);
    (ribbon, at, positions.clone(), alpha)
}

/// A plain glide of four to one, at about 30 m/s, leaves trails that show: the glow of its
/// wingtips is over half of the full one, and a wing just over its stall still leaves some.
#[test]
fn a_steady_glide_keeps_its_wingtip_trails() {
    let glide = tip_heat(&scene(Mode::Glide, 30.0, 1.0));
    assert!(glide >= 0.5, "a glide at 30 m/s glows at {glide}");
    let slow = tip_heat(&scene(Mode::Glide, 20.0, 1.0));
    assert!(slow > 0.25 && slow < glide, "20 m/s glows at {slow}");
    let mut world = world();
    fly(&mut world, Vec3::new(0.0, 60.0, 0.0), 120);
    let (.., alpha) = drawn(&mut world);
    assert!(alpha > 0.4, "the ribbon of a glide shows at most {alpha}");
}

/// The ribbon entity sits at the head of its trail, far from the render origin or near it, and
/// its vertices lie around it. The renderer sorts a mesh without bounds among the see-through
/// things by where its entity is: left at the origin, a trail 300 m away was drawn before the
/// sea it flew over, which then hid it, or after a cloud or a pane in front of it.
#[test]
fn a_ribbon_sits_at_the_head_of_its_trail() {
    for from in [Vec3::new(280.0, 900.0, -150.0), Vec3::new(0.0, 5.0, 0.0)] {
        let mut world = world();
        let body = fly(&mut world, from, 120);
        let (ribbon, at, positions, _) = drawn(&mut world);
        let tip = body + Vec3::new(-0.85, 0.35, 0.0);
        assert!(at.distance(tip) < 1e-3, "ribbon at {at}, wingtip at {tip}");
        let reach = positions
            .iter()
            .map(|position| Vec3::from_array(*position).length())
            .fold(0.0, f32::max);
        assert!(reach > 20.0 && reach < 130.0, "vertices reach {reach} m");
        let oldest = Vec3::from_array(positions[positions.len() - 2]) + at;
        let laid = ribbon.trail.point(POINTS - 1).at - DVec3::Y * 3000.0;
        assert!(
            oldest.distance(laid.as_vec3()) < 1e-2,
            "{oldest} for {laid}"
        );
    }
}

/// A trail nobody feeds, with the player in the ship, is emptied and its mesh cleared, so
/// nothing hangs in the air. Fed again from another place, it does not run straight there.
#[test]
fn an_unfed_trail_is_cleared_and_does_not_reach_across() {
    let mut world = world();
    let left = fly(&mut world, Vec3::new(0.0, 80.0, 0.0), 90);
    assert!(drawn(&mut world).3 > 0.4, "no trail to begin with");
    world.insert_resource(Scene::default());
    world.run_system_once(draw).expect("the trails draw");
    let (ribbon, _, _, alpha) = drawn(&mut world);
    assert!(ribbon.trail.is_empty() && !ribbon.drawn && alpha == 0.0);
    fly(&mut world, left + Vec3::new(8.0, -3.0, 0.0), 2);
    let (ribbon, at, positions, _) = drawn(&mut world);
    let far = positions
        .iter()
        .map(|position| (Vec3::from_array(*position) + at).distance(left))
        .fold(f32::MAX, f32::min);
    assert!(
        ribbon.trail.len() <= 2 && far > 5.0,
        "a row {far} m from the old place"
    );
}

/// A body put somewhere else, 15 m away or just past [`BREAK`], starts its trail over, and the
/// fastest dive at the longest frame the controller takes never does.
#[test]
fn a_jump_starts_the_trail_over_and_a_fast_dive_does_not() {
    let mut trail = Trail::default();
    for step in 0..40 {
        trail.feed(DVec3::NEG_Z * (step as f64 * 110.0 * 0.05), 1.0, 0.05);
        assert!(trail.len() > step, "a dive broke the trail at step {step}");
    }
    for jump in [15.0, BREAK + 0.5] {
        let end = trail.tip().at;
        trail.feed(end + DVec3::X * jump, 1.0, DT);
        assert_eq!(trail.len(), 1, "a jump of {jump} m left a line");
        trail.feed(end + DVec3::X * (jump + 4.0), 1.0, DT);
    }
}
