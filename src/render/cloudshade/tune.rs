//! Tuning constants of the cloud shadows. The shader constants of the same names in
//! `daylight.wesl` hold the same numbers, and a test keeps them equal.

/// Texels along one side of the shadow patch.
pub const SIDE: usize = 32;

/// Half the side of the patch in metres: it covers 8 km around the camera, so a texel is 250 m,
/// about half a cell of the puffs of the clouds, which are 450 m. Unit: metres.
pub const REACH: f32 = 4000.0;

/// Seconds between two works of the patch. The clouds drift 5 m/s and the camera crosses the
/// patch in minutes on foot, so a second keeps up; in flight the patch fades at its edge.
/// Unit: seconds.
pub const PERIOD: f32 = 1.0;

/// Optical depth of a full puff for the direct beam. The beam that gets through is
/// `exp(-depth)`: 0.135 for 2, which is what a thin cloud deck leaves of the sun. A share of 1
/// stands for no light. Unit: none.
pub const OPTICAL_DEPTH: f32 = 2.0;

/// Sines of the height of the sun between which the shadows come on: none below the first,
/// where the beam crosses many clouds and the air takes the light anyway, all above the second.
pub const SUN_EDGE: (f32, f32) = (0.03, 0.15);

/// Texels of the block that hold the header: the planet axes in three, the centre of the patch,
/// and its two axes.
pub const HEAD: usize = 6;

/// Texels the patch adds to the light block: the header and the grid.
pub const TEXELS: usize = HEAD + SIDE * SIDE;

/// Share of the half side of the patch, counted from its centre, past which the shadows fade out
/// toward the edge, so no line shows where the patch ends. A share of 1.
pub const EDGE_FADE: f32 = 0.75;
