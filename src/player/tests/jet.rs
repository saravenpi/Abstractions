//! The boot water jet in the controller: it dives along the view and skims at the surface.

use bevy::prelude::*;

use crate::gear::{Gear, GearInput};
use crate::player::jet::jet_velocity;
use crate::player::lead::View;
use crate::player::walk::Step;

/// Returns the values of one frame of a body whose centre is `afloat` metres above the sea,
/// looking `pitch` radians up.
fn frame(afloat: f32, pitch: f32) -> Step {
    Step {
        dt: 1.0 / 60.0,
        time: 0.0,
        view: View {
            yaw: 0.0,
            pitch,
            third: false,
        },
        height: afloat,
        afloat,
        gravity: 22.0,
        air: 1.0,
    }
}

/// Returns boots held and spooled under home water.
fn jetting() -> Gear {
    let mut gear = Gear {
        input: GearInput {
            boost: true,
            in_water: true,
            ..GearInput::default()
        },
        ..Gear::default()
    };
    gear.state.thrust = 1.0;
    gear
}

/// Returns the velocity after `seconds` of jetting from rest in the frame `step`.
fn after(gear: &Gear, step: &Step, seconds: f32) -> Vec3 {
    let mut velocity = Vec3::ZERO;
    for _ in 0..(seconds / step.dt).round() as u32 {
        velocity = jet_velocity(velocity, gear, step);
    }
    velocity
}

/// Deep under the surface, looking down dives along the view and looking up climbs.
#[test]
fn the_jet_follows_the_view_under_water() {
    let gear = jetting();
    let down = after(&gear, &frame(-5.0, -0.8), 1.0);
    let up = after(&gear, &frame(-5.0, 0.8), 1.0);
    assert!(down.y < -3.0 && down.z < -3.0, "{down}");
    assert!(up.y > 3.0 && up.z < -3.0, "{up}");
    let ahead = View {
        yaw: 0.0,
        pitch: -0.8,
        third: false,
    }
    .direction();
    assert!(down.normalize().dot(ahead) > 0.99);
}

/// At the surface a level view skims: the body runs fast along the water and floats, it does
/// not climb out of it.
#[test]
fn the_jet_skims_at_the_surface() {
    let gear = jetting();
    let skim = after(&gear, &frame(-0.3, 0.2), 2.0);
    assert!(skim.z < -5.0, "{skim}");
    assert!(skim.y.abs() < 2.5, "{skim}");
}

/// Under a frozen sea the held boots push nothing.
#[test]
fn a_frozen_sea_gives_no_jet() {
    let mut gear = jetting();
    gear.input.liquid = 0.0;
    assert_eq!(after(&gear, &frame(-5.0, 0.0), 1.0), Vec3::ZERO);
}
