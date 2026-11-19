//! Loops and rolls on the wings in a whole game: the keys held down through the real keys, the
//! view rig and the gear, in both views. Under power the back key loops, or let go at the top
//! rolls out the other way; the roll keys roll a glide over and on round; on a plain glide the
//! back key stalls out of the climb. Through all of it the camera follows without a snap.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::prelude::*;

use super::eye::{Eye, comes_out_level, heading};
use super::watch::Watch;
use super::{Body, Game};
use crate::gear::Mode;
use crate::player::wrap;
use crate::space::{Anchor, up};
use crate::view::{ViewMode, ViewPrefs};

/// Height above the sea the flights start at, in metres: low enough for the full grip of the
/// air, high enough to dive, loop and stall out.
const START: f32 = 620.0;

/// Frames a flight is left alone to settle before the keys are held, at 30 frames per second.
const SETTLE: usize = 150;

/// Frames a key may be held for the body to come all the way round.
const PATIENCE: usize = 300;

/// Puts the body high in the air on its wings, with the boots on when `powered`, and lets the
/// flight settle on the speed it keeps by itself.
fn takes_wing(game: &mut Game, watch: &mut Watch, powered: bool) -> Body {
    let vertical = up(&game.get::<Anchor>(), game.body().at);
    let north = Quat::from_rotation_arc(Vec3::Y, vertical) * Vec3::NEG_Z;
    let lift = START - game.body().height;
    game.lift(lift, north);
    watch.cut();
    watch.step(game, 2);
    game.tap(game.bound(|keys| keys.wings), watch);
    game.until_mode(Mode::Glide, 30, watch);
    let boost = game.bound(|keys| keys.sprint);
    if powered {
        game.press(boost);
    }
    watch.step(game, SETTLE);
    game.body()
}

/// Holds `key` until the body turned all the way round, its pitch for a loop or its bank for a
/// roll, and returns how many frames it took and whether it was ever on its back.
fn rounds(game: &mut Game, eye: &mut Eye, key: KeyCode, roll: bool) -> (usize, bool) {
    let angle = |game: &mut Game| {
        let state = game.body().gear.state;
        if roll { state.bank } else { state.pitch }
    };
    game.press(key);
    let (mut turned, mut frames, mut over) = (0.0_f32, 0, false);
    while turned.abs() < TAU - 0.35 {
        let was = angle(game);
        eye.step(game);
        turned += wrap(angle(game) - was);
        frames += 1;
        let state = game.body().gear.state;
        over |= (state.frame() * Vec3::Y).y < -0.8;
        assert!(
            frames < PATIENCE,
            "the body hangs {turned} rad round at {} m/s",
            state.airspeed
        );
        assert!(state.mode.winged(), "the wings folded {turned} rad round");
    }
    game.release(key);
    (frames, over)
}

/// Holds the back key on a glide too slow to loop and checks that the nose never passes the
/// vertical and the body never ends on its back: the wing stalls out of the climb.
fn stalls(game: &mut Game, eye: &mut Eye) {
    let back = game.bound(|keys| keys.back);
    game.press(back);
    for frame in 0..150 {
        eye.step(game);
        let state = game.body().gear.state;
        assert!(
            state.pitch < FRAC_PI_2,
            "frame {frame}: looped at {} rad",
            state.pitch
        );
        assert!(
            (state.frame() * Vec3::Y).y > -0.3,
            "frame {frame}: on its back"
        );
    }
    game.release(back);
}

/// What a flight does once it has taken wing, by name, and whether the boots fly it.
const FIGURES: [(&str, bool); 4] = [
    ("loops", true),
    ("rolls out at the top", true),
    ("rolls", false),
    ("stalls", false),
];

/// Holds the back key until the body is on its back at the top of a loop, then lets go: the
/// wings roll level by themselves, the right way up on the heading the other way.
fn rolls_out(game: &mut Game, eye: &mut Eye) {
    let back = game.bound(|keys| keys.back);
    game.press(back);
    let mut frames = 0;
    while (game.body().gear.state.frame() * Vec3::Y).y > -0.8 {
        eye.step(game);
        frames += 1;
        assert!(frames < PATIENCE, "never on its back");
    }
    game.release(back);
}

/// Flies the figure `how` from where the flight is, watched by `eye`, and returns how far it
/// should turn the heading and by how much it may miss: a roll out at the top flies on the
/// other way, about.
fn figure(game: &mut Game, eye: &mut Eye, how: &str) -> (f32, f32) {
    let (back, right) = (game.bound(|keys| keys.back), game.bound(|keys| keys.right));
    match how {
        "stalls" => stalls(game, eye),
        "rolls out at the top" => rolls_out(game, eye),
        _ => {
            let rolls = how == "rolls";
            let key = if rolls { right } else { back };
            let (frames, over) = rounds(game, eye, key, rolls);
            assert!(over, "{how}: never on its back in {frames} frames");
        }
    }
    match how {
        "rolls out at the top" => (PI, 1.2),
        _ => (0.0, 0.6),
    }
}

/// Flies every one of [`FIGURES`] in the view `mode`, and checks that each comes out level on
/// the heading it should, the camera smooth all the way.
fn loops_and_rolls(tag: &str, mode: ViewMode) {
    let mut game = Game::full(tag);
    let game = &mut game;
    game.start("Loops", None);
    game.until("the player to stand", 300, |game| game.walker().grounded);
    game.app.world_mut().resource_mut::<ViewPrefs>().on_foot = mode;
    let mut watch = Watch::default();
    for (how, powered) in FIGURES {
        let what = format!("{mode:?}, {how}");
        println!("loops: {what}");
        let before = heading(&takes_wing(game, &mut watch, powered).gear.state);
        let mut eye = Eye::default();
        let (turn, slack) = figure(game, &mut eye, how);
        comes_out_level(game, &mut eye, &what);
        let turned = wrap(heading(&game.body().gear.state) - before - turn);
        assert!(turned.abs() < slack, "{what}: {turned} rad off");
        println!("loops: {what} done, camera {:?}", eye.worst);
        game.release(game.bound(|keys| keys.sprint));
    }
}

/// In first person the back key loops and the roll keys roll the body round, and the view
/// follows without a snap, over the top too, and ends upright.
#[test]
fn a_held_key_loops_and_rolls_in_first_person() {
    loops_and_rolls("loops-first", ViewMode::First);
}

/// In third person the same, and the chase camera follows round without a snap.
#[test]
fn a_held_key_loops_and_rolls_in_third_person() {
    loops_and_rolls("loops-third", ViewMode::Third);
}
