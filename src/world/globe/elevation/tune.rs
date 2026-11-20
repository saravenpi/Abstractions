//! Tuning of the large shape of the ground. Heights and lengths are in metres on a planet large enough to
//! carry them in full, and shrink with the planet as the scales of its relief say.

/// Rock strength over rock density in square metres per second squared: the height of the highest
/// mountain a planet can carry times its gravity. Everest, 8.8 km under 9.81 m/s².
pub const STRENGTH: f64 = 86_300.0;

/// Lowest and highest summit of the highest range of a planet, in metres, whatever its gravity.
pub const PEAK: (f64, f64) = (3000.0, 9000.0);

/// Depth of the ocean floor in metres on a planet with no sea, and on one that is all sea.
pub const DEPTH: (f64, f64) = (3000.0, 6000.0);

/// Mean rise over run of a range across its width: a little over 9 km of height for the 255 km of
/// [`crate::world::globe::plates::RANGE`] on an Earth. A smaller planet has narrower ranges, so lower ones.
pub const SLOPE: f64 = 0.036;

/// Width of a range in metres under which the plates add nothing, and from which they count in full. A
/// range narrower than 2 km is smaller than the hills of the local ground, 650 m across, that would
/// have to carry it: such a world is shaped by the local ground alone.
pub const LOCAL: (f32, f32) = (2000.0, 16_000.0);

/// Radius in metres of the planet the continents of the local ground were tuned on.
pub const TUNED_RADIUS: f64 = 6000.0;

/// Height in metres of the inside of a continent above the sea: the old shields of the Earth stand 300
/// to 500 m high.
pub const PLAIN: f32 = 400.0;

/// Height in metres of the shelf break, where the continental shelf ends and the slope begins.
pub const SHELF: f32 = -130.0;

/// Metres of large ground height for one unit of the coast measure of the local ground: within this
/// height of the sea the local ground draws the coast, with its beaches, cliffs, bars and reefs.
pub const FRINGE: f32 = 12.0;

/// How many times steeper than the plates give it the crust blend is read: the shelf, the slope and the
/// rise of a continental margin then take about 190 km on each side of the shelf break on an Earth.
pub const SHARPEN: f32 = 6.5;

/// The waves that move the shore off the plate edges: cycles per radian and height in units of the crust
/// blend. The first moves a coast by up to 320 km on an Earth, the second by 130 km.
pub const COAST: [(f64, f32); 2] = [(1.6, 0.25), (5.0, 0.1)];

/// Lowest and highest offset on the crust blend the shore is looked for between. At the lowest the whole
/// planet is ocean floor, and only volcanoes and arcs stand out.
pub const SHORE: (f32, f32) = (-2.0, 3.0);

/// Offset on the crust blend from which the whole planet is continent, and the metres the ground rises for
/// each unit of offset past it: a planet with hardly any sea keeps it in its deepest basins only.
pub const DRY: (f32, f32) = (1.5, 1500.0);

/// Most waves of the regional band.
pub const OCTAVES: usize = 5;

/// How many times shorter each wave of the regional band is than the one before: 400, 133, 44, 15 and
/// 5 km on a planet with room for the longest.
pub const BAND_STEP: f64 = 3.0;

/// Longest wave of the regional band in metres, and the share of the radius it may not pass.
pub const BAND_LONGEST: (f64, f64) = (400_000.0, 0.25);

/// Shortest wave of the regional band in metres: below it the local ground takes over.
pub const BAND_FINEST: f64 = 4000.0;

/// Height in metres of the wave of the regional band that is [`BAND_SIZE`] long.
pub const BAND_HEIGHT: f64 = 300.0;

/// Length in metres of the wave of the regional band the height is given for.
pub const BAND_SIZE: f64 = 100_000.0;

/// Power of the length of a wave its height grows with: the roughness of real ground, between the 0.5 of
/// old plains and the 0.8 of young mountains.
pub const BAND_HURST: f64 = 0.6;

/// Share of its height a range keeps in its valleys: the crests reach the full height.
pub const RANGE_BODY: f32 = 0.55;

/// Crest lines across the width of a range.
pub const RANGE_CRESTS: f64 = 4.0;

/// Lowest and highest inland measure a large planet gives the climate: the span the local ground has
/// from its open sea to the middle of its islands.
pub const INLAND: (f32, f32) = (-0.76, 0.45);
