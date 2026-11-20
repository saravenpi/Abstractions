//! The ring of a blow by an animal on the robot, as the audio hears it.

use super::kinds::Kind;
use super::tune::PLATE;
use crate::audio::{Body, Impact, Outer};
use crate::avatar::blow::Blow;
use crate::life::Species;

/// Returns the sound of the blow `blow` of an attack of `kind` by an animal of `species` on the
/// plate of the robot: the part that strikes is a tooth, a claw, a horn, a tail or a hoof, with its
/// share of the mass of the animal.
pub fn heard(kind: Kind, species: &Species, blow: &Blow) -> Impact {
    Impact {
        at: blow.at,
        joules: blow.joules,
        cut: blow.cut,
        outer: Outer::RobotPlate,
        struck: Body {
            stiffness: PLATE.0,
            mass: PLATE.1,
        },
        tool: Body {
            stiffness: kind.striker().anchor().stiffness,
            mass: species.history.mass * kind.share(),
        },
        deflected: false,
    }
}
