//! The stages high in the air, each checking what the one before left behind.

use bevy::prelude::*;

use super::watch::Watch;
use super::{Body, Game};
use crate::gear::{Mode, descent_target};
use crate::space::{Anchor, local_gravity, up};
use crate::view::{ViewMode, ViewPrefs};
use crate::world::Terrain;

/// Returns the specific energy of a flying body through the air: height and airspeed.
fn energy(body: &Body, gravity: f32) -> f32 {
    gravity * body.height + 0.5 * body.gear.state.airspeed.powi(2)
}

/// Returns the gravity at the body.
fn gravity(game: &mut Game) -> f32 {
    let (anchor, terrain) = (game.get::<Anchor>(), game.get::<Terrain>());
    local_gravity(&anchor, terrain.globe(), game.body().at).length()
}

/// Returns the render direction of the north of the upright frame at the body.
fn north(game: &mut Game) -> Vec3 {
    let vertical = up(&game.get::<Anchor>(), game.body().at);
    Quat::from_rotation_arc(Vec3::Y, vertical) * Vec3::NEG_Z
}

/// High above the ground, holding the run key fires the boots: the fall eases to their
/// descent rate and the body never rises above where they took it.
pub fn boots(game: &mut Game, watch: &mut Watch) {
    let toward = north(game);
    game.lift(260.0, toward);
    watch.cut();
    watch.step(game, 2);
    let run = game.bound(|keys| keys.sprint);
    game.press(run);
    game.until_mode(Mode::BootFall, 30, watch);
    let apex = game.body().height + 0.05;
    for frame in 0..240 {
        if frame == 100 {
            game.tap(game.bound(|keys| keys.view), watch);
        }
        watch.step(game, 1);
        let height = game.body().height;
        assert!(
            height <= apex,
            "the boots climbed to {height} m above {apex} m"
        );
    }
    let sink = -game.body().motion.velocity.y;
    let target = descent_target(gravity(game));
    assert!(
        (sink - target).abs() < 1.0,
        "sinking at {sink} m/s, not {target}"
    );
    assert_eq!(game.get::<ViewPrefs>().on_foot, ViewMode::Third);
}

/// The wingsuit alone glides and always loses energy. Trying to board in the air does nothing.
pub fn glides(game: &mut Game, watch: &mut Watch) {
    game.release(game.bound(|keys| keys.sprint));
    game.tap(game.bound(|keys| keys.wings), watch);
    game.until_mode(Mode::Glide, 30, watch);
    let g = gravity(game);
    let mut last = energy(&game.body(), g);
    for second in 0..6 {
        if second == 2 {
            game.tap(game.bound(|keys| keys.ship.board), watch);
            game.tap(game.bound(|keys| keys.view), watch);
        }
        watch.step(game, 30);
        let now = energy(&game.body(), g);
        assert!(now < last, "the glide gained energy: {last} to {now}");
        last = now;
    }
    assert_eq!(game.body().gear.state.mode, Mode::Glide);
    assert!(!game.walker().seated);
}

/// Wingsuit and boots together fly: the back key pulls the view and the nose up and climbs,
/// the forward key brings them level again, and level flight holds its height.
pub fn flies(game: &mut Game, watch: &mut Watch) {
    let boost = game.bound(|keys| keys.sprint);
    game.press(boost);
    game.until_mode(Mode::Fly, 30, watch);
    watch.step(game, 60);
    watch.near = true;
    let back = game.bound(|keys| keys.back);
    let before = game.body().height;
    game.press(back);
    watch.step(game, 45);
    game.release(back);
    let climbed = game.body().height;
    assert!(
        climbed > before,
        "pulling up sank from {before} m to {climbed} m"
    );
    let forward = game.bound(|keys| keys.forward);
    game.press(forward);
    watch.step(game, 45);
    game.release(forward);
    let level = game.body().height;
    watch.step(game, 180);
    let (body, kept) = (game.body(), level - 15.0);
    assert!(
        body.height > kept,
        "level flight sank to {} m under {kept} m",
        body.height
    );
    assert!(body.gear.state.airspeed > 15.0 && body.gear.state.mode == Mode::Fly);
}

/// Folding the wings in powered flight falls on without a jolt, and the boots catch the fall.
pub fn folds(game: &mut Game, watch: &mut Watch) {
    game.release(game.bound(|keys| keys.sprint));
    game.tap(game.bound(|keys| keys.wings), watch);
    let g = gravity(game);
    let mut was = game.body().gear.state.velocity;
    for _ in 0..30 {
        watch.step(game, 1);
        let now = game.body().gear.state.velocity;
        let pull = (now - was).length() * 30.0;
        assert!(
            pull < 3.0 * g,
            "the velocity jumped by {pull} m/s2 on the fold"
        );
        was = now;
    }
    assert_eq!(game.body().gear.state.mode, Mode::Fall);
    let run = game.bound(|keys| keys.sprint);
    game.press(run);
    game.until_mode(Mode::BootFall, 10, watch);
    watch.step(game, 240);
    let sink = -game.body().motion.velocity.y;
    let target = descent_target(g);
    assert!(
        (sink - target).abs() < 1.5,
        "the boots sink at {sink} m/s, not {target}"
    );
    game.release(run);
    watch.near = false;
}
