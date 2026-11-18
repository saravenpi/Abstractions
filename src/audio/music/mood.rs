//! The mood a planet class gives its piece, so that an ice world does not sound like a lava world.

use super::mode::Mode;
use crate::universe::PlanetClass;

/// How a planet class bends the piece drawn from the planet seed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mood {
    /// The mode the class imposes, or none to keep the one the seed drew.
    pub mode: Option<Mode>,
    /// What the midday cutoff of the pad is multiplied by: under 1 is darker, over 1 brighter.
    pub bright: f32,
}

/// Returns the mood of a planet class.
///
/// An island world, the class of the home planet, keeps its piece exactly as drawn. Ocean is a
/// little brighter in whatever mode it has. Desert is cool and dry, ice bright and floating,
/// lava dark, and a barren world plays a scale with no semitone, with the lid half closed.
pub fn mood(class: PlanetClass) -> Mood {
    let (mode, bright) = match class {
        PlanetClass::Island => (None, 1.0),
        PlanetClass::Ocean => (None, 1.15),
        PlanetClass::Desert => (Some(Mode::Dorian), 0.85),
        PlanetClass::Ice => (Some(Mode::Lydian), 1.3),
        PlanetClass::Lava => (Some(Mode::Aeolian), 0.6),
        PlanetClass::Barren => (Some(Mode::Pentatonic), 0.7),
        PlanetClass::GasGiant => (Some(Mode::Dorian), 0.75),
    };
    Mood { mode, bright }
}
