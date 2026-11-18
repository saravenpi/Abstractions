//! What the record of a planet lets drift in its air: nothing without air, water only where there is water.

use super::kind::{Ambient, Kind};
use super::pick::pick;
use crate::universe::{NO_AIR, PlanetClass, PlanetRecord};
use crate::world::SEA_LEVEL;

/// Returns true when the planet has too little air to carry a particle or a wing.
pub fn airless(record: &PlanetRecord) -> bool {
    record.pressure < NO_AIR
}

/// Returns true when no creature lives on the planet: no air, molten rock or bare rock.
pub fn lifeless(record: &PlanetRecord) -> bool {
    airless(record) || matches!(record.class, PlanetClass::Lava | PlanetClass::Barren)
}

/// Returns the surroundings as the rules of home must read them: on a planet without a sea the camera is
/// never under it, however low the dry ground lies.
pub fn seen(record: &PlanetRecord, ambient: &Ambient) -> Ambient {
    let dry = record.ocean <= 0.0;
    Ambient {
        height: [ambient.height, ambient.height.max(SEA_LEVEL)][usize::from(dry)],
        ..*ambient
    }
}

/// Returns true when a planet of `class` has `kind` in its air or its liquid: spray, bubbles and plankton
/// need water, and snow does not fall on the hot classes.
fn carried(class: PlanetClass, kind: Kind) -> bool {
    let water = matches!(
        class,
        PlanetClass::Ocean | PlanetClass::Island | PlanetClass::Desert
    );
    let cold = matches!(
        class,
        PlanetClass::Ocean | PlanetClass::Island | PlanetClass::Ice
    );
    match kind {
        Kind::Spray | Kind::Bubble | Kind::Plankton => water,
        Kind::Snow => cold,
        _ => true,
    }
}

/// Picks the kind of a respawning particle on a planet of `record` from the surroundings and a roll in
/// `[0, 1)`. The home planet gets exactly what [`pick`] gives.
pub fn drift(record: &PlanetRecord, ambient: &Ambient, roll: f32) -> Kind {
    if airless(record) {
        return Kind::Hidden;
    }
    let kind = pick(&seen(record, ambient), roll);
    match carried(record.class, kind) {
        true => kind,
        false => Kind::Hidden,
    }
}

#[cfg(test)]
mod tests;
