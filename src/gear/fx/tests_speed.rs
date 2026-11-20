//! Tests of the speed cues of the flight effects: buffeting and the condensing vortex.

use super::air::felt;
use super::scene::Scene;
use super::tests_fx::scene;
use super::trails::{TIP_SPEED, tip_heat};
use crate::gear::Mode;

/// The wingtips trail nothing under their speed threshold, dry or wet.
#[test]
fn wingtip_trails_have_no_length_under_their_threshold() {
    let slow = TIP_SPEED.0 - 1.0;
    let wet = Scene {
        moist: 1.0,
        ..scene(Mode::Glide, slow, 1.0)
    };
    assert_eq!(tip_heat(&scene(Mode::Glide, slow, 1.0)), 0.0);
    assert_eq!(tip_heat(&wet), 0.0);
}

/// The vortex condenses in wet air and not in dry air, in vacuum or on a bare planet.
#[test]
fn the_vortex_condenses_only_in_wet_air() {
    let dry = scene(Mode::Glide, 40.0, 1.0);
    let wet = Scene { moist: 1.0, ..dry };
    let airless = Scene {
        thin: 1.0,
        moist: 1.0,
        ..dry
    };
    assert!(tip_heat(&dry) > 0.0 && tip_heat(&wet) > 1.5 * tip_heat(&dry));
    assert_eq!(tip_heat(&airless), 0.0);
}

/// The buffeting felt follows its target smoothly and dies out to exactly zero.
#[test]
fn buffeting_eases_in_and_out() {
    let up = felt(0.0, 1.0, 1.0 / 60.0);
    assert!(up > 0.0 && up < 0.1);
    let mut level = 1.0;
    for _ in 0..600 {
        level = felt(level, 0.0, 1.0 / 60.0);
    }
    assert_eq!(level, 0.0);
}
