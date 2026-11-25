//! Six-tier mining hardness progression: mineral tiers, pickaxe tiers, and underground depth gates.

use super::crush_scale;
use crate::craft::build::Intent;
use crate::craft::matter::Mineral;
use crate::craft::wield::Held;

/// Returns the minimum pickaxe tier (`0..=5`) required to mine `mineral`.
pub fn mineral_tier(mineral: Mineral) -> u8 {
    match mineral {
        Mineral::Ice
        | Mineral::Brimstone
        | Mineral::Galena
        | Mineral::Cinnabar
        | Mineral::Cryolite
        | Mineral::Copper
        | Mineral::Basalt => 0,
        Mineral::Malachite
        | Mineral::Siderite
        | Mineral::Obsidian
        | Mineral::Ironstone
        | Mineral::Pyrite
        | Mineral::Jade => 1,
        Mineral::Cobaltite
        | Mineral::Ilmenite
        | Mineral::Chromite
        | Mineral::Flint
        | Mineral::Quartz
        | Mineral::Amethyst => 2,
        Mineral::Wolframite | Mineral::Corundum => 3,
        Mineral::Osmium | Mineral::Moissanite => 4,
        Mineral::Lonsdaleite => 5,
    }
}

/// Returns the mining tier (`0..=5`) of `held`: `0` for bare hands or non-`Crush` tools, `1..=5`
/// for `Intent::Crush` pickaxes derived from head `strength` (MPa) and `hardness` (Mohs).
pub fn tool_tier(held: &Held) -> u8 {
    let Some(hold) = held.0.as_ref() else {
        return 0;
    };
    if hold.intent != Intent::Crush {
        return 0;
    }
    let m = hold.build.head.material;
    if m.strength >= 630.0 || m.hardness >= 9.4 {
        5
    } else if m.strength >= 470.0 || m.hardness >= 7.4 {
        4
    } else if m.strength >= 300.0 || m.hardness >= 6.8 {
        3
    } else if m.strength >= 180.0 || m.hardness >= 6.2 {
        2
    } else {
        1
    }
}

/// Returns the maximum underground dig depth in meters reachable with a `tier` pickaxe.
pub fn dig_depth_limit(tier: u8) -> f32 {
    match tier {
        0 => 1.2,
        1 => 2.5,
        2 => 4.5,
        3 => 6.5,
        4 => 8.5,
        _ => f32::INFINITY,
    }
}

/// Returns the HUD display name for a required pickaxe `tier`.
pub fn tier_name(tier: u8) -> &'static str {
    match tier {
        0 => "Bare Hand",
        1 => "Tier 1 Pickaxe (Starter)",
        2 => "Tier 2 Pickaxe (Iron)",
        3 => "Tier 3 Pickaxe (Titanium/Crystal)",
        4 => "Tier 4 Pickaxe (Tungsten/Corundum)",
        _ => "Tier 5 Pickaxe (Apex Endgame)",
    }
}

/// Returns quarry yield multiplier: starter Basalt yields `0.45x` bare-handed to avoid soft-locks.
pub fn quarry_scale(held: &Held, stone: Mineral) -> f32 {
    if stone == Mineral::Basalt && tool_tier(held) == 0 {
        0.45
    } else {
        crush_scale(held)
    }
}
