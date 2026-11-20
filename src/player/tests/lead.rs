//! Where the view points and what the body turns to.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::gear::Gear;
use crate::gear::{FlightState, Mode, Servo};
use crate::player::controller::Motion;
use crate::player::intent::Intent;
use crate::player::lead::*;
use crate::player::steer::Lead;
use crate::view::tune::PITCH_LIMIT;
use crate::view::{Heading, Look, ViewMode};

/// Returns a gear in `mode` whose nose heads `yaw`.
fn gear(mode: Mode, yaw: f32) -> Gear {
    let servo = Servo {
        yaw,
        ..Servo::default()
    };
    Gear {
        state: FlightState {
            mode,
            servo,
            ..FlightState::default()
        },
        ..Gear::default()
    }
}

/// Returns what a player pushing along `wish` asks.
fn pushing(wish: Vec3) -> Intent {
    Intent {
        wish,
        forward: 1.0,
        right: 0.0,
        sprint: false,
        crouch: false,
        jump: false,
        yaw: 0.0,
        steer: false,
    }
}

/// The view is the held angles plus the mouse, clamped short of straight up, and looks
/// down -Z at rest.
#[test]
fn the_view_runs_one_mouse_move_ahead_of_the_rig() {
    let held = Heading {
        yaw: 0.5,
        pitch: 1.5,
    };
    let look = Look {
        yaw_delta: 0.25,
        pitch_delta: 0.5,
        ..Look::default()
    };
    let view = View::ahead(&held, &look, ViewMode::Third);
    assert_eq!(
        (view.yaw, view.pitch, view.third),
        (0.75, PITCH_LIMIT, true)
    );
    let level = View::ahead(&Heading::default(), &Look::default(), ViewMode::First);
    assert!((level.direction() - Vec3::NEG_Z).length() < 1e-6 && !level.third);
}

/// First person follows the view, third person the keys or nothing, a flight its nose and
/// a slide its own way.
#[test]
fn each_movement_has_its_lead() {
    let (still, east) = (pushing(Vec3::ZERO), pushing(Vec3::NEG_X));
    let first = View::ahead(&Heading::default(), &Look::default(), ViewMode::First);
    let third = View {
        third: true,
        ..first
    };
    let motion = Motion {
        velocity: Vec3::new(0.0, 0.0, 9.0),
        ..Motion::default()
    };
    let off = gear(Mode::Off, 2.0);
    assert_eq!(lead(&motion, &off, &east, &first), Lead::View(0.0));
    assert_eq!(lead(&motion, &off, &still, &third), Lead::Hold);
    assert_eq!(lead(&motion, &off, &east, &third), Lead::Travel(FRAC_PI_2));
    let glide = gear(Mode::Glide, 2.0);
    assert_eq!(lead(&motion, &glide, &east, &third), Lead::Nose(2.0));
    let slide = lead(&motion, &gear(Mode::Slide, 2.0), &east, &first);
    assert!(matches!(slide, Lead::Travel(yaw) if (yaw.abs() - std::f32::consts::PI).abs() < 1e-5));
}
