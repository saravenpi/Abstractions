//! Sky, light and fog colors keyed on the day clock.

use bevy::light::GlobalAmbientLight;
use bevy::prelude::*;

use super::celestial::{daylight, golden, moon_direction, smoothstep, sun_direction};
use super::clock::START_PHASE;
use super::vantage::Vantage;
use super::{DayClock, SunDir};

/// Everything that changes color or strength with the time of day.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    /// Sky color straight up, in sRGB.
    pub zenith: Vec3,
    /// Sky color at the horizon, in sRGB. The distance fog uses it too.
    pub horizon: Vec3,
    /// Ambient light color, in sRGB.
    pub ambient: Vec3,
    /// Ambient light brightness.
    pub ambient_brightness: f32,
    /// Sun light color, in sRGB.
    pub sun_color: Vec3,
    /// Sun illuminance in lux.
    pub sun_lux: f32,
    /// Moon illuminance in lux.
    pub moon_lux: f32,
    /// Cloud color, in sRGB.
    pub cloud: Vec3,
    /// Star visibility from 0 to 1.
    pub stars: f32,
    /// Unit vector pointing at the sun.
    pub sun_dir: Vec3,
    /// Unit vector pointing at the moon.
    pub moon_dir: Vec3,
}

/// Converts an sRGB triple to a color.
pub fn srgb(v: Vec3) -> Color {
    Color::srgb(v.x, v.y, v.z)
}

/// Night, noon and golden hour colors of one palette entry, in sRGB.
struct Key {
    night: Vec3,
    noon: Vec3,
    glow: Vec3,
    glow_amount: f32,
}

impl Key {
    /// Blends the three colors for a daylight factor and a golden hour weight.
    fn at(&self, day: f32, dusk: f32) -> Vec3 {
        let base = self.night.lerp(self.noon, day);
        base.lerp(self.glow, dusk * self.glow_amount)
    }
}

/// Sky color straight up.
const ZENITH: Key = Key {
    night: Vec3::new(0.015, 0.025, 0.08),
    noon: Vec3::new(0.28, 0.52, 0.90),
    glow: Vec3::new(0.36, 0.38, 0.68),
    glow_amount: 0.6,
};

/// Sky color at the horizon.
const HORIZON: Key = Key {
    night: Vec3::new(0.05, 0.08, 0.18),
    noon: Vec3::new(0.62, 0.78, 0.95),
    glow: Vec3::new(1.0, 0.55, 0.32),
    glow_amount: 1.0,
};

/// Ambient light color.
const AMBIENT: Key = Key {
    night: Vec3::new(0.35, 0.45, 0.9),
    noon: Vec3::new(0.7, 0.8, 1.0),
    glow: Vec3::new(1.0, 0.7, 0.6),
    glow_amount: 0.6,
};

/// Cloud color.
const CLOUD: Key = Key {
    night: Vec3::new(0.12, 0.14, 0.24),
    noon: Vec3::new(1.0, 1.0, 1.0),
    glow: Vec3::new(1.0, 0.62, 0.45),
    glow_amount: 0.9,
};

impl Palette {
    /// Illuminance in lux of the moon when it stands high.
    pub const MOON_LUX: f32 = 350.0;

    /// Ambient brightness at night, low enough that torches and lamps make visible pools of light.
    pub const NIGHT_AMBIENT: f32 = 40.0;

    /// Illuminance in lux of the star in open space, where no air dims it.
    pub const SPACE_LUX: f32 = 12_000.0;

    /// Share of the ambient light left in space, enough to read a cockpit by.
    pub const SPACE_AMBIENT: f32 = 0.2;

    /// Sunlight color in space, in sRGB.
    pub const SPACE_SUN: Vec3 = Vec3::new(1.0, 0.96, 0.9);

    /// Returns the palette for a clock phase.
    pub fn at(phase: f32) -> Self {
        let sun_dir = sun_direction(phase);
        let moon_dir = moon_direction(phase);
        let day = daylight(phase);
        let dusk = golden(phase);
        let warmth = 1.0 - smoothstep(0.0, 0.4, sun_dir.y);
        Self {
            zenith: ZENITH.at(day, dusk),
            horizon: HORIZON.at(day, dusk),
            ambient: AMBIENT.at(day, dusk),
            ambient_brightness: Self::NIGHT_AMBIENT + (350.0 - Self::NIGHT_AMBIENT) * day,
            sun_color: Vec3::new(1.0, 0.93, 0.78).lerp(Vec3::new(1.0, 0.55, 0.25), warmth),
            sun_lux: 12_000.0 * smoothstep(-0.02, 0.3, sun_dir.y),
            moon_lux: Self::MOON_LUX * smoothstep(-0.02, 0.3, moon_dir.y),
            cloud: CLOUD.at(day, dusk),
            stars: 1.0 - smoothstep(-0.2, 0.0, sun_dir.y),
            sun_dir,
            moon_dir,
        }
    }

    /// Returns this palette as seen from the camera: the sun and moon stand where the star really is,
    /// and with `air` going from 1 at the ground to 0 in space the sky turns black, the stars come
    /// out and the star shines undimmed unless the planet hides it.
    pub fn seen(mut self, sun: &SunDir, air: f32) -> Self {
        self.sun_dir = sun.dir;
        self.moon_dir = -sun.dir;
        self.zenith *= air;
        self.horizon *= air;
        self.ambient_brightness *= Self::SPACE_AMBIENT + (1.0 - Self::SPACE_AMBIENT) * air;
        self.sun_color = Self::SPACE_SUN.lerp(self.sun_color, air);
        self.sun_lux = Self::SPACE_LUX * sun.lit * (1.0 - air) + self.sun_lux * air;
        self.moon_lux *= air;
        self.stars = self.stars.max(1.0 - air);
        self
    }
}

/// Recomputes the palette from the clock and the camera's place, and applies it to ambient light
/// and the clear color.
fn update_palette(
    clock: Res<DayClock>,
    place: (Res<SunDir>, Res<Vantage>),
    mut palette: ResMut<Palette>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut clear: ResMut<ClearColor>,
) {
    *palette = Palette::at(clock.phase).seen(&place.0, place.1.air);
    ambient.color = srgb(palette.ambient);
    ambient.brightness = palette.ambient_brightness;
    clear.0 = srgb(palette.horizon);
}

/// Registers the palette resource and its update.
pub(super) fn plugin(app: &mut App) {
    app.insert_resource(Palette::at(START_PHASE))
        .add_systems(Update, update_palette.run_if(resource_changed::<DayClock>));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Noon is bright with a lit sun, midnight is dim with a lit moon and stars.
    ///
    /// The moon and the night ambient stay weak enough that a torch still shows at three meters.
    #[test]
    fn noon_and_midnight_differ() {
        let noon = Palette::at(0.25);
        let night = Palette::at(0.75);
        assert!(noon.sun_lux > 10_000.0 && noon.moon_lux < 1.0);
        assert!(night.sun_lux < 1.0 && (250.0..=450.0).contains(&night.moon_lux));
        assert!(night.ambient_brightness <= 45.0 && (noon.ambient_brightness - 350.0).abs() < 1.0);
        assert!(noon.horizon.length() > night.horizon.length() * 3.0);
        assert!(night.stars > 0.99 && noon.stars < 0.01);
    }
}
