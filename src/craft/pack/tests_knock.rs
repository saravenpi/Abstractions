//! Tests of a knock on a rock: the blow that reaches it cracks it with the sound of struck stone,
//! throws off the piece as a stone of its own mass, and the prompt line says what was picked up.

use bevy::prelude::*;

use super::notice::Notice;
use super::pick::{Knocked, crack, piece, striker};
use super::tune::HANDFUL;
use crate::audio::Outer;
use crate::craft::build::Place;
use crate::craft::matter::{Mineral, Stuff};
use crate::craft::wield::Held;
use crate::world::rocks::{Plant, Rock, RockId};

/// Returns the rock of the test, where the blow reached it.
fn knocked() -> Knocked {
    let rock = Rock {
        id: RockId(7),
        plant: Plant::Rock,
        scale: 1.0,
        at: Vec3::ZERO,
        mid: Vec3::Y * 0.45,
        left: 1.0,
    };
    Knocked {
        rock,
        at: Vec3::new(1.0, 0.5, -2.0),
    }
}

/// The blow that reaches a rock cracks it with the sound of stone struck by the fist, throws off
/// the piece as a stone of its own mass, and the prompt line says what was picked up.
#[test]
fn the_blow_cracks_the_rock_and_says_what_it_gave() {
    let knocked = knocked();
    let place = Place {
        gravity: 9.81,
        air: 1.225,
    };
    let (fist, joules) = striker(&Held::default(), &place);
    assert!(joules > 0.0 && fist.stiffness > 10.0);
    let sound = crack(Mineral::Flint, &knocked, (fist, joules));
    assert_eq!(
        (sound.outer, sound.at, sound.cut),
        (Outer::Ground, knocked.at, false)
    );
    assert_eq!(sound.struck.stiffness, Mineral::Flint.material().stiffness);
    let thrown = piece(Mineral::Flint, &knocked, (HANDFUL, Vec3::Y));
    let mass = 4.0 / 3.0
        * std::f32::consts::PI
        * thrown.radius.powi(3)
        * Mineral::Flint.material().density;
    assert!(
        (mass - HANDFUL).abs() < 1e-3 && thrown.velocity.y > 0.0,
        "{thrown:?}"
    );
    let notice = Notice::of(&Stuff::Mineral(Mineral::Flint), 1.24);
    assert_eq!(notice.shown(), Some("Flint 1.2 kg"));
}
