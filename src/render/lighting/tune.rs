//! Tuning constants of the scene lights.

/// Lowest height of a light above the horizon, as a sine, at which its shadows are cast.
pub const SHADOW_HEIGHT: f32 = 0.02;

/// Distance from the camera in metres at which the first shadow cascade ends, whatever the reach.
/// It holds the robot and the companion in third person: at 12 m one texel of a 2048 map is under
/// 2 cm, so a leg 10 cm wide is five texels and the 5 by 5 filter of the shadow keeps its shape.
/// The first cascade used to end at a quarter of the reach, 28 m: 4.3 cm texels, which the filter
/// smeared over 20 cm, more than a limb, so the robot cast a blob. Unit: metres.
pub const FIRST_BOUND: f32 = 12.0;

/// Share of a cascade that overlaps the next, so the cascades blend. A share of 1.
pub const OVERLAP: f32 = 0.2;

/// Illuminance in lux of the moon below which it casts no shadow: a sixth of a full moon, about
/// what the night ambient light already gives, so a shadow would not read.
pub const MOON_SHADOW_LUX: f32 = 60.0;

/// Illuminance in lux below which the light of a lamp is lost in the ambient light of the night,
/// about a sixtieth of it (the night is about 310 lux), and where its range ends. It is what the
/// 3.5 m range of the lamp of the robot, 850 lm, gives under the inverse square law. Unit: lux.
pub const LOCAL_FLOOR_LUX: f32 = 5.5;
