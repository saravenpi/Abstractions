//! Harvested wood, propellant charges, bullets, and craftable placeables.

use serde::{Deserialize, Serialize};

use super::Material;
use crate::world::rocks::Mineral;

/// Wood and carbonized derivatives harvested from breaking trees and crafted at the bench.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WoodKind {
    /// Raw timber chopped directly from a tree trunk.
    Log,
    /// Milled wooden plank used for chests, stocks, and shafts.
    Plank,
    /// Shaped wooden handle/shaft used for tools and torches.
    Stick,
    /// Carbonized charcoal used to mix propellant powder and light torches.
    Charcoal,
}

impl WoodKind {
    /// Plain lowercase name.
    pub fn name(self) -> &'static str {
        match self {
            WoodKind::Log => "wood log",
            WoodKind::Plank => "wood plank",
            WoodKind::Stick => "stick",
            WoodKind::Charcoal => "charcoal",
        }
    }

    /// Physical mass in kilograms per discrete unit item.
    pub fn unit_mass(self) -> f32 {
        match self {
            WoodKind::Log => 0.40,
            WoodKind::Plank => 0.15,
            WoodKind::Stick => 0.06,
            WoodKind::Charcoal => 0.12,
        }
    }

    /// Mechanical numbers when used in tools or craft recipes.
    pub fn material(self) -> Material {
        let [
            density,
            stiffness,
            strength,
            toughness,
            hardness,
            conductivity,
        ] = match self {
            WoodKind::Log => [600.0, 10.0, 80.0, 5.0, 2.0, 0.15],
            WoodKind::Plank => [620.0, 11.0, 88.0, 5.2, 2.2, 0.15],
            WoodKind::Stick => [640.0, 11.5, 92.0, 5.4, 2.3, 0.15],
            WoodKind::Charcoal => [420.0, 4.0, 12.0, 0.4, 1.5, 0.10],
        };
        Material {
            density,
            stiffness,
            strength,
            toughness,
            hardness,
            conductivity,
        }
    }
}

/// Propellant charge mixed from charcoal and reactive minerals for crafting bullets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChargeKind {
    /// Starter black-powder charge milled from charcoal and flint or pyrite.
    SparkPowder,
    /// High-energy volcanic charge milled from charcoal and brimstone or cinnabar.
    BrimstonePowder,
}

impl ChargeKind {
    /// Plain lowercase name.
    pub fn name(self) -> &'static str {
        match self {
            ChargeKind::SparkPowder => "spark powder",
            ChargeKind::BrimstonePowder => "brimstone powder",
        }
    }

    /// Physical mass in kilograms per powder charge.
    pub fn unit_mass(self) -> f32 {
        0.08
    }

    /// Physical properties placeholder for stack display.
    pub fn material(self) -> Material {
        Material {
            density: 1150.0,
            stiffness: 2.0,
            strength: 8.0,
            toughness: 0.3,
            hardness: 1.5,
            conductivity: 0.2,
        }
    }
}

/// Craftable structures placed onto a planet's ground and shared in multiplayer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlaceableKind {
    /// Shared 24-slot wooden & mineral chest persisted in single-player, LAN, and dedicated servers.
    Chest,
    /// Standing torch that illuminates its surroundings with a warm flame light.
    Torch,
}

impl PlaceableKind {
    /// Plain lowercase name.
    pub fn name(self) -> &'static str {
        match self {
            PlaceableKind::Chest => "chest",
            PlaceableKind::Torch => "torch",
        }
    }

    /// Physical mass in kilograms per item.
    pub fn unit_mass(self) -> f32 {
        match self {
            PlaceableKind::Chest => 2.0,
            PlaceableKind::Torch => 0.25,
        }
    }

    /// Structural material properties for ground drop physics and collision bounds.
    pub fn material(self) -> Material {
        WoodKind::Plank.material()
    }
}

/// Mass in kilograms of one crafted bullet cast from `mineral`.
pub fn bullet_mass(mineral: Mineral) -> f32 {
    (0.02 + mineral.material().density / 1000.0 * 0.004).clamp(0.02, 0.06)
}
