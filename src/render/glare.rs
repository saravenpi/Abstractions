//! The glare of the star of the system seen from where no air stands in the way: the light the
//! eye and the lens scatter around its disc. It is drawn on the sky as light, and the same glow
//! outshines the stars around the star, so nothing dark ever rings it: a star shows where it is
//! brighter than the glow there. The band of the galaxy is not cut out of the glow but washed out
//! by it: both are light and add, and the band keeps a share of its light that falls smoothly as
//! the glow grows, the same share over its dust lanes and its clumps.
//!
//! The glow falls off as the cube of the angle past a small core, the way the scatter of an eye
//! does, so it fades into black space with no edge. Under the air the dome draws the glow of
//! the air around the star instead, and this one fades out as the air thickens.

/// Luminance in linear light of the glow at the heart of its core, for the star at full light
/// at the reference exposure. Unit: linear light.
#[cfg(test)]
pub const GLOW_PEAK: f32 = 40.0;

/// Angular radius of the core of the glow, past which it falls as the cube of the angle: about
/// two thirds of a degree. Unit: radians.
#[cfg(test)]
pub const GLOW_CORE: f32 = 0.01;

/// Returns the strength of the glow for a star of which the share `lit` of the light reaches the
/// camera, under the share `air` of a home atmosphere over it, at the exposure gain `gain`: the
/// star in full light and in space at the reference exposure gives 1, the air takes it away.
pub fn strength(lit: f32, air: f32, gain: f32) -> f32 {
    lit.clamp(0.0, 1.0) * (1.0 - air.clamp(0.0, 1.0)) * gain.max(0.0)
}

/// Returns the luminance in linear light of the glow of strength `strength` at the cosine `cos`
/// of the angle from the star. `starfield.wesl` and `sky.wesl` have the same rule: the squared
/// chord `2 (1 - cos)` stands for the squared angle, the core keeps the heart finite.
#[cfg(test)]
pub fn glow(cos: f32, strength: f32) -> f32 {
    let spread = (2.0 * (1.0 - cos)).max(0.0) / (GLOW_CORE * GLOW_CORE);
    GLOW_PEAK * strength.max(0.0) / (1.0 + spread).powf(1.5)
}

/// Luminance in linear light of the glow at which a band no brighter than this keeps half its
/// light. The faint and the bright parts of a band under it keep the same share, so its lanes
/// and clumps fade evenly; a brighter part keeps more. Unit: linear light.
#[cfg(test)]
pub const BAND_HOLD: f32 = 0.03;

/// Returns the share of its light the band of the galaxy keeps where it shows at the luminance
/// `light` under the glow `glow`: `hold / (hold + glow)`, with `hold` the brighter of the band
/// and [`BAND_HOLD`]. The glow and the light kept add to more as the glow grows and as the band
/// brightens, by no more than the band's own step, so the band has no notch, no hole and no new
/// edge around the star. `sky.wesl` has the same rule.
#[cfg(test)]
pub fn washed(light: f32, glow: f32) -> f32 {
    let hold = light.max(BAND_HOLD);
    hold / (hold + glow.max(0.0))
}

/// Returns the share of its light a star keeps when it shows at the luminance
/// `light` over the glow `glow`: all of it on black space, nothing once the glow is as bright.
/// The light kept and the glow add to the brighter of the two, which never falls as the glow
/// grows, so nothing grows darker toward the star. `starfield.wesl` has the same rule.
#[cfg(test)]
pub fn outshone(light: f32, glow: f32) -> f32 {
    (1.0 - glow.max(0.0) / light.max(1.0e-6)).clamp(0.0, 1.0)
}
