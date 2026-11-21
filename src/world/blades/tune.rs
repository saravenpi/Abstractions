//! Tuning of the blades near the camera.

/// Width in chart metres of one square tile of blades: four cells of the terrain grid.
pub const TILE: f32 = 8.0;

/// Blades per square metre of fully covered ground within [`FULL`].
pub const DENSE: f32 = 50.0;

/// Radius in metres within which every blade stands. Past it the share of blades kept falls with the square of
/// the distance, so the blades on screen do not thin out with the distance. The `FULL` of `blades.wesl`.
pub const FULL: f32 = 9.0;

/// Distance in metres from the camera past which no blade stands: the `END` of `blades.wesl`.
pub const END: f32 = 50.0;

/// Metres before [`END`] over which every blade shrinks to nothing: the `FADE` of `blades.wesl`.
pub const FADE: f32 = 12.0;

/// Share of its keep, 0 to 1, over which a blade the distance thins out shrinks away instead of vanishing: the
/// `SOFT` of `blades.wesl`.
pub const SOFT: f32 = 0.3;

/// Height of the farthest blades as a share of the nearest, 0 to 1: the `SHORT` of `blades.wesl`.
pub const SHORT: f32 = 0.6;

/// The highest rank, 0 to 1, of the blades of each band of a tile, the farthest band first. A band holds the
/// blades ranked from the top of the band before it, is built where the distance keeps any of them, and is
/// dropped past that, so a tile the camera nears gains bands and never builds again what it holds.
pub const BANDS: [f32; 4] = [0.08, 0.2, 0.6, 1.0];

/// Metres the landing pad keeps blade roots off beyond its edge: about one blade length, so none leans over it.
pub const PAD_MARGIN: f64 = 1.0;

/// Metres past its reach a band of a tile keeps its blades before it is dropped, so a camera that paces a
/// border builds nothing.
pub const LINGER: f32 = 4.0;

/// Metres the ground near the camera may rise over the ground under it: taken off the height of the camera
/// before that height counts in its distance to a tile, so a hillside above a low flyer keeps its blades.
pub const RELIEF: f32 = 10.0;

/// Metres before its reach a band nearer than the farthest is built, so its first blades never show late: a
/// band arrives where they stand barely grown.
pub const AHEAD: f32 = 1.0;

/// Most bands landed, and most bands started, in one frame: a band finished off-thread waits for its turn. A
/// band holds at most 1,280 blades, so three of them upload fewer than the two whole tiles of 3,200 of 0.11.0.
pub const LANDS: usize = 3;

/// Most bands being built at once.
pub const FLIGHTS: usize = 8;

/// Threads of the pool the bands are built on, apart from the shared compute pool.
pub const WORKERS: usize = 1;

/// Metres added to the sphere that holds the blades of a tile, so a prop is never dropped for a tile whose
/// blades curve a little past the corners.
pub const SLACK: f32 = 1.0;

/// Metres the box a band is culled by reaches past its roots: up for the blade height, sideways for its lean.
pub const RISE: f32 = 3.0;

/// Metres of bare ground left around a trunk, a stem or a rock.
pub const ROOM: f32 = 0.2;
