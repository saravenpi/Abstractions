//! Tuning constants of the rocks: when a quarried rock is drawn again, and where a stone comes to rest.

use super::Mineral;
use crate::world::propmesh::Plant;

/// Share of its size a quarried rock must lose before its chunk draws it smaller, unitless: a rebuild
/// of the props of a chunk is not worth a smaller rock than that.
pub const SHRINK_STEP: f32 = 0.05;

/// Mass in kilograms under which what is left of a rock counts as nothing: the rock is gone.
pub const LEAST: f32 = 1.0e-3;

/// Angle of repose of every stone that rests on the ground, in degrees: the steepest slope it lies on
/// without rolling or sliding off. Blocky basalt and crystals hold steepest, rounded nodules and slick
/// ice least; rock on rock sits near 35 degrees.
pub const REPOSE: [(Plant, f32); 13] = [
    (Plant::Rock, 38.0),
    (Plant::Quartz, 37.0),
    (Plant::Obsidian, 36.0),
    (Plant::Ironstone, 36.0),
    (Plant::Brimstone, 35.0),
    (Plant::Boulder, 34.0),
    (Plant::Jade, 34.0),
    (Plant::Sandstone, 33.0),
    (Plant::Malachite, 33.0),
    (Plant::Flint, 32.0),
    (Plant::Siderite, 32.0),
    (Plant::IceRock, 28.0),
    (Plant::SmoothRock, 28.0),
];

/// Radius of the ring of ground a stone is weighed against, in metres: three cells of the finest mesh.
pub const RING: f32 = 6.0;

/// Height in metres a spot may stand over the mean of its ring and still hold a stone: past it the
/// spot is a crest, a ridge or a peak a stone would roll off.
pub const CREST: f32 = 0.6;

/// Most stones a hollow holds against flat ground, unitless: a hollow as deep under its ring as
/// [`CREST`] gathers this many times the stones of flat ground.
pub const HOLLOW: f32 = 2.0;

/// Step of the difference that measures the slope under a stone on the legacy plane, in metres.
pub const STEP: f32 = 1.0;

/// The hard stones a plain rock of the tables can be, each with its share where they all lie: basalt
/// most, then flint, then obsidian, plus planetary minerals where the record allows them.
pub const LODES: [(Mineral, f32); 9] = [
    (Mineral::Basalt, 0.5),
    (Mineral::Flint, 0.3),
    (Mineral::Obsidian, 0.2),
    (Mineral::Quartz, 0.28),
    (Mineral::Jade, 0.16),
    (Mineral::Ironstone, 0.24),
    (Mineral::Malachite, 0.14),
    (Mineral::Brimstone, 0.18),
    (Mineral::Siderite, 0.12),
];

/// Share of the pick of a hard stone that is drawn at random rather than read from the lode, unitless:
/// a field of rocks is mostly one stone with a few others in it.
pub const LODE_MIX: f32 = 0.25;

/// Frequency of the lode field, per metre: one field of a stone is about a hundred metres across.
pub const LODE_FREQ: f64 = 0.012;
