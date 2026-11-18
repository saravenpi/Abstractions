//! The grey of one texel of the ground map: what covers the ground there, and how bright and
//! how patterned that reads on each class of planet.
//!
//! The interface has no hue, so brightness and pattern carry everything. Water is the darkest
//! and flat. Land brightens with height in five bands, so a coast and a hill read at a glance.
//! A dry basin is dark land with a dot grid, ice is bright with flakes, molten rock is the
//! brightest thing on its dark world and shimmers as a checkerboard.

use crate::universe::PlanetClass;

/// Heights in metres above the sea level that part the five bands of land: the shore, the
/// lowland, the hills, the heights and the summits.
pub const BANDS: [f32; 4] = [1.5, 10.0, 22.0, 38.0];

/// Floor height in metres under which a liquid is deep.
const SHALLOW: f32 = -4.6;

/// Grey of the five bands of ordinary land, from the shore up.
const LAND: [u8; 5] = [104, 128, 154, 184, 218];

/// Grey of the five bands of ice.
const ICE: [u8; 5] = [176, 194, 212, 230, 248];

/// Grey of the five bands of the dark crust of a lava world.
const CRUST: [u8; 5] = [30, 44, 60, 78, 98];

/// What covers the ground at one texel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cover {
    /// Liquid over a deep floor.
    Deep,
    /// Liquid over a floor near the surface.
    Shallow,
    /// Ground below the sea level of a planet that has no sea.
    Basin,
    /// Land in this band, from 0 at the shore to 4 at the summits.
    Land(u8),
}

/// Returns what covers ground at `height` metres above the sea level, on a planet with or
/// without a `sea`.
pub fn cover(height: f32, sea: bool) -> Cover {
    if height > 0.0 {
        let band = BANDS.iter().filter(|edge| height > **edge).count();
        return Cover::Land(band as u8);
    }
    match (sea, height > SHALLOW) {
        (false, _) => Cover::Basin,
        (true, true) => Cover::Shallow,
        (true, false) => Cover::Deep,
    }
}

/// Returns the grey of a liquid on a planet of `class` at the texel `(x, y)`: dark flat water,
/// or bright molten rock that cools toward its shore.
fn sea(class: PlanetClass, deep: bool, (x, y): (usize, usize)) -> u8 {
    let molten = class == PlanetClass::Lava;
    let checker = u8::from((x + y) % 2 == 1) * 48;
    match (molten, deep) {
        (true, true) => 255 - checker,
        (true, false) => 208 - checker,
        (false, true) => 10,
        (false, false) => 34,
    }
}

/// Returns the grey of ground that is not under a liquid: a band of land, or a basin, which
/// is a darker floor dotted with the grey of the shore. On a frozen world the basin is the
/// frozen sea, and stays bright.
fn ground(class: PlanetClass, band: Option<u8>, (x, y): (usize, usize)) -> u8 {
    let (ramp, floor) = match class {
        PlanetClass::Ice => (ICE, 150),
        PlanetClass::Lava => (CRUST, 18),
        _ => (LAND, 70),
    };
    let flake = class == PlanetClass::Ice && (x + 2 * y) % 5 == 0;
    let base = match band {
        Some(band) => ramp[usize::from(band).min(4)],
        None if x % 2 == 0 && y % 2 == 0 => ramp[0],
        None => floor,
    };
    base - u8::from(flake) * 70
}

/// Returns the grey, from 0 to 255, of `cover` on a planet of `class` at the texel `cell` of
/// the map, whose column and row give the patterns their grid.
pub fn tone(class: PlanetClass, cover: Cover, cell: (usize, usize)) -> u8 {
    match cover {
        Cover::Deep => sea(class, true, cell),
        Cover::Shallow => sea(class, false, cell),
        Cover::Basin => ground(class, None, cell),
        Cover::Land(band) => ground(class, Some(band), cell),
    }
}
