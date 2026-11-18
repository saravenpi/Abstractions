//! The local weather as the sky draws it: how much cloud, how wet and how dusty the air is
//! where the camera stands. Without a weather report the sky keeps the look of the planet's
//! class, so the sky of a planet with no weather is the sky it had before weather existed.

use bevy::prelude::*;

use super::air::Air;
use super::palette::Palette;
use super::scatter::luminance;

/// Colour of lifted dust against the light, in sRGB: a pale ochre.
const DUST_RGB: Vec3 = Vec3::new(0.78, 0.63, 0.44);

/// Share of the sky colour full dust replaces with its own at the same brightness.
const DUST_SKY: f32 = 0.85;

/// Share of the cloud colour full dust stains.
const DUST_CLOUD: f32 = 0.5;

/// Share of their light a full storm takes from the clouds.
const RAIN_DARK: f32 = 0.55;

/// Share of the sky colour a full storm turns grey at the same brightness.
const RAIN_GREY: f32 = 0.5;

/// Share of its height a full storm takes from the bottom of the cloud layer.
const RAIN_LOWER: f32 = 0.35;

/// How many more times the fog closes in under full dust.
const DUST_FOG: f32 = 3.0;

/// How many more times the fog closes in under a full storm.
const RAIN_FOG: f32 = 1.0;

/// What the weather at the camera does to the sky, each value from 0 to 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeatherLook {
    /// Share of the sky under cloud on average, which the cloud cover of the day drifts around.
    pub cover: f32,
    /// Rain and storm: dark, low clouds and a grey, closer horizon.
    pub haze: f32,
    /// Lifted dust: an ochre sky and a fog that closes in, in the colour of the dust.
    pub dust: f32,
}

impl WeatherLook {
    /// Returns calm weather under the mean cloud of `air`, the look of the planet's class.
    pub fn of(air: &Air) -> Self {
        Self {
            cover: air.clouds,
            haze: 0.0,
            dust: 0.0,
        }
    }

    /// Returns the cloud cover the shader draws when the cover of the day drifts to `drift`
    /// under `air`: the class cover of that air with this weather's cover as its mean.
    pub fn cover(&self, air: &Air, drift: f32) -> f32 {
        let weathered = Air {
            clouds: self.cover,
            ..*air
        };
        weathered.cover(drift)
    }

    /// Returns the height in metres of the bottom of the cloud layer, lowered by rain from `base`.
    pub fn base(&self, base: f32) -> f32 {
        base * (1.0 - RAIN_LOWER * self.haze)
    }

    /// Returns the share of the clear sight the fog still leaves under this weather.
    pub fn sight(&self) -> f32 {
        1.0 / (1.0 + DUST_FOG * self.dust + RAIN_FOG * self.haze)
    }

    /// Returns `palette` under this weather: dust stains the sky and the clouds, rain greys the
    /// sky and darkens the clouds. Calm weather returns it untouched.
    pub fn sky(&self, mut palette: Palette) -> Palette {
        if self.haze <= 0.0 && self.dust <= 0.0 {
            return palette;
        }
        let rain = self.haze.clamp(0.0, 1.0);
        let dust = self.dust.clamp(0.0, 1.0);
        palette.zenith = stained(palette.zenith, rain, dust);
        palette.horizon = stained(palette.horizon, rain, dust);
        let cloud = palette.cloud * (1.0 - RAIN_DARK * rain);
        palette.cloud = cloud.lerp(alike(cloud, DUST_RGB), dust * DUST_CLOUD);
        palette
    }
}

/// Returns the colour `color` would have with the hue of `hue` at its own brightness.
pub(super) fn alike(color: Vec3, hue: Vec3) -> Vec3 {
    hue * (luminance(color) / luminance(hue))
}

/// Returns a sky colour greyed by `rain` and stained by `dust`, both from 0 to 1.
fn stained(color: Vec3, rain: f32, dust: f32) -> Vec3 {
    let grey = color.lerp(Vec3::splat(luminance(color)), rain * RAIN_GREY);
    grey.lerp(alike(grey, DUST_RGB), dust * DUST_SKY)
}

/// The weather reported at the camera, if anything reports it.
///
/// Empty, the sky keeps the class look of the planet below.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct LocalWeather(pub Option<WeatherLook>);

impl LocalWeather {
    /// Returns the weather to draw over a planet of `air`.
    pub fn look(&self, air: &Air) -> WeatherLook {
        self.0.unwrap_or(WeatherLook::of(air))
    }
}
