//! Sky, light and fog colors, derived from where the star stands and what the air does to its light.

use bevy::light::GlobalAmbientLight;
use bevy::prelude::*;

use super::SunDir;
use super::air::Air;
use super::atmosphere::horizon_dip;
use super::celestial::{smoothstep, sun_direction};
use super::clock::START_PHASE;
use super::moonlight::{Moonlight, moonlit};
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

    /// Returns the palette at the ground of the home planet under a star of linear colour `star`
    /// whose height above the horizon has the sine `height`.
    pub fn lit(height: f32, star: Vec3) -> Self {
        Self::through(height, star, &Air::HOME)
    }

    /// Returns the palette at the ground under a star of linear colour `star` whose height
    /// above the horizon has the sine `height`, seen through `air` at its full pressure.
    ///
    /// Nothing here is keyed on the hour. The star light is what the air lets through on its
    /// path, so it reddens and dims as the star sinks, the more so the thicker the air; the sky
    /// is what the air scatters out of it, stained by what the air carries; the ambient light
    /// and the clouds take their colour from both. Thin air is left to [`Palette::seen`].
    pub fn through(height: f32, star: Vec3, air: &Air) -> Self {
        let depth = air.depth();
        let through = star * transmit(height, depth);
        let tint = peak(through);
        let day = smoothstep(-0.12, 0.25, height);
        let dusk = smoothstep(-0.2, 0.1, height);
        let white = HAZE + (1.0 - HAZE) * (1.0 - 1.0 / depth);
        let blue =
            scattered(star).lerp(peak(star), 0.1) * peak(star * transmit(height, 0.2 * depth));
        let haze = scattered(star).lerp(peak(star), white) * tint.powf(0.7);
        let zenith = air.stain(NIGHT_ZENITH.lerp(blue * 0.92, day), star);
        let horizon = air.stain(NIGHT_HORIZON.lerp(haze * 0.95, dusk), star);
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
            cloud: NIGHT_CLOUD.lerp(tint.lerp(Vec3::ONE, 0.35), day) * air.tint,
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
    /// and with `air` going from 1 under a whole home sky to 0 in space or on a planet without
    /// air, the sky turns black, the stars come out and the star shines in its own colour
    /// `star`, undimmed unless the planet hides it.
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
    /// real height over the horizon the camera sees, which drops as the camera climbs, through
    /// the air of the planet below.
    pub fn under(sun: &SunDir, vantage: &Vantage, local: &LocalSystem) -> (Self, Vec3) {
        let star = local
            .0
            .as_ref()
            .map_or(Vec3::ONE, |near| near.system.star.color);
        let dip = horizon_dip(vantage.altitude, vantage.radius);
        let air = local.air_below(vantage);
        (Self::through(sun.height + dip, star, &air), star)
    }
}

/// Recomputes the palette from where the star stands for the camera and the real moon that
/// lights its night, and applies it to the ambient light and the clear color.
fn update_palette(
    place: (Res<SunDir>, Res<Vantage>, Res<LocalSystem>, Res<Moonlight>),
    mut palette: ResMut<Palette>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut clear: ResMut<ClearColor>,
) {
    let (sun, vantage, local, moon) = place;
    let (ground, star) = Palette::under(&sun, &vantage, &local);
    *palette = moonlit(ground.seen(&sun, vantage.air, star), &moon);
    ambient.color = srgb(palette.ambient);
    ambient.brightness = palette.ambient_brightness;
    clear.0 = srgb(palette.horizon);
}

/// Registers the palette resource and its update.
pub(super) fn plugin(app: &mut App) {
    app.insert_resource(Palette::at(START_PHASE))
        .add_systems(Update, update_palette);
}
