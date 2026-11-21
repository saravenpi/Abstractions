//! Tuning constants of the metered exposure.

/// Illuminance in lux of a full day under the home star, in the game scale: the sun at its
/// zenith, 12,000, plus pi times the day sky of 650. At it the camera keeps the reference exposure
/// the whole game was tuned at. Unit: lux.
pub const REFERENCE_LUX: f32 = 14_000.0;

/// Stops of exposure the camera moves per stop of light that changes. A real eye would follow
/// every stop; the game already compresses its night to about a fortieth of its day, and the
/// owner tuned it dark but readable, so the camera follows a third of the way. Unit: stops per stop.
pub const FOLLOW: f32 = 0.35;

/// Most stops the exposure brightens the picture over the reference, so a moonless night stays
/// dark. Unit: stops of EV100.
pub const BRIGHTEN_LIMIT: f32 = 1.0;

/// Most stops the exposure darkens the picture under the reference, for light brighter than the
/// reference day. Unit: stops of EV100.
pub const DARKEN_LIMIT: f32 = 0.5;

/// Time constant in seconds of the camera going toward a darker exposure, when the scene gets
/// brighter: the eye closes quickly. Unit: seconds.
pub const TO_BRIGHT: f32 = 1.5;

/// Time constant in seconds of the camera going toward a brighter exposure, when the scene
/// gets darker: the eye opens slowly. Unit: seconds.
pub const TO_DARK: f32 = 5.0;

/// Change in EV100 below which the camera is not written again. Unit: stops.
pub const STEP: f32 = 0.01;

/// Time constant in seconds of the white balance: chromatic adaptation is 90 percent complete
/// after about a minute (Fairchild and Reniff, 1995), which is 26 seconds of time constant.
/// Unit: seconds.
pub const ADAPT_SECONDS: f32 = 26.0;

/// Reflectance of the field the eye adapts to: the mean grey of a land surface. Unit: none.
pub const MEAN_ALBEDO: f32 = 0.2;

/// The CAT02 factor `F` for an average surround. Unit: none.
pub const SURROUND: f32 = 1.0;

/// Change in the chromaticity of the white below which the camera is not written again.
pub const WHITE_STEP: f32 = 1.0e-4;

/// The linear colour of the light of a G star, the Sun. The game draws the daylight of a G star as
/// its neutral light, so the balance is the identity under it and a redder or bluer star is
/// measured against it. Unit: linear RGB.
pub const SUN_LIGHT: bevy::math::Vec3 = bevy::math::Vec3::new(1.0, 0.9117, 0.7454);
