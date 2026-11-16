//! The look of one village: a flavor from the biome and a palette rolled from the village seed.

use crate::world::Biome;
use crate::world::structures::palette::{
    DARK_WOOD, PLANK, ROOF, SAND, SAND_DARK, STONE, STONE_DARK, THATCH, WHITE, WOOD,
};

/// Building tradition of a village.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    /// Timber houses with red or slate roofs.
    Timber,
    /// Dark log houses with snow on the roofs.
    Taiga,
    /// Flat-roofed adobe with awnings.
    Adobe,
    /// Thatched houses on stilts.
    Stilt,
    /// White walls under dark tiered roofs, with lanterns.
    Cherry,
}

/// Colors and tradition shared by every building of a village.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// Building tradition.
    pub flavor: Flavor,
    /// Two wall tones.
    pub wall: [[f32; 3]; 2],
    /// Two roof tones.
    pub roof: [[f32; 3]; 2],
    /// Door frames, beams and shutters.
    pub trim: [f32; 3],
    /// Foundation platforms.
    pub base: [f32; 3],
    /// Street surface.
    pub road: [f32; 3],
    /// Awnings, flowers and banners.
    pub cloth: [[f32; 3]; 3],
}

const CLOTH: [[f32; 3]; 3] = [[0.78, 0.2, 0.18], [0.2, 0.5, 0.62], [0.9, 0.72, 0.25]];

const TIMBER_WALLS: [[[f32; 3]; 2]; 3] = [[PLANK, WOOD], [WHITE, PLANK], [[0.86, 0.78, 0.6], WOOD]];

const TIMBER_ROOFS: [[[f32; 3]; 2]; 3] = [
    [ROOF, [0.5, 0.2, 0.15]],
    [[0.33, 0.36, 0.42], [0.28, 0.3, 0.36]],
    [[0.45, 0.3, 0.2], [0.38, 0.25, 0.17]],
];

/// Taiga: dark logs under snowy roofs.
fn taiga(base: Style) -> Style {
    Style {
        flavor: Flavor::Taiga,
        wall: [[0.3, 0.2, 0.12], [0.38, 0.25, 0.15]],
        roof: [[0.24, 0.2, 0.18], [0.2, 0.17, 0.15]],
        road: [0.55, 0.55, 0.57],
        ..base
    }
}

/// Desert: sand-colored adobe with flat roofs.
fn desert(base: Style) -> Style {
    Style {
        flavor: Flavor::Adobe,
        wall: [SAND, [0.78, 0.62, 0.4]],
        roof: [SAND_DARK, [0.66, 0.52, 0.32]],
        base: SAND_DARK,
        road: [0.8, 0.68, 0.45],
        ..base
    }
}

/// Tropical: thatched stilt houses on plank walkways.
fn tropical(base: Style) -> Style {
    Style {
        flavor: Flavor::Stilt,
        wall: [WOOD, PLANK],
        roof: [THATCH, [0.7, 0.58, 0.28]],
        base: PLANK,
        road: PLANK,
        ..base
    }
}

/// Cherry: white walls under dark tiered roofs.
fn cherry(base: Style) -> Style {
    Style {
        flavor: Flavor::Cherry,
        wall: [WHITE, [0.95, 0.9, 0.88]],
        roof: [[0.2, 0.2, 0.26], [0.26, 0.22, 0.3]],
        base: STONE,
        road: [0.62, 0.58, 0.56],
        ..base
    }
}

/// Returns the style of a village centered in `biome`, rolled from `variant`.
pub fn style_for(biome: Biome, variant: u32) -> Style {
    let base = Style {
        flavor: Flavor::Timber,
        wall: TIMBER_WALLS[(variant >> 4) as usize % 3],
        roof: TIMBER_ROOFS[(variant >> 9) as usize % 3],
        trim: DARK_WOOD,
        base: STONE_DARK,
        road: [0.48, 0.4, 0.3],
        cloth: CLOTH,
    };
    match biome {
        Biome::Taiga => taiga(base),
        Biome::Desert => desert(base),
        Biome::Tropical => tropical(base),
        Biome::Cherry => cherry(base),
        Biome::Meadow | Biome::Forest => base,
    }
}
