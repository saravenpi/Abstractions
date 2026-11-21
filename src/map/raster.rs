//! The flat coloured picture of a body: 512 by 256 pixels, longitude across from 0 going east
//! and latitude down from the north pole, one pixel per cell of its own grid.
//!
//! Each body asked for keeps its picture. A cell is painted, on a worker thread and once, when
//! a leaf of the explored ground falls in it. The rest stays clear.
//! Painting waits while the map is closed, and the pictures stay painted.
//!
//! Zoomed in, tiles painted from the planet's own ground functions lie over the picture, never
//! from the terrain the world has loaded. While a tile is painted, a coarser one that holds it
//! is drawn in its place.

mod cover;
pub mod grid;
mod paint;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_bake;
#[cfg(test)]
mod tests_coast;
#[cfg(test)]
mod tests_ground;
#[cfg(test)]
mod tests_sea;
#[cfg(test)]
mod tests_tile;
pub mod tile;
mod tile_paint;
mod tiles;
mod tiles_view;
mod work;

pub use paint::colour;
pub use tiles::Tiles;
pub use tiles_view::tile_layer;

use std::collections::HashMap;
use std::sync::Arc;

use abstractions_net::leaves::Leaves;
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::tasks::Task;

use self::grid::{COLS, ROWS, WORDS};
use crate::screens::Screen;
use crate::universe::BodyId;
use crate::world::Globe;

/// Bytes of one pixel: red, green, blue and alpha.
pub const BYTES: usize = 4;

/// A pixel of ground nobody has seen: clear, so the dim tone of the map shows through it and
/// the unexplored ground reads as a quiet part of the panel, not as a black hole.
pub const UNSEEN: [u8; BYTES] = [0; BYTES];

/// The pictures of the bodies the map asked for.
#[derive(Resource, Default)]
pub struct Rasters {
    /// The picture of each body asked for.
    bodies: HashMap<BodyId, Raster>,
}

/// The picture of one body and the painting under way.
#[derive(Default)]
struct Raster {
    /// The image, from the frame after the first request.
    image: Option<Handle<Image>>,
    /// The cells painted, one bit a cell.
    lit: Vec<u64>,
    /// The leaves of the explored ground already looked at.
    painted: Leaves,
    /// The version of the explored ground the last worker was started for.
    seen: u64,
    /// The ground of the body, once a worker built it.
    globe: Option<Arc<Globe>>,
    /// The worker painting cells.
    job: Option<Task<Painted>>,
}

/// What a worker hands back: the ground it read and the colour of each cell it painted.
struct Painted {
    /// The ground of the body.
    globe: Arc<Globe>,
    /// Each cell with its pixel.
    cells: Vec<(u32, [u8; BYTES])>,
}

impl Rasters {
    /// Returns the picture of `body`, or nothing until the frame after the first
    /// [`Rasters::request`]. The handle stays the same while the world lasts; its pixels change
    /// as ground is explored.
    pub fn image(&self, body: BodyId) -> Option<Handle<Image>> {
        self.bodies.get(&body)?.image.clone()
    }

    /// Asks for the picture of `body`. It is kept and painted from then on; asking again does
    /// nothing.
    pub fn request(&mut self, body: BodyId) {
        self.bodies.entry(body).or_insert_with(|| Raster {
            lit: vec![0; WORDS],
            ..default()
        });
    }
}

/// Returns a picture where nothing is explored: clear everywhere, with no lines.
fn blank() -> Image {
    let size = Extent3d {
        width: COLS as u32,
        height: ROWS as u32,
        depth_or_array_layers: 1,
    };
    let pixels = UNSEEN.repeat(COLS * ROWS);
    let (format, usage) = (TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    let mut image = Image::new(size, TextureDimension::D2, pixels, format, usage);
    image.sampler = ImageSampler::nearest();
    image
}

/// Forgets the pictures of the world that ended: the same body of another world looks else.
fn forget(mut rasters: ResMut<Rasters>, mut tiles: ResMut<tiles::Tiles>) {
    *rasters = Rasters::default();
    *tiles = tiles::Tiles::default();
}

/// Registers the painting of body pictures.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Rasters>()
        .init_resource::<tiles::Tiles>()
        .add_systems(
            Update,
            (
                tiles::want,
                tiles::start,
                tiles::collect,
                tiles::trim,
                tiles_view::show,
            )
                .chain()
                .run_if(super::on_planet)
                .run_if(super::open),
        )
        .add_systems(
            Update,
            (work::open, work::plan, work::collect)
                .chain()
                .run_if(|rasters: Res<Rasters>| !rasters.bodies.is_empty())
                .run_if(super::open),
        )
        .add_systems(OnEnter(Screen::Title), forget);
}
