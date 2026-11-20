//! The one colour of the ground: what a place of a planet looks like, read the same way by the near mesh,
//! the far mesh, the ball and the map, so a place keeps its colour at every height of the camera.
//!
//! The surface follows from the height and the slope of the land read at a fixed scale, as
//! [`Globe::glance`] reads them, and never from the facet of the mesh that happens to draw it, whose size
//! changes with the level of detail. A carpet of ground cover shifts the colour of grass to the mean colour
//! of its turf, and the ground shader adds only the detail of the turf around that mean.

use bevy::color::{LinearRgba, Srgba};
use bevy::math::{DVec3, Vec2, Vec3};

use super::Lands;
use crate::world::SEA_LEVEL;
use crate::world::Terrain;
use crate::world::biome::Biome;
use crate::world::globe::Globe;
use crate::world::scatter::{Flora, turf_of};
use crate::world::surface::Surface;

/// A place of the land as the colour reads it: its height above the sea in metres and its slope as rise over run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reading {
    /// Height of the bare ground above the sea level, in metres.
    pub height: f32,
    /// Rise over run of the ground, measured over the same distance wherever it is read.
    pub slope: f32,
}

impl Reading {
    /// Reads the land of `globe` in unit direction `dir`: the bare height and the slope over a fixed
    /// analytic distance, whatever the grid of the mesh that asks.
    pub fn of(globe: &Globe, dir: DVec3) -> Self {
        let (height, spot) = globe.glance(dir);
        Self {
            height,
            slope: spot.slope.length(),
        }
    }
}

/// The ground cover of a planet as the colour of its grass reads it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Verdure {
    /// Mean linear colour of the carpet its turf lays.
    pub mean: Vec3,
    /// How thick the carpet grows over a stretch too wide to see its patches, from 0 to 1.
    pub cover: f32,
}

impl Verdure {
    /// Returns the ground cover at chart position `at` of `terrain`, or nothing where no turf grows.
    ///
    /// The turf has two colours in patches, so its mean is halfway between them. A wide stretch is covered
    /// as much as the planet is on the whole: what its patchiness leaves, times the density of the turf.
    pub fn of(terrain: &Terrain, at: Vec2) -> Option<Self> {
        let flora = Flora::of(terrain, at)?;
        let turf = turf_of(&flora)?;
        let [low, high] = turf.colours.map(Vec3::from_array);
        Some(Self {
            mean: (low + high) * 0.5,
            cover: (1.0 - flora.patchiness) * turf.density.sqrt(),
        })
    }

    /// Returns the sRGB colour `rgb` of grass moved toward the mean colour of the carpet by `thick`, the
    /// share of the ground the carpet covers, mixed as light is.
    pub fn carpeted(self, rgb: (f32, f32, f32), thick: f32) -> (f32, f32, f32) {
        let grass = LinearRgba::from(Srgba::rgb(rgb.0, rgb.1, rgb.2));
        let grass = Vec3::new(grass.red, grass.green, grass.blue);
        let mixed = grass.lerp(self.mean, thick.clamp(0.0, 1.0));
        let shown = Srgba::from(LinearRgba::rgb(mixed.x, mixed.y, mixed.z));
        (shown.red, shown.green, shown.blue)
    }
}

impl Lands {
    /// Returns the sRGB colour of the ground at `reading` in the biome `base`, tinted by the biome `near`
    /// dithered across a border when that biome covers it the same way, and the surface that colour is of.
    /// A channel `bed` above the sea takes the place of the surface.
    ///
    /// The surface always follows `base`: a dithered biome that would turn sand into grass or grass into
    /// rock would leave single triangles of the wrong surface on a border.
    pub fn colour(
        self,
        reading: Reading,
        [base, near]: [Biome; 2],
        bed: Option<Surface>,
    ) -> ((f32, f32, f32), Surface) {
        let Reading { height, slope } = reading;
        let surface = match bed {
            Some(bed) if height > SEA_LEVEL => bed,
            _ => self.surface(height, slope, base),
        };
        let tint = match self.surface(height, slope, near) == surface {
            true => near,
            false => base,
        };
        (self.rgb(surface, tint, height), surface)
    }
}

impl Globe {
    /// Returns the sRGB colour of the dry land of this planet at `reading` in the biome `biome`: the colour
    /// the meshes of the world draw there, without a carpet of ground cover.
    pub fn land_colour(&self, reading: Reading, biome: Biome) -> Vec3 {
        let ((r, g, b), _) = self.lands().colour(reading, [biome, biome], None);
        Vec3::new(r, g, b)
    }
}
