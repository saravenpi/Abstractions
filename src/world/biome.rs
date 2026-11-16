//! Biomes: the climate regions that decide ground colors, plants and structures.

use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

use super::Surface;
use super::islands::sub_seed;

/// The climate region of a point on land.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Biome {
    /// Open grassland with scattered oaks and many flowers.
    Meadow,
    /// Dense oak and pine woodland with ferns and mushrooms.
    Forest,
    /// Pink blossom groves with petals and flowers.
    Cherry,
    /// Dry sand with cacti and dead bushes.
    Desert,
    /// Cold snowy land with pines.
    Taiga,
    /// Hot humid land with palms and lush plants.
    Tropical,
}

impl Biome {
    /// Every biome, in a stable order.
    pub const ALL: [Biome; 6] = [
        Biome::Meadow,
        Biome::Forest,
        Biome::Cherry,
        Biome::Desert,
        Biome::Taiga,
        Biome::Tropical,
    ];
}

/// The temperature and moisture noise fields.
#[derive(Clone)]
pub struct Climate {
    temperature: Fbm<Perlin>,
    moisture: Fbm<Perlin>,
}

impl Climate {
    /// Builds the climate fields from the world seed.
    pub fn new(seed: u64) -> Self {
        Self {
            temperature: Fbm::<Perlin>::new(sub_seed(seed, 6))
                .set_octaves(2)
                .set_frequency(1.0 / 850.0),
            moisture: Fbm::<Perlin>::new(sub_seed(seed, 7))
                .set_octaves(2)
                .set_frequency(1.0 / 700.0),
        }
    }

    /// Returns the biome at `(x, z)`.
    pub fn biome(&self, x: f32, z: f32) -> Biome {
        let p = [x as f64, z as f64];
        let t = self.temperature.get(p) as f32;
        let m = self.moisture.get(p) as f32;
        if t < -0.32 {
            Biome::Taiga
        } else if t > 0.22 {
            if m < 0.0 {
                Biome::Desert
            } else {
                Biome::Tropical
            }
        } else if m > 0.1 {
            Biome::Forest
        } else if m < -0.12 && t < 0.0 {
            Biome::Cherry
        } else {
            Biome::Meadow
        }
    }
}

/// Returns the base sRGB color of a surface in a biome.
pub fn ground_rgb(surface: Surface, biome: Biome) -> (f32, f32, f32) {
    match (surface, biome) {
        (Surface::Water, _) => (0.16, 0.22, 0.26),
        (Surface::Snow, Biome::Taiga) => (0.84, 0.89, 0.94),
        (Surface::Snow, _) => (0.94, 0.96, 0.99),
        (Surface::Sand, Biome::Desert) => (0.90, 0.72, 0.40),
        (Surface::Sand, Biome::Tropical) => (0.90, 0.82, 0.60),
        (Surface::Sand, _) => (0.82, 0.74, 0.52),
        (Surface::Rock, Biome::Desert) => (0.62, 0.45, 0.34),
        (Surface::Rock, Biome::Taiga) => (0.36, 0.36, 0.40),
        (Surface::Rock, Biome::Forest) => (0.42, 0.42, 0.40),
        (Surface::Rock, _) => (0.48, 0.46, 0.45),
        (Surface::Grass, Biome::Forest) => (0.16, 0.46, 0.18),
        (Surface::Grass, Biome::Meadow) => (0.52, 0.68, 0.26),
        (Surface::Grass, Biome::Cherry) => (0.62, 0.70, 0.42),
        (Surface::Grass, Biome::Tropical) => (0.14, 0.70, 0.22),
        (Surface::Grass, Biome::Desert) => (0.70, 0.62, 0.34),
        (Surface::Grass, Biome::Taiga) => (0.25, 0.28, 0.20),
    }
}
