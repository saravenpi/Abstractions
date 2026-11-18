//! What a planet sounds like: how loud its air is, how hard its wind blows and what breaks on
//! its shore.

use crate::universe::{PlanetClass, PlanetRecord, Water};
use crate::wind::thickness;

/// What is heard where the liquid of a planet meets its land.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surf {
    /// Nothing: a dry world, a frozen sea, or water that only exists as steam.
    Silent,
    /// Waves of water washing ashore.
    Waves,
    /// The low rumble of molten rock.
    Molten,
}

/// Returns what the shore of a planet sounds like.
///
/// A world with no liquid on its surface has no shore to hear, and a gas giant has no surface
/// at all. Molten rock rumbles on a lava world whatever its water does. Anywhere else only
/// liquid water makes waves: ice, steam and no water are silent.
pub fn surf(record: &PlanetRecord) -> Surf {
    let liquid = record.ocean > 0.0;
    match (liquid, record.class, record.water) {
        (false, _, _) | (_, PlanetClass::GasGiant, _) => Surf::Silent,
        (true, PlanetClass::Lava, _) => Surf::Molten,
        (true, _, Water::Liquid) => Surf::Waves,
        (true, _, _) => Surf::Silent,
    }
}

/// Returns how loud the air of a planet is against the home planet, from 0 without air to 1 in
/// the home air or thicker. Wind, the rush of air past a flying body and the whoosh of clouds
/// are multiplied by it; engines, boots, footsteps and the interface are not.
pub fn loudness(record: &PlanetRecord) -> f32 {
    thickness(record).min(1.0) as f32
}

/// Least wind strength heard in the deep air of a gas giant, which is never still.
pub const GIANT_WIND: f32 = 0.6;

/// Returns the wind strength heard on a planet for a wind of `strength`: none without air, at
/// least [`GIANT_WIND`] on a gas giant, and the wind itself anywhere else.
pub fn wind(record: &PlanetRecord, strength: f32) -> f32 {
    match record.class {
        _ if loudness(record) <= 0.0 => 0.0,
        PlanetClass::GasGiant => strength.max(GIANT_WIND),
        _ => strength,
    }
}

/// Returns the number that carries a planet class to the audio thread.
pub fn code(class: PlanetClass) -> f32 {
    f32::from(class as u8)
}

/// Returns the planet class a [`code`] stands for, the home class for anything else.
pub fn class(code: f32) -> PlanetClass {
    let known = |class: &&PlanetClass| (code - self::code(**class)).abs() < 0.5;
    *PlanetClass::ALL
        .iter()
        .find(known)
        .unwrap_or(&PlanetClass::Island)
}
