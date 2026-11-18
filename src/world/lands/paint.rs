//! The colours of the lands: the palette of home on a world of seas and islands, the look of the class elsewhere.

use bevy::color::{LinearRgba, Srgba};
use bevy::math::Vec3;

use super::Lands;
use crate::world::SEA_LEVEL;
use crate::world::biome::{Biome, ground_rgb};
use crate::world::surface::Surface;

/// sRGB colour of the flat sea of a coarse mesh of water. Lit by day it reads like the near sea at the
/// hand-over, whose deep water mostly shows the sky (measured from 900 m up).
const WATER_RGB: [f32; 3] = [0.23, 0.6, 0.76];

/// Steps per sRGB channel the colour lookup of the ground keeps, see `world::colormap`.
const STEPS: f32 = 31.0;

/// How much darker than the open ground steep and high rock is.
const ROCK_SHADE: f32 = 0.72;

/// How much of the liquid colour the floor of a basin of molten rock keeps once it is dry.
const EMBER: f32 = 0.2;

/// Returns the sRGB colour of a linear one.
fn srgb_of(linear: Vec3) -> Vec3 {
    let c = Srgba::from(LinearRgba::rgb(linear.x, linear.y, linear.z));
    Vec3::new(c.red, c.green, c.blue)
}

impl Lands {
    /// Returns the sRGB colour of the low ground the liquid of the class fills or once filled.
    fn basin(self) -> Vec3 {
        let look = self.class.look();
        let (land, sea) = (srgb_of(look.land), srgb_of(look.sea));
        match look.glow > 0.0 {
            true => land.lerp(sea, EMBER),
            false => sea,
        }
    }

    /// Returns the sRGB colour of bare ground of the class: its land colour, darker on rock, and the colour
    /// of its basins on the floors and where `low` ground lies dry below the sea level.
    fn bare(self, surface: Surface, low: bool) -> Vec3 {
        let land = srgb_of(self.class.look().land);
        match surface {
            Surface::Rock => land * ROCK_SHADE,
            Surface::Water => self.basin() * 0.6,
            Surface::Shallows | Surface::Reef | Surface::River => self.basin() * 0.9,
            Surface::Pond => self.basin() * 0.8,
            Surface::Sand | Surface::Grass if low => self.basin(),
            _ => land,
        }
    }

    /// Returns the base sRGB colour of `surface` in `biome` at `height` above the sea level.
    ///
    /// A world of seas and islands has the palette of home. Every other class is painted from its look, so
    /// the ground seen from afar is the colour the far view and the sky expect.
    pub fn rgb(self, surface: Surface, biome: Biome, height: f32) -> (f32, f32, f32) {
        if self.temperate() {
            return ground_rgb(surface, biome);
        }
        let c = self.bare(surface, height <= SEA_LEVEL);
        (c.x, c.y, c.z)
    }

    /// Returns the sRGB colour of the flat sea a coarse mesh draws: water as at home, or the liquid of the class.
    pub fn sea_rgb(self) -> [f32; 3] {
        match self.watery() {
            true => WATER_RGB,
            false => srgb_of(self.class.look().sea).to_array(),
        }
    }

    /// Returns the linear colour the flat sea of a coarse mesh shows before it is lit: [`Lands::sea_rgb`]
    /// rounded as the colour lookup stores it, a step per channel and then a byte. The near sea fades to
    /// this colour, lit the same way, so the two meet without a seam.
    pub fn sea_shown(self) -> Vec3 {
        let stored = |c: f32| ((c.clamp(0.0, 1.0) * STEPS).round() * 255.0 / STEPS).round() / 255.0;
        let [r, g, b] = self.sea_rgb().map(stored);
        let shown = LinearRgba::from(Srgba::rgb(r, g, b));
        Vec3::new(shown.red, shown.green, shown.blue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::{PlanetClass, PlanetRecord};
    use crate::world::colormap;
    use crate::world::shapes::srgb;

    /// The record of a world with a sea, whose class each test sets.
    const WET: PlanetRecord = PlanetRecord {
        ocean: 0.4,
        ..PlanetRecord::TEMPERATE
    };

    /// The colour the near sea fades to is, byte for byte, the texel a coarse sea triangle reads.
    #[test]
    fn the_shown_sea_is_the_texel_of_the_coarse_sea() {
        let image = colormap::image();
        let (data, size) = (image.data.as_ref().unwrap(), image.texture_descriptor.size);
        for class in PlanetClass::ALL {
            let lands = Lands::of(&PlanetRecord { class, ..WET });
            let [r, g, b] = lands.sea_rgb();
            let [u, v] = colormap::uv(srgb(r, g, b));
            let (x, y) = (
                (u * size.width as f32) as usize,
                (v * size.height as f32) as usize,
            );
            let at = (y * size.width as usize + x) * 4;
            let texel = Srgba::rgb_u8(data[at], data[at + 1], data[at + 2]);
            let stored = LinearRgba::from(texel);
            let shown = lands.sea_shown();
            let off = Vec3::new(stored.red, stored.green, stored.blue) - shown;
            assert!(off.abs().max_element() < 1.0e-6, "{class:?}: {off}");
        }
    }

    /// On a world whose sea glows, no ground colour under any tone of a triangle reads the texel of the
    /// sea, so the glow of that texel lights the sea alone.
    #[test]
    fn no_ground_of_a_glowing_world_reads_the_sea_texel() {
        let glowing = PlanetClass::ALL.into_iter().filter(|c| c.look().glow > 0.0);
        for lands in glowing.map(|class| Lands::of(&PlanetRecord { class, ..WET })) {
            let [r, g, b] = lands.sea_rgb();
            let sea = colormap::uv(srgb(r, g, b));
            let surfaces = [
                Surface::Water,
                Surface::Shallows,
                Surface::Pond,
                Surface::Sand,
            ];
            let spots = surfaces.into_iter().chain([Surface::Grass, Surface::Rock]);
            for (surface, step) in spots.flat_map(|s| (0..=64).map(move |k| (s, k))) {
                let t = 0.92 + 0.16 * step as f32 / 64.0;
                for height in [-8.0, 12.0] {
                    let (r, g, b) = lands.rgb(surface, Biome::Desert, height);
                    assert_ne!(colormap::uv(srgb(r * t, g * t, b * t)), sea, "{surface:?}");
                }
            }
        }
    }
}
