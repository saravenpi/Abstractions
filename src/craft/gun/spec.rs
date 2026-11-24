//! Firearm data specifications: flintlock pistol and mineral rifle state kept in the pack.

use serde::{Deserialize, Serialize};

use crate::craft::matter::Mineral;

/// Ballistic and handling profile of one firearm kind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GunStats {
    /// Rounds held in chamber or magazine.
    pub capacity: u8,
    /// True when the firearm is gripped with both hands.
    pub two_handed: bool,
    /// Muzzle velocity in metres per second.
    pub speed: f32,
    /// Base damage per direct hit before mineral bullet modifiers.
    pub damage: f32,
    /// Seconds between shots in the magazine.
    pub cadence: f32,
    /// Seconds to reload from the pack.
    pub reload: f32,
    /// Empty mass in kilograms before barrel density scaling.
    pub base_mass: f32,
}

/// Kind of craftable firearm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GunKind {
    /// Starter one-handed single-shot mineral flintlock pistol.
    FlintlockPistol,
    /// Two-handed four-round bolt-action mineral rifle with crystal sights.
    MineralRifle,
}

impl GunKind {
    /// Returns the ballistic and physical stats of `self`.
    pub fn stats(self) -> GunStats {
        match self {
            GunKind::FlintlockPistol => GunStats {
                capacity: 1,
                two_handed: false,
                speed: 110.0,
                damage: 38.0,
                cadence: 0.45,
                reload: 1.3,
                base_mass: 1.35,
            },
            GunKind::MineralRifle => GunStats {
                capacity: 4,
                two_handed: true,
                speed: 185.0,
                damage: 72.0,
                cadence: 0.55,
                reload: 1.9,
                base_mass: 3.10,
            },
        }
    }
}

/// One crafted firearm in the player's pack or on the ground.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gun {
    /// Pack-unique id matching crafted tool ids.
    pub id: u32,
    /// Pistol or rifle chassis.
    pub kind: GunKind,
    /// Primary mineral forged into the barrel.
    pub barrel: Mineral,
    /// Rounds currently chambered/loaded (`0..=kind.stats().capacity`).
    pub loaded: u8,
    /// Mineral of the currently loaded rounds, if any.
    pub ammo: Option<Mineral>,
    /// Cumulative wear from 0 (new) to 1 (fouled/worn).
    pub wear: f32,
}

impl Gun {
    /// Returns a freshly crafted unloaded firearm.
    pub fn new(id: u32, kind: GunKind, barrel: Mineral) -> Self {
        Self {
            id,
            kind,
            barrel,
            loaded: 0,
            ammo: None,
            wear: 0.0,
        }
    }

    /// Returns total mass in kilograms scaled mildly by barrel density.
    pub fn mass(&self) -> f32 {
        let ratio = (self.barrel.material().density / 3000.0).clamp(0.7, 1.45);
        self.kind.stats().base_mass * (0.7 + 0.3 * ratio)
    }

    /// Returns a human-readable item name.
    pub fn name(&self) -> String {
        let m = match self.barrel {
            Mineral::Basalt => "basalt",
            Mineral::Flint => "flint",
            Mineral::Ironstone => "ironstone",
            Mineral::Siderite => "siderite",
            Mineral::Copper => "copper",
            Mineral::Cobaltite => "cobaltite",
            Mineral::Ilmenite => "ilmenite",
            Mineral::Pyrite => "pyrite",
            Mineral::Galena => "galena",
            _ => "mineral",
        };
        match self.kind {
            GunKind::FlintlockPistol => format!("{m} pistol"),
            GunKind::MineralRifle => format!("{m} rifle"),
        }
    }
}
