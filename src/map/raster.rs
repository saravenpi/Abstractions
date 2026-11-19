//! The flat coloured picture of a body: 512 by 256 pixels, longitude across from 0 going east
//! and latitude down from the north pole, one pixel per cell of the explored grid.
//!
//! Each body asked for keeps its picture. Only explored cells are painted, on a worker thread,
//! and each cell once: a newly explored cell is painted alone. The rest stays black with a
//! faint grid.

mod paint;
#[cfg(test)]
mod tests;
mod work;

pub use paint::colour;

use std::collections::HashMap;
use std::sync::Arc;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::tasks::Task;

use super::explored::grid::{COLS, ROWS, WORDS};
use crate::screens::Screen;
use crate::universe::BodyId;
use crate::world::Globe;

/// Bytes of one pixel: red, green, blue and alpha.
pub const BYTES: usize = 4;

/// Cells between two lines of the faint grid over unexplored ground.
const GRID: usize = 32;

/// Grey of the faint grid.
const LINE: u8 = 24;

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
    /// The cells painted or being painted, one bit a cell.
    painted: Vec<u64>,
    /// The version of the explored ground every painted cell was taken from.
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
            painted: vec![0; WORDS],
            ..default()
        });
    }
}

/// Returns a picture where nothing is explored: black with a faint grid.
fn blank() -> Image {
    let size = Extent3d {
        width: COLS as u32,
        height: ROWS as u32,
        depth_or_array_layers: 1,
    };
    let mut pixels = vec![0; COLS * ROWS * BYTES];
    for (index, pixel) in pixels.as_chunks_mut::<BYTES>().0.iter_mut().enumerate() {
        let (row, column) = (index / COLS, index % COLS);
        let grey = match row % GRID == 0 || column % GRID == 0 {
            true => LINE,
            false => 0,
        };
        *pixel = [grey, grey, grey, u8::MAX];
    }
    let (format, usage) = (TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    let mut image = Image::new(size, TextureDimension::D2, pixels, format, usage);
    image.sampler = ImageSampler::nearest();
    image
}

/// Forgets the pictures of the world that ended: the same body of another world looks else.
fn forget(mut rasters: ResMut<Rasters>) {
    *rasters = Rasters::default();
}

/// Registers the painting of body pictures.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Rasters>()
        .add_systems(
            Update,
            (work::open, work::plan, work::collect)
                .chain()
                .run_if(|rasters: Res<Rasters>| !rasters.bodies.is_empty()),
        )
        .add_systems(OnEnter(Screen::Title), forget);
}
