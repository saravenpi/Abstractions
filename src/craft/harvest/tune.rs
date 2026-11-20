//! Tuning constants of harvesting: how much of a part comes away, by the tool that takes it.

/// Share of a part's box that is the part itself, unitless: a rounded part fills about half its box.
pub const FILL: f32 = 0.5;

/// Share of a part's mass that is its skin, unitless.
pub const SKIN: f32 = 0.1;

/// Share of the yield the right tool takes when its edge is no harder than the tissue, unitless.
pub const BLUNT: f32 = 0.5;

/// Share of the yield the wrong tool, or bare hands, leaves whole, unitless.
pub const RUINED: f32 = 0.25;
