//! Tuning of the water mirror.

/// Farthest distance in metres from the mirror camera at which a mirrored actor is drawn, the far
/// plane of the mirror camera, and where the mirror image has faded out on the water. A robot of
/// 2 m is under two game pixels tall there.
pub const REACH: f32 = 300.0;

/// Distance in metres from the camera of the water at which the mirror image starts to fade.
pub const FADE: f32 = 150.0;

/// Height in metres of the camera above the water past which no actor is mirrored: from there
/// the mirror image of anything near the water is a few pixels.
pub const CEILING: f32 = 300.0;

/// Fewest texels of the sea floor map around the focus, 2 m squares, that must hold water for the
/// mirror to draw: a 256 m square with less shows no water worth a pass.
pub const WET: u32 = 4;

/// Least share of the sky a liquid mirrors for the pass to draw: water mirrors all of it, the
/// rough crust of molten rock a tenth.
pub const CLEAR: f32 = 0.5;

/// Metres the mirror image moves on the water per unit of slope of the waves and the ripples.
pub const WOBBLE: f32 = 2.5;
