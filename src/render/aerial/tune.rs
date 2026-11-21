//! Tuning constants of the aerial perspective.

use bevy::math::Vec3;

/// Share of red, green and blue light the aerosols of dry home air take per metre at sea level.
/// With the gas it leaves an ordinary day a sight of 23 km in green light at the 2 percent contrast
/// of [`CONTRAST`], the standard rural air of the aerosol models: a ridge 2 km away has lost three
/// tenths of its colour to the sky and one 10 km away four fifths. Air as clear as 50 km is rare
/// near the ground. Shorter waves are taken a little more, as the inverse of the wavelength.
/// Unit: per metre.
pub const AEROSOL: Vec3 = Vec3::new(1.34e-4, 1.58e-4, 1.93e-4);

/// How many more times than clean air the aerosols take the light in air as dusty as air gets:
/// the dust that stains the sky of a dry world hazes its distance too. Unit: none.
pub const DUST_AIR: f32 = 1.0;

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
