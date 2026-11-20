//! Tuning constants of the metered exposure.

/// Illuminance in lux of a full day under the home star, in the game scale: the sun at its
/// zenith plus the day sky. At it the camera keeps the reference exposure the whole game was
/// tuned at. Unit: lux.
pub const REFERENCE_LUX: f32 = 13_000.0;

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
