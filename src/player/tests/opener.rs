//! The keys that open the wings: the jump pressed again in the air spreads them whatever else
//! is held, a run held since the ground never fires the boots on them, and every change of the
//! wings is told as a [`Spread`] message.

use bevy::prelude::*;

use super::wings::{hold, lift, world};
use crate::gear::Mode;
use crate::gear::tune::BODY_RATE;
use crate::player::chase::{CARRY, WingView, carried};
use crate::player::{Player, SLING, Spread, Squared};
use crate::sim::Intent;

/// The forward key held.
const AHEAD: Intent = Intent::Move {
    forward: 1.0,
    strafe: 0.0,
};

/// The back key held.
const BACK: Intent = Intent::Move {
    forward: -1.0,
    strafe: 0.0,
};

/// The view seen from behind the body, looking half a turn round.
const BEHIND: Intent = Intent::Look {
    yaw: std::f32::consts::PI,
    pitch: 0.0,
    turn: 0.0,
    behind: true,
};

/// A sprint off a ledge, the run still held, and the jump pressed again in the air: the wings
/// spread into a glide and the boots stay off, so the run that carried the body over the edge
/// never turns the glide into powered flight. The run pressed again on the wings boosts.
#[test]
fn a_sprint_off_a_ledge_glides_and_a_new_run_press_boosts() {
    let mut app = world();
    hold(&mut app, 30, &[AHEAD, Intent::Boost]);
    lift(&mut app);
    let (over, _, fired) = hold(&mut app, 6, &[AHEAD, Intent::Boost]);
    assert!(!over.grounded && !over.wings && !fired, "{over:?}");
    hold(&mut app, 2, &[AHEAD, Intent::Boost, Intent::Jump]);
    let (glide, gear, fired) = hold(&mut app, 30, &[AHEAD, Intent::Boost]);
    assert!(glide.wings && !glide.boots && !fired, "{glide:?}");
    assert_eq!(gear.state.mode, Mode::Glide);
    hold(&mut app, 1, &[AHEAD]);
    let (fly, gear, _) = hold(&mut app, 10, &[AHEAD, Intent::Boost]);
    assert!(fly.wings && fly.boots, "{fly:?}");
    assert_eq!(gear.state.mode, Mode::Fly);
}

/// Returns the [`Spread`] messages sent since the last call.
fn told(app: &mut App) -> Vec<Spread> {
    app.world_mut()
        .resource_mut::<Messages<Spread>>()
        .drain()
        .collect()
}

/// The press that opens the wings tells it once, with the body, on that frame; the press that
/// folds them tells that once too; nothing is told while they stay as they are.
#[test]
fn every_change_of_the_wings_is_told_once() {
    let mut app = world();
    app.add_message::<Spread>();
    lift(&mut app);
    hold(&mut app, 6, &[]);
    assert!(told(&mut app).is_empty());
    let (open, _, _) = hold(&mut app, 1, &[Intent::Jump]);
    let world = app.world_mut();
    let mut players = world.query_filtered::<Entity, With<Player>>();
    let entity = players.iter(world).next().expect("the body");
    assert!(open.wings);
    assert_eq!(
        told(&mut app),
        vec![Spread {
            body: entity,
            open: true
        }]
    );
    hold(&mut app, 20, &[]);
    assert!(told(&mut app).is_empty());
    hold(&mut app, 1, &[Intent::Jump]);
    assert_eq!(
        told(&mut app),
        vec![Spread {
            body: entity,
            open: false
        }]
    );
}

/// Returns the seconds from the jump pressed again in the air to the first frame the gear spreads
/// the wings, with a tool `drawn` in the hands or not. Every frame after the press asks them open.
fn first_spread(drawn: bool) -> f32 {
    let mut app = world();
    app.insert_resource(Squared {
        drawn,
        striking: false,
    });
    lift(&mut app);
    hold(&mut app, 6, &[]);
    let (open, _, _) = hold(&mut app, 1, &[Intent::Jump]);
    assert!(open.wings, "not asked open");
    (1..30_u32)
        .find(|_| hold(&mut app, 1, &[]).1.state.wings > 0.0)
        .map_or(f32::INFINITY, |frame| frame as f32 / 30.0)
}

/// With a tool in the hands the jump pressed again in the air asks the wings open at once, the
/// tool is slung meanwhile, and the gear spreads them only once [`SLING`] has run: not a frame
/// of spread before it. With empty hands they spread from the press.
#[test]
fn a_drawn_tool_is_slung_before_the_wings_spread() {
    let slung = first_spread(true);
    assert!((SLING - 0.01..SLING + 0.07).contains(&slung), "{slung} s");
    let bare = first_spread(false);
    assert!(bare < 0.04, "{bare} s");
}

/// Raises the body 190 m, keeping how it moves.
fn raise(app: &mut App) {
    let world = app.world_mut();
    let mut bodies = world.query_filtered::<&mut Transform, With<Player>>();
    for mut transform in bodies.iter_mut(world) {
        transform.translation.y += 190.0;
    }
}

/// A sprint toward the camera, off an edge, the wings spread with the jump and the boots fired
/// with a new press of the run: the body flies on the way it ran, never turns more than the
/// body may in a frame, and never goes over backward, and the view is carried by a few tenths of
/// a radian a frame at most.
#[test]
fn a_boosted_launch_toward_the_camera_never_flips_back() {
    let mut app = world();
    hold(&mut app, 30, &[BEHIND, BACK, Intent::Boost]);
    raise(&mut app);
    hold(&mut app, 6, &[]);
    hold(&mut app, 1, &[Intent::Jump]);
    let mut last = hold(&mut app, 1, &[Intent::Boost]).1.state;
    let most = BODY_RATE / 30.0 + 1e-3;
    for frame in 0..60 {
        let (motion, gear, _) = hold(&mut app, 1, &[Intent::Boost]);
        let state = gear.state;
        let turned = state.attitude.angle_between(last.attitude);
        let what = format!(
            "frame {frame}: turned {turned}, pitch {}, {motion:?}",
            state.pitch
        );
        assert!(turned <= most, "{what}");
        assert!(
            state.pitch.cos() > 0.0 && (state.frame() * Vec3::Y).y > 0.0,
            "{what}"
        );
        last = state;
    }
    assert_eq!(last.mode, Mode::Fly);
}

/// However far the reading of the body jumps in a frame, as the heading of a body falling
/// straight down, the view is carried by no more than [`CARRY`] radians of heading and of pitch.
#[test]
fn the_view_is_never_whipped_round() {
    let mut wing = WingView::default();
    carried(&mut wing, Vec2::new(0.0, -1.5), Vec2::ZERO);
    let turn = carried(&mut wing, Vec2::new(2.5, -0.4), Vec2::ZERO);
    assert!(
        turn.x.abs() <= CARRY + 1e-6 && turn.y.abs() <= CARRY + 1e-6,
        "{turn}"
    );
    let small = carried(&mut wing, Vec2::new(2.55, -0.42), Vec2::ZERO);
    assert!((small - Vec2::new(0.05, -0.02)).length() < 1e-4, "{small}");
}
