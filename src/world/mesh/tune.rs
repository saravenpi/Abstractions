//! Tuning constants of the terrain meshes.

/// Widest cell in metres whose triangles are toned apart, each by a stable factor from 0.97 to 1.03. The
/// factors average 1, so a toned level and an untoned one show the same mean colour. Coarser cells are
/// wider than the grain the tone stands for, and carry the colour alone.
pub const TONE_REACH: f32 = 4.0;

/// Finest cell of the terrain in metres: the grid of a near chunk. No cell is read at a scale under it.
pub const FINE: f32 = 2.0;

/// Widest scale in metres at which the slope of the ground is read: cells this wide or wider are classed by
/// the slope of the land over this distance, as `Globe::glance` reads it, so the far mesh, the ball and the
/// map agree. Narrower cells are classed by the slope of their own facet, read from the heights the mesh
/// already holds, which costs no height sample.
pub const COARSE: f32 = 125.0;

/// Slope angle in degrees from the horizontal above which ground is rock, the `ROCK_SLOPE` of 1 in
/// `surface.rs`.
pub const ROCK_ANGLE: f32 = 45.0;

/// Width in degrees of the band around [`ROCK_ANGLE`] over which a coarse cell turns from ground to rock by
/// a smooth step, so a hillside of small cliffs reads as a rock-tinted slope from afar and not as grass.
pub const ROCK_BAND: f32 = 20.0;

/// Levels above the leaves from which a coarse node has half and a quarter of the cells per side.
/// A node of the quadtree lies 2 to 4.8 of its widths from the eye, so a cell of 32 per side
/// covers 2 to 4 pixels of the 360 rows of the game image whatever the level. Four levels up a
/// node is about 1 km wide and 2 km away or more, where the air has taken a third of its colour,
/// and gets 16 cells of 3.6 to 9 pixels; seven levels up, 16 km away or more, the air has taken
/// nine tenths, and it gets 8. Unit: quadtree levels.
pub const SPARSE_LEVELS: (u8, u8) = (4, 7);
