//! Every number of the drainage grid worth tuning.

/// Cells along one side of a cube face of the drainage grid on a planet big enough for it: about 40 km on
/// an Earth. A small planet takes one cell per atlas cell instead.
pub const SIDE: i32 = 256;

/// Fewest land cells draining through a cell for it to carry a river, in cells: about 10,000 square
/// kilometres on an Earth.
pub const RIVER: u32 = 8;

/// Metres between two springs along a trunk river, so that the flows of neighbouring springs overlap.
pub const STATION: f64 = 2000.0;

/// Candidate bases the shore search returns, at most.
pub const STARTS: usize = 8;

/// Lowest and highest latitude in degrees, north or south, a start is taken from: mild land, off the poles
/// and the tropics.
pub const LATITUDE: (f64, f64) = (20.0, 48.0);

/// Fewest drainage cells between two starts.
pub const APART: f64 = 6.0;

/// Share of its score a river mouth takes off a start: mouths are sheltered, wet and flat.
pub const MOUTH: f32 = 0.35;

/// Bisection steps that narrow the shore between a land cell and a sea cell, 2^-14 of their distance.
pub const BISECT: u32 = 14;
