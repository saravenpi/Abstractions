//! Tests of the drawn ground under the animals: in the frame of an animal it stays where it is when the
//! chart is rooted elsewhere, as after a rebase, in the rig no foot of a drawn walker, near or far,
//! stands in it, and the sunk check sets a sunk walker back on it.

use std::sync::Arc;

use bevy::math::{DVec2, DVec3, Vec2};

use super::beast::Beast;
use super::ground::Heights;
use super::place::Spot;
use super::rig::app;
use super::stage::{run, staying};
use super::sunk::{depth, mend};
use super::want::Lead;
use crate::life::body::Locomotion;
use crate::world::globe::Face;
use crate::world::{Globe, Terrain};

/// Chart units a rebase moves the root of the chart by along each axis: one block.
const BLOCK: f64 = 3840.0;

/// Returns the height of the drawn ground of `terrain` in the frame `spot` over `at`, read through a
/// fresh cache as the walk reads it once the chart changed.
fn read(spot: &Spot, terrain: &Terrain, at: Vec2) -> f32 {
    let heights = Heights::default();
    spot.ground_by(terrain, at, |x, z| heights.drawn(terrain, None, x, z))
}

/// The ground under an animal, in its own frame, is the same before and after the chart is rooted a
/// block away, within 5 mm, and it is the finest drawn ground there.
#[test]
fn a_rebase_keeps_the_ground_under_an_animal() {
    let globe = Arc::new(Globe::new(9, 6000.0, 22.0));
    let dir = DVec3::new(0.21, 1.0, -0.13).normalize();
    let chart = globe.face_chart(Face::PosY).near(dir);
    let here = Terrain::on(globe.clone(), chart);
    let there = Terrain::on(
        globe.clone(),
        chart.rooted(chart.origin() + DVec2::new(BLOCK, -BLOCK)),
    );
    let spot = Spot::at(dir, globe.radius());
    for k in 0..200 {
        let at = Vec2::new((k % 20) as f32 * 3.1 - 30.0, (k / 20) as f32 * 5.3 - 25.0);
        let (before, after) = (read(&spot, &here, at), read(&spot, &there, at));
        assert!(
            (before - after).abs() < 5e-3,
            "{before} m before, {after} m after at {at}"
        );
        let drawn = spot.ground(&here, at);
        assert!(
            (before - drawn).abs() < 1e-3,
            "{before} m read, {drawn} m drawn at {at}"
        );
    }
}

/// Returns true when `beast` of the life of `terrain` walks on its feet.
fn walks(terrain: &Terrain, beast: &Beast) -> bool {
    let life = terrain.globe().life();
    let species = life.and_then(|l| l.species.get(usize::from(beast.index)));
    species.is_some_and(|s| s.body.locomotion == Locomotion::Walk)
}

/// In the rig, after three seconds of walking, no foot of any living drawn walker, near the robot or
/// far from it, stands 2 cm under the drawn ground. Flyers and swimmers do not stand on their feet.
#[test]
fn no_foot_of_a_drawn_animal_sinks() {
    let (mut app, _) = app(true);
    run(&mut app, 3.0, 1.0 / 30.0);
    let terrain = app.world().resource::<Terrain>().clone();
    let beasts = staying(&mut app);
    let living = beasts
        .iter()
        .filter(|b| !matches!(b.lead, Lead::Carcass { .. }) && walks(&terrain, b));
    let mut footed = 0;
    for beast in living {
        let spot = beast.spot;
        let Some(deep) = depth(beast, &|p: Vec2| spot.ground(&terrain, p)) else {
            continue;
        };
        assert!(deep < 0.02, "a foot {deep} m under the ground");
        footed += 1;
    }
    println!("{footed} animals with feet, none sunk");
    assert!(footed > 0, "no animal with feet in the rig");
}

/// A walker whose feet were pushed 3 m under the drawn ground is set back on it by the sunk check:
/// every foot on or over the ground and its body held no lower than the ground under it.
#[test]
fn a_sunk_walker_is_set_back_on_the_ground() {
    let (mut app, _) = app(true);
    run(&mut app, 1.0, 1.0 / 30.0);
    let terrain = app.world().resource::<Terrain>().clone();
    let mut beast = staying(&mut app)
        .into_iter()
        .find(|b| walks(&terrain, b) && !b.motion.feet.is_empty())
        .expect("a drawn walker");
    for foot in &mut beast.motion.feet {
        foot.at.y -= 3.0;
        foot.to.y -= 3.0;
    }
    beast.motion.root.hold.at.x -= 3.0;
    let spot = beast.spot;
    let ground = |p: Vec2| spot.ground(&terrain, p);
    let deep = mend(&mut beast, &ground).expect("the sunk walker is mended");
    assert!(deep > 2.9, "it stood only {deep} m deep");
    let left = depth(&beast, &ground).expect("feet");
    assert!(left <= 1e-4, "a foot is still {left} m under the ground");
    let base = ground(Vec2::new(beast.motion.root.at.x, beast.motion.root.at.z));
    assert!(
        beast.motion.root.hold.at.x >= base - 1e-4,
        "the body is still under the ground"
    );
}
