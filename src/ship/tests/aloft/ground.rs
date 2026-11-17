//! The stages on and near the ground: a jump, a landing into a run, a walk in third person and
//! a glide into the sea.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Game;
use super::watch::Watch;
use crate::avatar::Mode as Pose;
use crate::gear::Mode;
use crate::space::{Anchor, up};
use crate::view::{ViewMode, ViewPrefs};
use crate::world::Planet;

/// Spreads the wings of the body and sends it gliding at `speed` along its heading, sinking at
/// `sink` metres per second.
fn glide_in(game: &mut Game, speed: f32, sink: f32) {
    game.with_body(|motion, gear| {
        let forward = Vec3::new(-motion.yaw.sin(), 0.0, -motion.yaw.cos());
        motion.wings = true;
        gear.state.mode = Mode::Glide;
        gear.state.wings = 1.0;
        gear.state.servo.yaw = motion.yaw;
        gear.state.velocity = forward * speed + Vec3::Y * sink;
    });
}

/// Returns how far the walked `path` strays from the straight line from `start` to its end.
fn bend(start: DVec3, path: &[DVec3]) -> f64 {
    let end = *path.last().expect("a path");
    let chord = (end - start).normalize();
    path.iter()
        .map(|at| (*at - start).reject_from(chord).length())
        .fold(0.0, f64::max)
}

/// A tap of the jump key lifts the body off the ground and it lands again. Returns where the
/// feet stood, in the anchor frame.
pub fn jumps(game: &mut Game, watch: &mut Watch) -> DVec3 {
    let feet = game.get::<Anchor>().to_frame(game.walker().feet);
    watch.step(game, 10);
    game.tap(game.bound(|keys| keys.jump), watch);
    watch.step(game, 30);
    assert!(watch.modes.contains(&Pose::Jump), "{:?}", watch.modes);
    game.until("the jump to land", 120, |game| game.walker().grounded);
    feet
}

/// Stands the body 1.5 m over the start, facing away from the ship, once the ground there has
/// streamed in.
fn over_the_start(game: &mut Game, watch: &mut Watch, feet: DVec3) {
    let anchor = game.get::<Anchor>();
    let spot = anchor.to_render(feet);
    let vertical = up(&anchor, spot);
    let away = (spot - game.hull()).reject_from(vertical).normalize();
    crate::player::place(game.app.world_mut(), spot + vertical * 1.5, away);
    watch.cut();
    game.wait("the ground to stream in", |game| {
        let here = game.get::<Anchor>().to_render(feet);
        game.has_ground(here)
    });
}

/// Gliding in shallow and fast just over the ground near the start with the run keys held,
/// the body slides and runs on with its momentum.
pub fn lands_running(game: &mut Game, watch: &mut Watch, feet: DVec3) {
    over_the_start(game, watch, feet);
    let run = [
        game.bound(|keys| keys.forward),
        game.bound(|keys| keys.sprint),
    ];
    run.iter().for_each(|key| game.press(*key));
    glide_in(game, 14.0, -1.0);
    watch.cut();
    game.until("the body to touch down", 90, |game| {
        !game.body().gear.state.mode.airborne()
    });
    let mut slid = game.body().motion.velocity.with_y(0.0).length();
    for _ in 0..120 {
        watch.step(game, 1);
        let body = game.body();
        let speed = body.motion.velocity.with_y(0.0).length();
        if body.gear.state.mode == Mode::Off {
            assert!(body.motion.grounded && speed > 7.0, "ran on at {speed} m/s");
            assert!(
                slid - speed < 1.0,
                "the run started at {speed} m/s after {slid} m/s"
            );
            break;
        }
        slid = speed;
    }
    run.iter().for_each(|key| game.release(*key));
    let body = game.body();
    assert!(body.gear.state.mode == Mode::Off && !body.motion.wings);
}

/// Returns the direction from the centre of the planet of a point of the sea at least 10 m
/// deep within a few kilometres of the body.
fn sea_near(game: &mut Game) -> DVec3 {
    let (anchor, planet) = (game.get::<Anchor>(), game.get::<Planet>());
    let here = anchor.to_frame(game.body().at).normalize();
    let across = here.any_orthonormal_vector();
    let reach = (1..60).map(|step| f64::from(step) * 80.0 / planet.globe.radius());
    let turns = (0..24).map(|turn| f64::from(turn) * std::f64::consts::TAU / 24.0);
    let spots = reach.flat_map(|angle| turns.clone().map(move |turn| (angle, turn)));
    spots
        .map(|(angle, turn)| {
            let side = bevy::math::DQuat::from_axis_angle(here, turn) * across;
            (here * angle.cos() + side * angle.sin()).normalize()
        })
        .find(|dir| planet.globe.height(*dir) < -10.0)
        .expect("a sea within reach")
}

/// Gliding low over the sea and into it, the wings fold and swimming takes over.
pub fn glides_into_the_sea(game: &mut Game, watch: &mut Watch) {
    let dir = sea_near(game);
    let radius = game.get::<Planet>().globe.radius();
    let anchor = game.get::<Anchor>();
    let feet = anchor.to_render(dir * (radius + 6.0));
    let toward = anchor.to_render(dir.any_orthonormal_vector() + dir * radius) - feet;
    crate::player::place(game.app.world_mut(), feet, toward);
    watch.cut();
    watch.step(game, 2);
    glide_in(game, 12.0, -4.8);
    watch.cut();
    game.until("swimming to take over", 240, |game| {
        game.body().motion.swimming
    });
    watch.step(game, 30);
    let body = game.body();
    assert!(body.motion.swimming && body.gear.state.mode == Mode::Off && !body.motion.wings);
    assert!(watch.modes.contains(&Pose::Swim), "{:?}", watch.modes);
}

/// In third person, walking diagonally turns the robot to face where it goes, and with the
/// camera following on its own the walk stays straight.
pub fn walks_third_person(game: &mut Game, watch: &mut Watch) {
    if game.get::<ViewPrefs>().on_foot != ViewMode::Third {
        game.tap(game.bound(|keys| keys.view), watch);
    }
    assert!(game.get::<ViewPrefs>().auto_follow);
    watch.step(game, 30);
    let keys = [
        game.bound(|keys| keys.forward),
        game.bound(|keys| keys.right),
    ];
    keys.iter().for_each(|key| game.press(*key));
    watch.step(game, 20);
    let start = game.get::<Anchor>().to_frame(game.body().at);
    let mut path = Vec::new();
    for _ in 0..60 {
        watch.step(game, 1);
        path.push(game.get::<Anchor>().to_frame(game.body().at));
    }
    keys.iter().for_each(|key| game.release(*key));
    let bent = bend(start, &path);
    assert!(bent < 0.6, "the diagonal bent {bent} m off its line");
    let body = game.body();
    let going = body.motion.velocity;
    let heading = (-going.x).atan2(-going.z);
    let off = crate::player::wrap(heading - body.motion.yaw).abs();
    assert!(
        off < 0.2,
        "the robot faces {off} rad away from where it walks"
    );
}
