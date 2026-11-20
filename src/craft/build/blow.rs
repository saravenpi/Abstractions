//! Blow: what a build carries into a target for an intent.

use super::hit::{Blow, Mode};
use super::part::Form;
use super::swing::swing;
use super::throw::throw;
use super::tune::THROW_RANGE;
use super::{Build, Intent, Place};

/// Returns how the head of `build` meets a target: an edge slashes, a point stabs, a block crushes.
fn mode(build: &Build) -> Mode {
    match build.head.form {
        Form::Edge => Mode::Slash,
        Form::Point => Mode::Stab,
        Form::Mass => Mode::Blunt,
    }
}

/// Returns the blow `build` lands for `intent` at `place`.
///
/// Cut and crush swing, so they carry ½Iω². Pierce thrusts, carrying the arm's thrust work. Throw
/// carries the spear's kinetic energy on arrival at the judging range; a spear balanced behind its
/// middle tumbles and lands blunt.
pub fn blow(build: &Build, intent: Intent, place: &Place) -> Blow {
    let head = &build.head;
    let base = Blow {
        energy: swing(build, place).energy,
        mode: mode(build),
        width: head.width(),
        area: head.area(),
        hardness: head.material.hardness,
    };
    match intent {
        Intent::Cut | Intent::Crush => base,
        Intent::Pierce => Blow {
            energy: super::tune::THRUST_ENERGY,
            ..base
        },
        Intent::Throw => {
            let flight = throw(build, place);
            let mode = if flight.stable {
                base.mode
            } else {
                Mode::Blunt
            };
            Blow {
                energy: flight.energy_at(THROW_RANGE),
                mode,
                ..base
            }
        }
    }
}
