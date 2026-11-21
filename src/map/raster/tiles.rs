//! The cache of tiles: the tiles the lens looks at are painted on workers, kept while they are
//! used and dropped, the farthest from the view first, when the cache is full.

use std::collections::HashMap;
use std::sync::Arc;

use abstractions_net::leaves::Leaves;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

#[cfg(test)]
use super::BYTES;
use super::Rasters;
use super::tile::{SIDE, TileId, ZOOM_FROM, away, level_for, visible};
use super::tile_paint::{lit, paint};
use crate::map::explored::Explored;
use crate::map::{Level, Surface};
use crate::universe::BodyId;
use crate::world::Globe;

/// Most tiles painted at once.
const JOBS: usize = 2;

/// Most tiles kept, of every body and level together: about 25 MB of pictures, see [`KEPT_BYTES`].
pub const KEPT: usize = 96;

/// Bytes the pictures of the tiles kept take at most.
#[cfg(test)]
pub const KEPT_BYTES: usize = KEPT * SIDE * SIDE * BYTES;

/// One tile of the cache.
#[derive(Default)]
struct Slot {
    /// The picture, once painted.
    image: Option<Handle<Image>>,
    /// The version of the explored ground the picture shows.
    seen: u64,
    /// The worker painting it, with the version of the ground it reads.
    job: Option<(Task<Vec<u8>>, u64)>,
}

/// The tiles of the body the map shows.
#[derive(Resource, Default)]
pub struct Tiles {
    /// Every tile kept, by body.
    slots: HashMap<(BodyId, TileId), Slot>,
    /// The body shown and the tiles asked for now, each with its column counted without
    /// wrapping.
    wanted: (Option<BodyId>, Vec<(i64, TileId)>),
}

impl Tiles {
    /// Returns the tiles asked for now that need a worker, at most as many as the free workers.
    fn todo(&self, body: BodyId, version: u64) -> Vec<TileId> {
        let running = self
            .slots
            .values()
            .filter(|slot| slot.job.is_some())
            .count();
        self.wanted
            .1
            .iter()
            .map(|(_, id)| *id)
            .filter(|id| {
                self.slots.get(&(body, *id)).is_some_and(|slot| {
                    slot.job.is_none() && !(slot.image.is_some() && slot.seen == version)
                })
            })
            .take(JOBS.saturating_sub(running))
            .collect()
    }

    /// Returns the tiles asked for now whose picture is ready, with the picture.
    pub fn ready(&self) -> Vec<(i64, TileId, Handle<Image>)> {
        let Some(body) = self.wanted.0 else {
            return Vec::new();
        };
        let slot = |id: &TileId| self.slots.get(&(body, *id))?.image.clone();
        let ready = self.wanted.1.iter();
        ready
            .filter_map(|(col, id)| Some((*col, *id, slot(id)?)))
            .collect()
    }
}

/// Returns the picture of a painted tile.
fn picture(data: Vec<u8>) -> Image {
    let size = Extent3d {
        width: SIDE as u32,
        height: SIDE as u32,
        depth_or_array_layers: 1,
    };
    let (format, usage) = (TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    let mut image = Image::new(size, TextureDimension::D2, data, format, usage);
    image.sampler = ImageSampler::nearest();
    image
}

/// Asks for the tiles the lens looks at, or for none while the map is zoomed out.
pub(super) fn want(level: Res<Level>, surface: Res<Surface>, mut tiles: ResMut<Tiles>) {
    let Level::Planet(body) = *level else {
        return;
    };
    let lens = surface.lens;
    let asked = match lens.zoom >= ZOOM_FROM {
        true => visible(lens, level_for(lens.zoom)),
        false => Vec::new(),
    };
    if tiles.wanted.0 == Some(body) && tiles.wanted.1 == asked {
        return;
    }
    for (_, id) in &asked {
        tiles.slots.entry((body, *id)).or_default();
    }
    tiles.wanted = (Some(body), asked);
}

/// Starts workers for the tiles asked for that are not painted, or show older ground.
pub(super) fn start(mut tiles: ResMut<Tiles>, rasters: Res<Rasters>, explored: Res<Explored>) {
    let Some(body) = tiles.wanted.0 else {
        return;
    };
    let version = explored.version(body);
    let todo = tiles.todo(body, version);
    if todo.is_empty() {
        return;
    }
    let Some(globe) = rasters
        .bodies
        .get(&body)
        .and_then(|raster| raster.globe.clone())
    else {
        return;
    };
    let leaves = explored.leaves(body).cloned().unwrap_or_default();
    for id in todo {
        if let Some(slot) = tiles.slots.get_mut(&(body, id)) {
            slot.job = Some((spawn(globe.clone(), leaves.clone(), id), version));
        }
    }
}

/// Starts a worker that paints tile `id` of `globe`, lit where `leaves` reach it.
fn spawn(globe: Arc<Globe>, leaves: Leaves, id: TileId) -> Task<Vec<u8>> {
    AsyncComputeTaskPool::get()
        .spawn(async move { paint(&globe, id, Some(&lit(&leaves, id, globe.radius()))) })
}

/// Puts the tiles the workers finished into pictures.
pub(super) fn collect(mut tiles: ResMut<Tiles>, images: Option<ResMut<Assets<Image>>>) {
    let Some(mut images) = images else {
        return;
    };
    if tiles.slots.values().all(|slot| slot.job.is_none()) {
        return;
    }
    for slot in tiles.slots.values_mut() {
        let done = slot
            .job
            .as_mut()
            .and_then(|(job, _)| block_on(poll_once(job)));
        if let Some(data) = done {
            slot.seen = slot.job.take().map_or(0, |(_, version)| version);
            slot.image = Some(images.add(picture(data)));
        }
    }
}

/// Drops the tiles farthest from what the map shows once more than [`KEPT`] are kept: those of
/// another body first, then those of the levels farthest from the one shown, then those farthest
/// from the centre of the view. The tiles asked for now are the nearest, so they stay, and no
/// level is ever kept whole.
pub(super) fn trim(mut tiles: ResMut<Tiles>, surface: Res<Surface>) {
    if tiles.slots.len() <= KEPT {
        return;
    }
    let shown = tiles.wanted.0;
    let far = |(body, id): &(BodyId, TileId)| {
        let (levels, gap) = away(*id, surface.lens);
        (Some(*body) != shown, levels, gap)
    };
    let mut order: Vec<(BodyId, TileId)> = tiles.slots.keys().copied().collect();
    order.sort_by(|a, b| {
        let (a, b) = (far(a), far(b));
        (b.0, b.1).cmp(&(a.0, a.1)).then(b.2.total_cmp(&a.2))
    });
    let surplus = tiles.slots.len() - KEPT;
    for key in order.into_iter().take(surplus) {
        tiles.slots.remove(&key);
    }
}

#[cfg(test)]
#[path = "tests_cache.rs"]
mod tests;
