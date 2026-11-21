//! Tuning constants of the aerial perspective.

use bevy::math::Vec3;

/// Share of red, green and blue light the aerosols of clear home air take per metre at sea
/// level. With the gas it leaves a clear day a sight of about 54 km in green light at the 2 percent
/// contrast of [`CONTRAST`]: tens of kilometres, as on a clear day on the Earth. Shorter waves
/// are taken a little more, as particles near the size of the wave do. Unit: per metre.
pub const AEROSOL: Vec3 = Vec3::new(4.8e-5, 6.0e-5, 7.6e-5);

/// Scale height of the aerosols as a share of the scale height of the gas: 1.2 km against 8 km
/// on the Earth (Hillaire 2020, table 1). A share of 1.
pub const AEROSOL_SCALE: f32 = 0.15;

/// Scale height of the air of the Earth, for a camera that has no planet yet. Unit: metres.
pub const HOME_SCALE: f32 = 8000.0;

/// The optical depth at which a dark thing is lost against the sky: `-ln(0.02)`, the 2 percent
/// contrast of the law of Koschmieder. A sight in metres is this over the extinction. Unit: none.
pub const CONTRAST: f32 = 3.912;

/// Share of the light of a far thing below which it is counted as hidden, where the view ends.
/// A share of 1.
pub const LEFT: f32 = 0.01;

/// Span of a path in scale heights under which its two ends count as level. Unit: scale heights.
pub const FLAT: f32 = 1.0e-3;
