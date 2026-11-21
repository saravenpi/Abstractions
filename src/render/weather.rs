//! The local weather as the palette of the sky draws it: how wet and how dusty the air is
//! where the camera stands. Without a weather report the sky is calm. The clouds themselves
//! are not here: both cloud shells read them from the cover map of the cloud field.

use bevy::prelude::*;

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

/// How many more times than clear air the aerosols take the light under full dust: a dust storm
/// shows about 800 m where the ordinary day shows 23 km.
const DUST_FOG: f32 = 30.0;

/// How many more times than clear air the aerosols take the light under a full storm: heavy rain
/// shows about 3 km.
const RAIN_FOG: f32 = 7.0;

/// How many more times than dry air the aerosols take the light in the damp air under a whole
/// cover of cloud: particles swell with the water they take up, and an ordinary day of 23 km
/// becomes a humid one of about 10 km.
const DAMP_FOG: f32 = 1.5;

/// What the weather at the camera does to the sky, each value from 0 to 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeatherLook {
    /// Rain and storm: dark clouds and a grey, closer horizon.
    pub haze: f32,
    /// Lifted dust: an ochre sky and a fog that closes in, in the colour of the dust.
    pub dust: f32,
    /// Damp air, as much as there is cloud over the place: a paler distance.
    pub damp: f32,
}

impl WeatherLook {
    /// Calm weather: no rain, no storm and no dust in the air.
    pub const CALM: Self = Self {
        haze: 0.0,
        dust: 0.0,
        damp: 0.0,
    };

    /// Returns the weather to draw from the report `sky`, calm without a report.
    pub fn of(sky: Option<&LocalWeather>) -> Self {
        sky.map_or(Self::CALM, LocalWeather::look)
    }

    /// Returns the share of the clear sight the aerosols still leave under this weather.
    pub fn sight(&self) -> f32 {
        1.0 / (1.0 + DUST_FOG * self.dust + RAIN_FOG * self.haze + DAMP_FOG * self.damp)
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
/// Empty, the sky is calm.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct LocalWeather(pub Option<WeatherLook>);

impl LocalWeather {
    /// Returns the weather to draw, calm when nothing is reported.
    pub fn look(&self) -> WeatherLook {
        self.0.unwrap_or(WeatherLook::CALM)
    }
}
