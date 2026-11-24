//! Underground mineral veins: planet-specific ores discovered by digging below the surface.

use bevy::math::DVec3;

use super::Mineral;
use crate::universe::hash::cell;
use crate::universe::{PlanetClass, PlanetRecord, Water};

/// Lateral spacing of underground ore pockets, in metres.
const VEIN_METRES: f64 = 18.0;

/// Vertical thickness of each geological vein stratum, in metres.
const STRATUM_METRES: f32 = 1.5;

/// Salt mixed with planet seed when sampling underground veins.
const VEIN_SALT: u64 = 0x7E1A_5EED_4D19_u64;

impl Mineral {
    /// Returns true when digging `depth` metres below ground on `record` can hit a vein of `self`.
    pub fn mined_on(self, record: &PlanetRecord, depth: f32) -> bool {
        use Mineral::*;
        use PlanetClass as P;
        if record.class == P::GasGiant {
            return false;
        }
        let wet = record.water == Water::Liquid;
        match self {
            Copper => matches!(record.class, P::Island | P::Desert | P::Barren) && depth >= 0.6,
            Pyrite => matches!(record.class, P::Island | P::Desert | P::Lava) && depth >= 0.8,
            Galena => matches!(record.class, P::Island | P::Barren | P::Ice) && depth >= 1.2,
            Cobaltite => matches!(record.class, P::Ice | P::Barren | P::Ocean) && depth >= 1.6,
            Cinnabar => {
                (record.class == P::Lava
                    || (record.temperature > 310.0
                        && matches!(record.class, P::Desert | P::Island)))
                    && depth >= 1.0
            }
            Amethyst => wet && matches!(record.class, P::Island | P::Ocean) && depth >= 1.5,
            Ilmenite => matches!(record.class, P::Barren | P::Lava | P::Desert) && depth >= 1.8,
            Cryolite => (record.class == P::Ice || record.water == Water::Ice) && depth >= 0.7,
            Basalt | Flint | Mineral::Ice => self.lies_on(record),
            _ => self.lies_on(record) && depth >= 0.9,
        }
    }

    /// Returns a short guide phrase telling the player where this mineral is found.
    pub fn where_found(self) -> &'static str {
        use Mineral::*;
        match self {
            Basalt => "Surface & dig on any solid world",
            Obsidian => "Surface on volcanic or air worlds",
            Flint => "Surface & shallow dig on wet worlds",
            Ice => "Surface & dig on frozen worlds",
            Quartz => "Dry, cold, or barren rock",
            Jade => "Warm island or ocean coasts",
            Ironstone => "Desert, lava, or barren worlds",
            Malachite => "Desert & dry island outcrops",
            Brimstone => "Volcanic & superheated worlds",
            Siderite => "Barren, ice, or thin-air craters",
            Copper => "Dig >= 0.6m on island, desert, or barren",
            Pyrite => "Dig >= 0.8m on island, desert, or lava",
            Galena => "Dig >= 1.2m on island, barren, or ice",
            Cobaltite => "Dig >= 1.6m on ice/barren/ocean worlds",
            Cinnabar => "Dig >= 1.0m on lava & hot worlds",
            Amethyst => "Dig >= 1.5m on liquid-water worlds",
            Ilmenite => "Dig >= 1.8m on barren/lava/desert",
            Cryolite => "Dig >= 0.7m on ice worlds",
        }
    }
}

/// Returns the underground mineral vein struck at world position `pos_m` and `depth` metres on the
/// planet of `(record, seed)`, or nothing on a gas giant.
pub fn mined_vein(record: &PlanetRecord, seed: u64, pos_m: DVec3, depth: f32) -> Option<Mineral> {
    let pool: Vec<Mineral> = Mineral::ALL
        .into_iter()
        .filter(|m| m.mined_on(record, depth))
        .collect();
    if pool.is_empty() {
        return None;
    }
    let layer = (depth / STRATUM_METRES).floor() as i32;
    let q = (pos_m / VEIN_METRES).floor();
    let coord = bevy::math::IVec3::new(
        (q.x as i64).rem_euclid(1_000_000) as i32,
        layer + (q.y as i64).rem_euclid(1_000_000) as i32,
        (q.z as i64).rem_euclid(1_000_000) as i32,
    );
    let pick = cell(seed ^ VEIN_SALT, coord) as usize;
    Some(pool[pick % pool.len()])
}
