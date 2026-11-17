//! Sky, light and fog colors, derived from where the star stands and what the air does to its light.

use bevy::light::GlobalAmbientLight;
use bevy::prelude::*;

use super::SunDir;
use super::atmosphere::horizon_dip;
use super::celestial::{smoothstep, sun_direction};
use super::clock::START_PHASE;
use super::scatter::{luminance, peak, scattered, transmit};
use super::vantage::{LocalSystem, Vantage};

/// Everything that changes color or strength with the height of the star.
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

/// Sky color straight up at night, in sRGB.
const NIGHT_ZENITH: Vec3 = Vec3::new(0.015, 0.025, 0.08);

/// Sky color at the horizon at night, in sRGB.
const NIGHT_HORIZON: Vec3 = Vec3::new(0.05, 0.08, 0.18);

/// Ambient light color at night, in sRGB.
const NIGHT_AMBIENT_COLOR: Vec3 = Vec3::new(0.35, 0.45, 0.9);

/// Cloud color at night, in sRGB.
const NIGHT_CLOUD: Vec3 = Vec3::new(0.12, 0.14, 0.24);

/// Share of white in the sky toward the horizon, where the long path through the air has
/// scattered every colour.
const HAZE: f32 = 0.6;

impl Palette {
    /// Illuminance in lux of the moon when it stands high.
    pub const MOON_LUX: f32 = 350.0;

    /// Ambient brightness at night, low enough that torches and lamps make visible pools of light.
    pub const NIGHT_AMBIENT: f32 = 40.0;

    /// Illuminance in lux of the star in open space, and straight overhead through the air.
    pub const SPACE_LUX: f32 = 12_000.0;

    /// Share of the ambient light left in space, enough to read a cockpit by.
    pub const SPACE_AMBIENT: f32 = 0.2;

    /// Returns the palette at the ground under a star of linear colour `star` whose height
    /// above the horizon has the sine `height`.
    ///
    /// Nothing here is keyed on the hour. The star light is what the air lets through on its
    /// path, so it reddens and dims as the star sinks; the sky is what the air scatters out of
    /// it; the ambient light and the clouds take their colour from both.
    pub fn lit(height: f32, star: Vec3) -> Self {
        let through = star * transmit(height, 1.0);
        let tint = peak(through);
        let day = smoothstep(-0.12, 0.25, height);
        let dusk = smoothstep(-0.2, 0.1, height);
        let blue = scattered(star).lerp(peak(star), 0.1) * peak(star * transmit(height, 0.2));
        let haze = scattered(star).lerp(peak(star), HAZE) * tint.powf(0.7);
        let zenith = NIGHT_ZENITH.lerp(blue * 0.92, day);
        let horizon = NIGHT_HORIZON.lerp(haze * 0.95, dusk);
        let sky = peak(zenith + horizon).lerp(Vec3::ONE, 0.4);
        let full = luminance(star * transmit(1.0, 1.0)).max(1.0e-6);
        Self {
            zenith,
            horizon,
            ambient: NIGHT_AMBIENT_COLOR.lerp(sky, day),
            ambient_brightness: Self::NIGHT_AMBIENT + (350.0 - Self::NIGHT_AMBIENT) * day,
            sun_color: tint,
            sun_lux: Self::SPACE_LUX
                * (luminance(through) / full).min(1.0)
                * smoothstep(-0.06, 0.03, height),
            moon_lux: Self::MOON_LUX * smoothstep(-0.02, 0.3, -height),
            cloud: NIGHT_CLOUD.lerp(tint.lerp(Vec3::ONE, 0.35), day),
            stars: 1.0 - smoothstep(-0.2, 0.0, height),
            sun_dir: Vec3::Y,
            moon_dir: Vec3::NEG_Y,
        }
    }

    /// Returns the palette for a clock phase under a white star, with the sun on its plain path.
    pub fn at(phase: f32) -> Self {
        let sun_dir = sun_direction(phase);
        Self {
            sun_dir,
            moon_dir: -sun_dir,
            ..Self::lit(sun_dir.y, Vec3::ONE)
        }
    }

    /// Returns this palette as seen from the camera: the sun and moon stand where the star really is,
    /// and with `air` going from 1 at the ground to 0 in space the sky turns black, the stars come
    /// out and the star shines in its own colour `star`, undimmed unless the planet hides it.
    pub fn seen(mut self, sun: &SunDir, air: f32, star: Vec3) -> Self {
        self.sun_dir = sun.dir;
        self.moon_dir = -sun.dir;
        self.zenith *= air;
        self.horizon *= air;
        self.ambient_brightness *= Self::SPACE_AMBIENT + (1.0 - Self::SPACE_AMBIENT) * air;
        self.sun_color = peak(star).lerp(self.sun_color, air);
        self.sun_lux = Self::SPACE_LUX * sun.lit * (1.0 - air) + self.sun_lux * air;
        self.moon_lux *= air;
        self.stars = self.stars.max(1.0 - air);
        self
    }

    /// Returns the palette at the ground under the camera: lit by the star of `local` at its
    /// real height over the horizon the camera sees, which drops as the camera climbs.
    pub fn under(sun: &SunDir, vantage: &Vantage, local: &LocalSystem) -> (Self, Vec3) {
        let star = local
            .0
            .as_ref()
            .map_or(Vec3::ONE, |near| near.system.star.color);
        let dip = horizon_dip(vantage.altitude, vantage.radius);
        (Self::lit(sun.height + dip, star), star)
    }
}

/// Recomputes the palette from where the star stands for the camera, and applies it to the
/// ambient light and the clear color.
fn update_palette(
    place: (Res<SunDir>, Res<Vantage>, Res<LocalSystem>),
    mut palette: ResMut<Palette>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut clear: ResMut<ClearColor>,
) {
    let (sun, vantage, local) = place;
    let (ground, star) = Palette::under(&sun, &vantage, &local);
    *palette = ground.seen(&sun, vantage.air, star);
    ambient.color = srgb(palette.ambient);
    ambient.brightness = palette.ambient_brightness;
    clear.0 = srgb(palette.horizon);
}

/// Registers the palette resource and its update.
pub(super) fn plugin(app: &mut App) {
    app.insert_resource(Palette::at(START_PHASE))
        .add_systems(Update, update_palette);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Noon is bright with a lit sun, midnight is dim with a lit moon and stars. A star on the
    /// horizon gives orange, weaker light and a warm horizon under a sky still blue above, and
    /// a red star tints all of it.
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
        let (noon, low) = (Palette::lit(0.9, Vec3::ONE), Palette::lit(0.02, Vec3::ONE));
        assert!(low.sun_color.z < 0.3 && noon.sun_color.z > 0.75);
        assert!(low.sun_lux < 0.6 * noon.sun_lux && low.sun_lux > 1000.0);
        assert!(low.horizon.x > low.horizon.z && noon.horizon.z > noon.horizon.x);
        assert!(low.zenith.z > low.zenith.x);
        let red = Palette::lit(0.9, Vec3::new(1.0, 0.5, 0.3));
        assert!(red.sun_color.z < 0.4 && red.zenith.z < noon.zenith.z);
    }
}
