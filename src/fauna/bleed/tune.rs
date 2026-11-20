//! Tuning of wounds and the blood that leaves them. Lengths are metres, volumes cubic metres.

/// Volume of the smallest drop a wound throws, in cubic metres: one millilitre, a sphere of 6 mm.
pub const PARCEL: f32 = 1.0e-6;

/// Most drops a wound throws in a second, whatever its flow: a stronger flow throws bigger drops.
pub const RATE: f32 = 40.0;

/// Most drops a wound throws in one frame.
pub const EMIT: usize = 6;

/// Length of the edge that wets with blood at the moment of a cut, in metres: the blood the cut
/// displaces is its width times its depth times this.
pub const WET: f32 = 0.1;

/// Most drops of the burst of a cut.
pub const BURST: usize = 12;

/// Speed at which a shallow wound lets its blood go, in metres per second: it oozes, not jets.
pub const OOZE: f32 = 0.2;

/// Half the cone the drops of a wound scatter in, in radians.
pub const SCATTER: f32 = 0.15;

/// Share of its flow a wound keeps while a spear plugs it, unitless.
pub const PLUG: f32 = 0.1;

/// Metres from the point of a stuck spear within which it plugs a wound.
pub const PLUG_RANGE: f32 = 0.6;

/// Flow in cubic metres per second under which a wound has closed.
pub const CLOSED: f32 = 1.0e-9;

/// Thickness of the outer layer of a part (hide, shell, plate), in shares of the part's thickness.
pub const OUTER: f32 = 0.05;

/// Thickness of the flesh under the outer layer, in shares of the part's thickness.
pub const FLESH: f32 = 0.3;

/// Thinnest layer, in metres.
pub const THIN: f32 = 0.001;

/// Least and most size of a stain on the ground that a trail drop leaves, in metres.
pub const TRAIL: [f32; 2] = [0.03, 0.6];

/// Share of the energy of a blow that throws chips off a hard layer, unitless.
pub const CHIP_SHARE: f32 = 0.02;

/// Radius of a chip, in metres.
pub const CHIP: f32 = 0.003;

/// Fastest chip, in metres per second.
pub const CHIP_FAST: f32 = 8.0;

/// Most chips a blow throws.
pub const CHIPS: usize = 8;

/// Most wounds an animal keeps open at once.
pub const MOST: usize = 4;

/// Stiffness in gigapascals from which a layer chips instead of giving, as in the hit test.
pub const HARD: f32 = 1.0;
