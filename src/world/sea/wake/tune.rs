//! Tuning constants of the wake waves: how long they last, how fast a source must move to leave a
//! wedge, and how strongly the sea answers its push.

/// Entries of the ring buffer of recent wake sources.
pub const WAKES: usize = 32;

/// Seconds a wake entry shows. `WAKE_LIFE` in `field.wesl` holds the same value.
///
/// A moving source lays one entry per [`PERIOD`], so a lone source fills 30 of the [`WAKES`]
/// before its oldest entry expires.
pub const LIFE: f32 = 3.0;

/// Seconds between two entries of a moving source.
pub const PERIOD: f32 = 0.1;

/// Speed in metres per second above which a source leaves a wedge and below which it only rings
/// the water. `WAKE_MOVING` in `field.wesl` holds the same value.
pub const MOVING: f32 = 2.0;

/// Half angle in radians of the Kelvin wedge of a slow source, `asin(1/3)`, 19.47 degrees.
pub const KELVIN: f32 = 0.339_836_9;

/// Least half angle in radians the arms of a wake are drawn at, 4 degrees: a wedge narrower than
/// this reads as two stripes, not as a V.
pub const ARM_FLOOR: f32 = 0.07;

/// Shortest wavelength in metres a wake wave is given: three cells of the sea mesh, the least it
/// can draw as a height.
pub const MIN_WAVELENGTH: f32 = 6.0;

/// Wavelength in metres of the ripple rings of a hovering source. They run out at the phase speed
/// of waves this long under the local gravity.
pub const HOVER_WAVELENGTH: f32 = 6.0;

/// Half width in radians of the arc of a ring that is kept: the part where its waves meet those
/// of the entries before it and build an arm. `WAKE_ARC` in `field.wesl` holds the same value.
pub const ARC: f32 = 0.45;

/// Half width of a wave packet, in wavelengths.
pub const PACKET: f32 = 1.5;

/// Seconds that, times the speed, give the length in metres of the window one entry gives the
/// transverse waves. `WAKE_SPAN` in `field.wesl` holds the same value.
pub const SPAN: f32 = 0.03;

/// Hull Froude numbers between which the transverse waves die out: a fast source leaves arms only.
pub const TRANSVERSE: (f32, f32) = (1.0, 2.5);

/// Fractions of a cell per radian of wave number, between which a wave goes from fully drawn to
/// not drawn at the footprint of a vertex or a pixel. `WAKE_KEEP_START` and `WAKE_KEEP_END` in
/// `field.wesl` hold the same pair.
pub const RESOLVE: (f32, f32) = (0.15, 0.5);

/// Footprint in metres of one vertex of the near sea.
pub const FOOT: f32 = 2.0;

/// Gain between the head of water the air flow of a source piles up and the height of its waves.
/// It is a tuning gain, with no unit.
pub const GAIN: f32 = 4.0;

/// Share of the head of the speed of a source that touches the sea, `v^2 / 2g`, that rises as a wave.
pub const TOUCH_SHARE: f32 = 0.01;

/// Greatest height in metres of a wake wave.
pub const MAX_AMPLITUDE: f32 = 0.5;

/// Height in metres under which a wake wave is not worth an entry.
pub const MIN_AMPLITUDE: f32 = 0.002;
