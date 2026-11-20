//! The cache of tiles: the tiles the lens looks at are painted on workers, kept while they are
//! used and dropped when the cache is full.

use std::collections::HashMap;
use std::sync::Arc;

use abstractions_net::leaves::Leaves;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::Rasters;
use super::tile::{SIDE, TileId, ZOOM_FROM, level_for, visible};
use super::tile_paint::{lit, paint};
use crate::map::explored::Explored;
use crate::map::{Level, Surface};
use crate::universe::BodyId;
use crate::world::Globe;

/// Most tiles painted at once.
const JOBS: usize = 2;

/// Most tiles kept: about 25 MB of pictures.
const KEPT: usize = 96;

/// One tile of the cache.
#[derive(Default)]
struct Slot {
    /// The picture, once painted.
    image: Option<Handle<Image>>,
    /// The version of the explored ground the picture shows.
    seen: u64,
    /// The worker painting it, with the version of the ground it reads.
    job: Option<(Task<Vec<u8>>, u64)>,
    /// The frame the tile was last asked for.
    used: u64,
}

/// The tiles of the body the map shows.
#[derive(Resource, Default)]
pub struct Tiles {
    /// Every tile kept, by body.
    slots: HashMap<(BodyId, TileId), Slot>,
    /// The body shown and the tiles asked for now, each with its column counted without
    /// wrapping.
    wanted: (Option<BodyId>, Vec<(i64, TileId)>),
    /// Counts frames.
    frame: u64,
}

impl Tiles {
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
    tiles.frame += 1;
    let frame = tiles.frame;
    for (_, id) in &asked {
        tiles.slots.entry((body, *id)).or_default().used = frame;
    }
    tiles.wanted = (Some(body), asked);
}

/// Starts workers for the tiles asked for that are not painted, or show older ground.
pub(super) fn start(mut tiles: ResMut<Tiles>, rasters: Res<Rasters>, explored: Res<Explored>) {
    let Some(body) = tiles.wanted.0 else {
        return;
    };
    let Some(globe) = rasters
        .bodies
        .get(&body)
        .and_then(|raster| raster.globe.clone())
    else {
        return;
    };
    let version = explored.version(body);
    let leaves = explored.leaves(body).cloned().unwrap_or_default();
    let mut running = tiles
        .slots
        .values()
        .filter(|slot| slot.job.is_some())
        .count();
    let asked: Vec<TileId> = tiles.wanted.1.iter().map(|(_, id)| *id).collect();
    for id in asked {
        let Some(slot) = tiles.slots.get_mut(&(body, id)) else {
            continue;
        };
        let current = slot.image.is_some() && slot.seen == version;
        if running < JOBS && slot.job.is_none() && !current {
            slot.job = Some((spawn(globe.clone(), leaves.clone(), id), version));
            running += 1;
        }
    }
}

/// Starts a worker that paints tile `id` of `globe`, lit where `leaves` reach it.
fn spawn(globe: Arc<Globe>, leaves: Leaves, id: TileId) -> Task<Vec<u8>> {
    AsyncComputeTaskPool::get().spawn(async move { paint(&globe, id, Some(&lit(&leaves, id))) })
}

/// Puts the tiles the workers finished into pictures.
pub(super) fn collect(mut tiles: ResMut<Tiles>, images: Option<ResMut<Assets<Image>>>) {
    let Some(mut images) = images else {
        return;
    };
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

/// Drops the tiles asked for longest ago once more than [`KEPT`] are kept.
pub(super) fn trim(mut tiles: ResMut<Tiles>) {
    if tiles.slots.len() <= KEPT {
        return;
    }
    let mut order: Vec<((BodyId, TileId), u64)> = tiles
        .slots
        .iter()
        .map(|(key, slot)| (*key, slot.used))
        .collect();
    order.sort_by_key(|(_, used)| *used);
    let surplus = tiles.slots.len() - KEPT;
    for (key, _) in order.into_iter().take(surplus) {
        tiles.slots.remove(&key);
    }
}
