//! Tests of what the prompt line says about the wingsuit: the key that spreads it in a fall,
//! and the sentence that says why it cannot glide in air too thin.

use bevy::prelude::KeyCode;

use super::context::Context;
use super::glide::{OPEN_HEIGHT, SPREAD, THIN, spoken};
use crate::gear::{Gear, GearInput};
use crate::player::Motion;
use crate::universe::PlanetRecord;
use crate::wind::thickness;

/// Returns how thick the air of a temperate planet under `pressure` bar is against home.
fn air(pressure: f64) -> f32 {
    thickness(&PlanetRecord {
        pressure,
        ..PlanetRecord::TEMPERATE
    }) as f32
}

/// Returns a body in the air `clearance` metres over the ground, its wings asked open or not.
fn falling(wings: bool, clearance: f32) -> (Motion, Gear) {
    let motion = Motion {
        wings,
        ..Motion::default()
    };
    let gear = Gear {
        input: GearInput {
            clearance,
            ..GearInput::default()
        },
        ..Gear::default()
    };
    (motion, gear)
}

/// Wings asked open on a world of 0.02 bar: the line says the air is too thin, in the tone of
/// a warning with no key. At home, at the thinnest air a home may have and in thick air it says
/// nothing while gliding.
#[test]
fn thin_air_says_why_the_wings_do_not_glide() {
    let (motion, gear) = falling(true, 300.0);
    let thin = spoken(
        Some((&motion, &gear)),
        air(0.02),
        KeyCode::Space,
        Context::Gliding,
    );
    assert_eq!(thin, (Some(THIN.to_string()), None));
    for pressure in [0.5, 1.0, 3.0] {
        let said = spoken(
            Some((&motion, &gear)),
            air(pressure),
            KeyCode::Space,
            Context::Gliding,
        );
        assert_eq!(said, (None, None), "{pressure} bar");
    }
}

/// Falling with the wings folded higher than a jump rises, the line shows the jump key and
/// "Spread the wings", whatever key the jump is bound to, the ground out of reach included.
#[test]
fn a_fall_shows_the_key_that_spreads_the_wings() {
    let want = |key: KeyCode| (None, Some((key, SPREAD.to_string())));
    for (clearance, key) in [(40.0, KeyCode::Space), (f32::INFINITY, KeyCode::KeyJ)] {
        let (motion, gear) = falling(false, clearance);
        let said = spoken(Some((&motion, &gear)), 1.0, key, Context::OnFoot);
        assert_eq!(said, want(key), "{clearance} m");
    }
}

/// Lower than [`OPEN_HEIGHT`], on the ground, in thin air, in the ship and without a body, the
/// line says nothing of the wings.
#[test]
fn no_key_shows_where_the_wings_would_not_glide() {
    let none = (None, None);
    for (what, grounded, clearance, thick, context) in [
        ("low", false, OPEN_HEIGHT - 1.0, 1.0, Context::OnFoot),
        ("thin", false, 40.0, air(0.02), Context::OnFoot),
        ("ship", false, 40.0, 1.0, Context::ShipLanded),
        ("standing", true, 40.0, 1.0, Context::OnFoot),
    ] {
        let (motion, gear) = falling(false, clearance);
        let motion = Motion { grounded, ..motion };
        let said = spoken(Some((&motion, &gear)), thick, KeyCode::Space, context);
        assert_eq!(said, none, "{what}");
    }
    assert_eq!(spoken(None, 1.0, KeyCode::Space, Context::OnFoot), none);
}
