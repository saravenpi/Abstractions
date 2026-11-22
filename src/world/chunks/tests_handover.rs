//! Hand-over test: past the hand-over the nodes of the world that were on the screen dither out
//! over a fade, above the ball the sky draws sunk under them, instead of vanishing on one frame.

use std::time::{Duration, Instant};

use bevy::light::NotShadowCaster;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::Chunks;
use super::dither::{FADE_SECONDS, Way};
use super::tests_stream::{settle_at_spawn, world};
use crate::space::Anchor;
use crate::world::{SpawnPoint, Terrain};

/// Returns the entities of the nodes the world draws now.
fn drawn(app: &App) -> Vec<Entity> {
    let chunks = app.world().resource::<Chunks>();
    let shown = chunks.nodes.live.values().filter(|node| node.visible);
    shown.map(|node| node.entity).collect()
}

/// Moves the focus to two radii out and runs frames until the world hands the planet over to the
/// sky, and returns the nodes drawn on the frame before.
fn leave(app: &mut App) -> Vec<Entity> {
    let radius = app.world().resource::<Terrain>().globe().radius();
    let anchor = *app.world().resource::<Anchor>();
    let spawn = SpawnPoint(anchor.to_render(DVec3::Y * radius * 2.0));
    app.world_mut().insert_resource(spawn);
    let start = Instant::now();
    loop {
        let before = drawn(app);
        app.update();
        if app.world().resource::<Chunks>().far.active() {
            return before;
        }
        assert!(start.elapsed() < Duration::from_secs(180), "no hand-over");
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// The nodes on the screen at the hand-over are no longer nodes of the quadtree but still drawn,
/// dithering out without a shadow, and gone a little after the fade ends.
#[test]
fn the_ground_melts_into_the_sky_at_the_hand_over() {
    let mut app = world();
    settle_at_spawn(&mut app);
    let shown = leave(&mut app);
    assert!(!shown.is_empty(), "nothing was drawn before the hand-over");
    let chunks = app.world().resource::<Chunks>();
    assert!(chunks.nodes.live.is_empty());
    for entity in &shown {
        assert_eq!(chunks.fades.way(*entity), Some(Way::Leave), "{entity:?}");
        assert!(app.world().get::<NotShadowCaster>(*entity).is_some());
    }
    let start = Instant::now();
    let limit = Duration::from_secs_f32(FADE_SECONDS) + Duration::from_secs(10);
    while shown
        .iter()
        .any(|entity| app.world().get_entity(*entity).is_ok())
    {
        assert!(start.elapsed() < limit, "the faded nodes stay");
        app.update();
        std::thread::sleep(Duration::from_millis(5));
    }
}
