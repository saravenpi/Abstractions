//! Tuning of the plates. Every angle is in radians on the unit sphere, so a width in metres is the angle
//! times the radius of the planet. Every share is of the height its comment names.

/// Fewest and most plates a planet has.
pub const COUNT: (usize, usize) = (8, 20);

/// Chord between two neighbour seeds over the square root of the area of one plate, on a hexagonal
/// lattice: `sqrt(2 / sqrt(3))`.
pub const SPACING: f64 = 1.0746;

/// How far a seed is moved off its lattice point, as a share of the spacing between seeds.
pub const JITTER: f64 = 0.3;

/// Radians a direction is displaced by before its plate is looked up, which bends the plate edges.
pub const WARP: f64 = 0.07;

/// Cycles of the warp noise per radian: one bend of an edge is about 0.6 radians long.
pub const WARP_GRAIN: f64 = 1.6;

/// Radians over which the crust blend changes from one plate to the next on each side of their edge. It
/// is wide, 1,270 km on an Earth, so that the shore has room to lie off the plate edge on either side, as
/// the passive margins of the Atlantic do.
pub const MARGIN: f64 = 0.2;

/// Radians from a converging edge within which the ground is lifted: 255 km on an Earth, where the
/// Himalaya and the Andes are 200 to 350 km wide.
pub const RANGE: f64 = 0.04;

/// Radians from a spreading edge within which the young sea floor stands high: about 950 km on an Earth,
/// the half width of the Mid-Atlantic Ridge.
pub const RIDGE: f64 = 0.15;

/// Radians past the cell of either plate of a pair where their edge has stopped shaping the ground.
pub const JUNCTION: f64 = 0.02;

/// Closing speed, as a share of the fastest plate, under which an edge neither closes nor opens, and
/// the one from which it lifts or spreads in full.
pub const CLOSING: (f32, f32) = (0.08, 0.5);

/// Share of its height an old range keeps on a planet whose plates have stopped: the Urals and the
/// Appalachians against the Himalaya.
pub const FOSSIL: f32 = 0.25;

/// Share of the plate motion a planet keeps without liquid water to weaken its crust: the stagnant lids of
/// Venus and Mars.
pub const DRY_LID: f32 = 0.25;

/// Height of a coastal range over a sinking sea plate, as a share of the highest mountains.
pub const CORDILLERA: f32 = 0.8;

/// Depth of a trench under the sea floor around, as a share of the ocean depth: the trenches of the
/// Earth reach 8 to 11 km in oceans 4 to 5 km deep.
pub const TRENCH: f32 = 0.6;

/// Height of an island arc over the sea floor, as a share of the ocean depth: just past 1, so the crest of
/// the arc stands out of the sea.
pub const ARC: f32 = 1.08;

/// Height of a spreading ridge over the old sea floor, as a share of the ocean depth: crests near 2.5 km
/// deep in oceans 4.5 km deep.
pub const RIDGE_RISE: f32 = 0.45;

/// Depth of a rift valley where a continent parts, as a share of the highest mountains, and its half
/// width as a share of [`RANGE`]: the East African Rift is about 1 km deep and 50 km wide.
pub const RIFT: (f32, f32) = (0.12, 0.2);

/// Billions of years a planet of one Earth mass keeps enough heat to move its plates. The time grows as the
/// square root of the mass: Mars (0.107) gets 3.3, the Moon (0.012) 1.1, and both are dead today.
pub const HEAT_GYR: f64 = 10.0;

/// Mass of the Earth in kilograms.
pub const EARTH_MASS: f64 = 5.972e24;

/// Kelvin from which the heat of the surface itself keeps the crust moving, and from which it is molten.
pub const MOLTEN: (f32, f32) = (700.0, 1500.0);

/// Youngest and oldest planet in billions of years, for a globe that was given no age.
pub const AGE_GYR: (f64, f64) = (0.5, 10.0);

/// Lowest and highest share of low ground on a world without a sea, picked by its seed.
pub const LOWLAND: (f64, f64) = (0.3, 0.7);

/// Billions of years after which a surface nothing renews is covered in impact basins.
pub const SCAR_GYR: f64 = 4.0;

/// Share of the basins that a thick air, rain and wind wear away.
pub const AIR_SHIELD: f32 = 0.7;

/// Most impact basins on a planet.
pub const BASINS: f32 = 24.0;

/// Smallest and widest basin, as the angle from its centre to its rim in radians: 100 to 1,300 km across
/// on an Earth.
pub const BASIN: (f64, f64) = (0.008, 0.1);

/// Depth of a wide crater in metres for a width of 1 km, and the power of the width it grows with: the
/// complex craters of the Moon measured by Pike (1977), `depth = 1.044 * width^0.301` in kilometres.
pub const BASIN_DEPTH: (f64, f64) = (1044.0, 0.301);

/// Most volcanoes over hot spots on a planet.
pub const CONES: f32 = 12.0;

/// Narrowest and widest volcano, as the angle from its summit to its foot in radians: 150 to 380 km
/// across on an Earth, Hawaii being about 200.
pub const CONE: (f64, f64) = (0.012, 0.03);

/// Lowest and highest volcano, as a share of the highest mountains.
pub const CONE_HEIGHT: (f32, f32) = (0.35, 0.9);

/// Share of its height a volcano loses on a planet whose plates move at full speed, since the plate carries
/// it off its hot spot before it has grown: Hawaii against Olympus Mons.
pub const CONE_DRIFT: f32 = 0.4;
