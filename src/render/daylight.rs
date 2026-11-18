//! Daylight at a place: how much of the light of the star reaches a point of a planet through
//! its air, and how much of the day sky stands over it.
//!
//! The palette of the camera and the shaders of everything seen from afar read this one rule,
//! so the colour of the sky and the light on the ground cannot disagree. `daylight.wesl` holds
//! the same rule for the GPU: [`daylight`] mirrors its `daylight`, [`Sky::light`] its `light_at`,
//! and the constants shared by both carry the same names.

use bevy::asset::{embedded_asset, load_embedded_asset};
use bevy::prelude::*;

use super::celestial::smoothstep;
use super::scatter::{luminance, peak, transmit};

mod place;
mod track;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_place;

pub use place::{PlaceUniform, Sky};
pub use track::{PlaceLight, PlaceLit, place_light, starlight};

/// Sines of the height of the star between which its disc clears the horizon and its direct
/// light comes on.
pub const SUN_EDGE: (f32, f32) = (-0.06, 0.03);

/// Sines of the height of the star between which the night sky turns into the day sky.
pub const SKY_EDGE: (f32, f32) = (-0.12, 0.25);

/// Sines of the depth of the star under the horizon between which the night is complete enough
/// for the moon and the stars to light the ground.
pub const NIGHT_EDGE: (f32, f32) = (-0.02, 0.3);

/// The light of the star at one point, as shares of what it gives in open space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Daylight {
    /// Colour the air leaves the direct light, in sRGB, its strongest channel at 1: white under
    /// a high white star, red at the terminator.
    pub tint: Vec3,
    /// Share of the brightness of the star the air lets through on its path, from 0 to 1.
    pub through: f32,
    /// Share of the disc of the star above the horizon, from 0 to 1.
    pub risen: f32,
    /// Share of the day sky over the point, from 0 at night to 1 in full day.
    pub sky: f32,
}

/// Returns the daylight at a point where a star of linear colour `star` stands at the sine
/// `height` above the horizon, through air `depth` home atmospheres deep.
///
/// `height` is the height the point sees: the sine of the elevation of the star over the level
/// ground there plus the dip of the horizon at the altitude of the point, so a summit is lit
/// before the valley under it. Neither the direct light nor the sky ever falls as the star climbs.
pub fn daylight(height: f32, star: Vec3, depth: f32) -> Daylight {
    let through = star * transmit(height, depth);
    let full = luminance(star * transmit(1.0, 1.0)).max(1.0e-6);
    Daylight {
        tint: peak(through),
        through: (luminance(through) / full).min(1.0),
        risen: smoothstep(SUN_EDGE.0, SUN_EDGE.1, height),
        sky: smoothstep(SKY_EDGE.0, SKY_EDGE.1, height),
    }
}

/// Keeps the shader module loaded, so the shaders that import it always find it.
#[derive(Resource)]
struct Module {
    /// The loaded module, held so that it is never unloaded.
    _handle: Handle<Shader>,
}

/// Embeds the shader module every lit shader imports as `package::render::daylight`, and works
/// out the light of the planet under the camera once the camera, the star and the moon are
/// tracked. It reads what the sun and the moonlight track, and starts from their defaults in an
/// app that tracks neither. An app without shaders, as in a test, embeds the module unloaded.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "daylight.wesl");
    if app.world().contains_resource::<Assets<Shader>>() {
        let _handle = load_embedded_asset!(app, "daylight.wesl");
        app.insert_resource(Module { _handle });
    }
    app.init_resource::<super::SunDir>()
        .init_resource::<super::Vantage>()
        .init_resource::<super::LocalSystem>()
        .init_resource::<super::moonlight::Moonlight>()
        .init_resource::<PlaceLight>()
        .add_systems(
            PostUpdate,
            place_light
                .in_set(PlaceLit)
                .after(super::sun::track)
                .after(super::moonlight::shine),
        );
}
