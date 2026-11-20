//! Tuning of the blades near the camera.

/// Width in chart metres of one square tile of blades: four cells of the terrain grid.
pub const TILE: f32 = 8.0;

/// Blades per square metre of fully covered ground within the inner ring.
pub const DENSE: f32 = 50.0;

/// Blades per square metre of fully covered ground between the rings: the share `EDGE` of
/// `blades.wesl` of the dense blades.
pub const SPARSE: f32 = 8.0;

/// Radius in metres of the inner ring, inside which the blades stand dense: the `NEAR` of `blades.wesl`.
pub const NEAR: f32 = 15.0;

/// Radius in metres of the outer ring, past which no blade stands: the `FAR` of `blades.wesl`.
pub const FAR: f32 = 40.0;

/// Metres past a ring a tile keeps its blades before it is rebuilt or dropped, so a camera that paces a
/// border builds nothing.
pub const LINGER: f32 = 4.0;

/// Most tiles built in one frame.
pub const BUILDS: usize = 2;

/// Height in metres of the camera above the ground over which no blade is drawn.
pub const ABOVE: f32 = 30.0;

/// Metres of bare ground left around a trunk, a stem or a rock.
pub const ROOM: f32 = 0.2;
