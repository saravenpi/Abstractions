//! Crouching: slower walking, lower eyes, no sprint, and no crouch while swimming, gliding, on
//! the boots or in the air.

use bevy::prelude::*;

use super::wings::{hold, world};
use crate::gear::{FlightState, Gear, Mode};
use crate::player::Player;
use crate::player::controller::Motion;
use crate::player::crouch::{allowed, drop, pace};
use crate::player::{BODY_LENGTH, BODY_RADIUS};
use crate::sim::Intent;

/// Walks straight ahead.
const AHEAD: Intent = Intent::Move {
    forward: 1.0,
    strafe: 0.0,
};

/// Returns the horizontal speed of `motion`.
fn pace_of(motion: &Motion) -> f32 {
    motion.velocity.with_y(0.0).length()
}

/// A crouched body walks at about 0.45 of the walking speed and does not sprint.
#[test]
fn a_crouched_body_walks_slowly_and_does_not_sprint() {
    let walk = pace_of(&hold(&mut world(), 60, &[AHEAD]).0);
    let (crouched, _, _) = hold(&mut world(), 60, &[AHEAD, Intent::Crouch]);
    let (sprint, _, _) = hold(&mut world(), 60, &[AHEAD, Intent::Crouch, Intent::Boost]);
    assert!(walk > 4.0, "walks at {walk}");
    assert!(
        (pace_of(&crouched) / walk - 0.45).abs() < 0.05,
        "{crouched:?}"
    );
    assert!((pace_of(&sprint) - pace_of(&crouched)).abs() < 0.1);
}

/// The eyes go down by about a third of the standing height in a quarter of a second, by steps,
/// and come back up when the key is let go.
#[test]
fn the_eyes_lower_smoothly_and_rise_again() {
    let mut app = world();
    let (early, _, _) = hold(&mut app, 3, &[Intent::Crouch]);
    assert!(early.crouch > 0.0 && early.crouch < 1.0, "{}", early.crouch);
    let (down, _, _) = hold(&mut app, 6, &[Intent::Crouch]);
    let height = BODY_LENGTH + 2.0 * BODY_RADIUS;
    assert_eq!(down.crouch, 1.0);
    assert!((drop(down.crouch) / height - 0.35).abs() < 1e-3);
    assert!(drop(early.crouch) < drop(down.crouch));
    let (up, _, _) = hold(&mut app, 10, &[]);
    assert_eq!(up.crouch, 0.0);
    assert_eq!(pace(0.0), 1.0);
}

/// Swimming, gliding, the boots and a fall allow no crouch.
#[test]
fn a_body_cannot_crouch_while_swimming_or_gliding() {
    let standing = Motion {
        grounded: true,
        ..default()
    };
    let with = |mode: Mode| Gear {
        state: FlightState {
            mode,
            ..FlightState::default()
        },
        ..Gear::default()
    };
    assert!(allowed(&standing, &with(Mode::Off)));
    let swimming = Motion {
        swimming: true,
        ..standing
    };
    assert!(!allowed(&swimming, &with(Mode::Off)));
    for mode in [
        Mode::Glide,
        Mode::Fly,
        Mode::BootFall,
        Mode::Fall,
        Mode::Slide,
    ] {
        assert!(!allowed(&standing, &with(mode)), "{mode:?}");
    }
    assert!(!allowed(&Motion::default(), &with(Mode::Off)));
}

/// A body high in the air asked to crouch stays up.
#[test]
fn a_falling_body_stays_up() {
    let mut app = world();
    let mut bodies = app
        .world_mut()
        .query_filtered::<&mut Transform, With<Player>>();
    for mut transform in bodies.iter_mut(app.world_mut()) {
        transform.translation.y += 190.0;
    }
    let (motion, _, _) = hold(&mut app, 10, &[Intent::Crouch]);
    assert_eq!(motion.crouch, 0.0);
}
