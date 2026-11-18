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
            Surface::Shallows | Surface::Reef => self.basin() * 0.9,
            Surface::Pond => self.basin() * 0.8,
            Surface::Sand | Surface::Grass if low => self.basin(),
            Surface::Sand | Surface::Grass | Surface::Snow => land,
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
}
