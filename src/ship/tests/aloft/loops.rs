//! No loops on the wings in a whole game: a pitch key held down, through the real keys, the
//! view rig and the gear, from an ordinary glide and from powered flight, in both views. The key
//! asks for an attitude, so the body holds the climb or the dive limit and never goes round.

use bevy::prelude::*;

use super::watch::Watch;
use super::{Body, Game};
use crate::gear::Mode;
use crate::gear::tune::{CLIMB_LIMIT, DIVE_LIMIT};
use crate::player::wrap;
use crate::space::{Anchor, up};
use crate::view::{CameraPose, Heading, ViewMode, ViewPrefs};

/// Height above the sea the flights start at, in metres: low enough for the full grip of the
/// air, high enough to hold a dive.
const START: f32 = 520.0;

/// Frames a flight is left alone to settle before the key is held, at 30 frames per second.
const SETTLE: usize = 150;

/// Frames the key is held: long enough to come all the way round, were loops still possible.
const HELD: usize = 120;

/// Margins in radians the nose and the flight path may run past the climb and dive limits: the
/// path runs a little under the nose, by the angle of attack.
const MARGIN: (f32, f32) = (0.02, 0.15);

/// Largest move of the camera relative to the body in one frame, in metres, and its largest
/// turn in radians: a view that follows smoothly stays far under both, a snap or a flip does not.
const SMOOTH: (f32, f32) = (1.2, 0.35);

/// The camera of the frame before, relative to the body, and the worst frame seen.
///
/// The limbs of the drawn body are not checked here as the stages of the flight check them:
/// the trunk joint of the avatar stops at its limits and cannot show a body on its back.
#[derive(Debug, Default)]
struct Eye {
    /// Offset from the body and rotation of the camera on the last frame.
    last: Option<(Vec3, Quat)>,
    /// Largest move and turn in one frame so far.
    worst: (f32, f32),
}

impl Eye {
    /// Runs one frame and checks that the camera neither jumped nor snapped round.
    fn step(&mut self, game: &mut Game) {
        game.step(1);
        let (pose, body) = (game.get::<CameraPose>(), game.body());
        let now = (pose.position - body.at, pose.rotation);
        if let Some(last) = self.last.replace(now) {
            let (moved, turned) = (now.0.distance(last.0), now.1.angle_between(last.1));
            self.worst = (self.worst.0.max(moved), self.worst.1.max(turned));
            let pitch = body.gear.state.pitch;
            assert!(
                moved < SMOOTH.0,
                "the camera jumped {moved} m at {pitch} rad"
            );
            assert!(
                turned < SMOOTH.1,
                "the camera snapped {turned} rad at {pitch} rad"
            );
        }
    }
}

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

/// Holds `key` for [`HELD`] frames and checks every frame that the nose and the flight path
/// stay inside the climb and dive limits, the back of the body faces the sky and the wings
/// stay spread.
fn holds(game: &mut Game, eye: &mut Eye, key: KeyCode) {
    game.press(key);
    for frame in 0..HELD {
        eye.step(game);
        let state = game.body().gear.state;
        let (v, pitch) = (state.velocity, state.pitch);
        let path = v.y.atan2(v.with_y(0.0).length());
        let inside =
            |angle: f32, margin: f32| (DIVE_LIMIT - margin..=CLIMB_LIMIT + margin).contains(&angle);
        assert!(
            inside(pitch, MARGIN.0),
            "frame {frame}: the nose at {pitch} rad"
        );
        assert!(
            inside(path, MARGIN.1),
            "frame {frame}: the path at {path} rad"
        );
        let back = (state.frame() * Vec3::Y).y;
        assert!(back > 0.3, "frame {frame}: the back faces {back}");
        assert!(state.mode.winged(), "frame {frame}: the wings folded");
    }
    game.release(key);
}

/// Frames a flight may take after the key is let go to fly nose first where the view looks.
const RECOVERY: usize = 300;

/// Returns how far the nose of the body is from where the view looks, around the vertical,
/// and how far from the way the body moves through the air.
fn astray(game: &mut Game) -> (f32, f32) {
    let state = game.body().gear.state;
    let view = game.get::<Heading>().yaw;
    (wrap(state.servo.yaw - view).abs(), state.aoa.abs())
}

/// Checks that the flight comes out of the held key the way it went in: on its wings, the right way
/// up, flying nose first where the view looks, with the camera upright behind it.
fn comes_out_level(game: &mut Game, eye: &mut Eye, what: &str) {
    let mut frames = 0;
    while frames < 75 || astray(game).0 > 0.3 || astray(game).1 > 0.6 {
        eye.step(game);
        frames += 1;
        assert!(
            frames < RECOVERY,
            "{what}: still astray by {:?}",
            astray(game)
        );
    }
    let body = game.body();
    let state = body.gear.state;
    let vertical = up(&game.get::<Anchor>(), body.at);
    let roof = game.get::<CameraPose>().rotation * Vec3::Y;
    assert!(
        roof.dot(vertical) > 0.5,
        "{what}: the camera ends on its side"
    );
    let view = game.get::<Heading>().pitch;
    assert!(view.abs() < 1.2, "{what}: the view ends at {view} rad");
    assert!(
        wrap(state.pitch).abs() < 1.2 && state.mode.winged(),
        "{what}"
    );
    assert!(body.height > 30.0, "{what}: ended at {} m", body.height);
}

/// Holds the back key and the forward key, from a glide and from powered flight, in the view
/// `mode`, and checks that the heading is where it was.
fn holds_both_ways(tag: &str, mode: ViewMode) {
    let mut game = Game::full(tag);
    let game = &mut game;
    game.start("Loops", None);
    game.until("the player to stand", 300, |game| game.walker().grounded);
    game.app.world_mut().resource_mut::<ViewPrefs>().on_foot = mode;
    let mut watch = Watch::default();
    let keys = [
        (game.bound(|keys| keys.back), "climbing"),
        (game.bound(|keys| keys.forward), "diving"),
    ];
    for powered in [false, true] {
        for (key, how) in keys {
            let what = format!("{mode:?}, powered {powered}, {how}");
            let before = takes_wing(game, &mut watch, powered);
            let mut eye = Eye::default();
            holds(game, &mut eye, key);
            comes_out_level(game, &mut eye, &what);
            let turned = wrap(game.body().gear.state.servo.yaw - before.gear.state.servo.yaw);
            assert!(turned.abs() < 0.2, "{what}: {turned} rad off");
            println!("holds: {what}: camera {:?}", eye.worst);
            game.release(game.bound(|keys| keys.sprint));
        }
    }
}

/// In first person a held pitch key holds the climb or the dive limit and never loops, from a
/// plain glide and from powered flight, and the view follows without a snap and ends upright.
#[test]
fn a_held_key_never_loops_in_first_person() {
    holds_both_ways("holds-first", ViewMode::First);
}

/// In third person a held pitch key does the same, and the camera stays behind the body.
#[test]
fn a_held_key_never_loops_in_third_person() {
    holds_both_ways("holds-third", ViewMode::Third);
}
