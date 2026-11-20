//! Keeping the cover maps: one cube texture per planet whose clouds show, asked for by the two
//! cloud shells, built off the main thread, moved on with universe time a few texels per
//! frame, and dropped when nobody asks any more.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemParam;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};
use bevy::tasks::{
    AsyncComputeTaskPool, Task, TaskPool, block_on, futures_lite::future::poll_once,
};

use super::map::{CHANNELS, CoverMap, Slab, budget, slab};
use crate::render::vantage::Scene;
use crate::universe::{BodyId, Universe};
use crate::weather::{FACE, Field, TEXELS};
use crate::world::{Globe, Planet};

/// Frames a map is kept after it was last asked for: its readers run in no fixed order
/// against the keeper, so one frame without a request is not yet nobody asking.
const GRACE: u8 = 3;

/// Faces of the cube grid of a cover map, each built by a task of its own.
const FACES: usize = 6;

/// A map being built off the main thread, one task per face of the grid.
struct Build {
    /// The cloud field the map is built from.
    field: Field,
    /// Universe time the map is built for.
    seconds: f64,
    /// The faces still being worked out, with their number.
    tasks: Vec<(usize, Task<Slab>)>,
    /// The faces that are done, with their number.
    slabs: Vec<(usize, Slab)>,
}

impl Build {
    /// Takes the faces that finished, and returns the map once all of them have.
    fn poll(&mut self) -> Option<CoverMap> {
        for (face, mut task) in std::mem::take(&mut self.tasks) {
            match block_on(poll_once(&mut task)) {
                Some(slab) => self.slabs.push((face, slab)),
                None => self.tasks.push((face, task)),
            }
        }
        if !self.tasks.is_empty() {
            return None;
        }
        self.slabs.sort_by_key(|(face, _)| *face);
        let slabs = self.slabs.drain(..).map(|(_, slab)| slab).collect();
        Some(CoverMap::from_slabs(self.field, self.seconds, slabs))
    }
}

/// How far a map has got.
enum Stage {
    /// Asked for, not started.
    Asked,
    /// Being built off the main thread, for the first time or again after a jump in time.
    Building(Build),
    /// Built and following universe time.
    Ready(CoverMap),
}

/// One cover map and its texture.
struct Kept {
    /// The cube texture the shaders read.
    image: Handle<Image>,
    /// How far the map has got.
    stage: Stage,
    /// Whether the texture has held the cover once: it keeps showing it while the map is built
    /// again after a jump in time.
    shown: bool,
    /// Frames since it was last asked for.
    idle: u8,
}

/// The cover maps of the planets whose clouds show.
#[derive(Resource, Default)]
pub struct CloudMaps(HashMap<BodyId, Kept>);

impl CloudMaps {
    /// Asks for the cover map of body `id` to be kept, and returns its texture once it holds
    /// the cover. A body without clouds never gets one.
    pub fn want(&mut self, id: BodyId) -> Option<Handle<Image>> {
        let kept = self.0.entry(id).or_insert_with(|| Kept {
            image: Handle::default(),
            stage: Stage::Asked,
            shown: false,
            idle: 0,
        });
        kept.idle = 0;
        kept.shown.then(|| kept.image.clone())
    }
}

/// Returns the empty cube texture of a cover map: six faces of two channels, read smoothly.
fn blank() -> Image {
    let side = FACE as u32;
    let mut image = Image::new(
        Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: 6,
        },
        TextureDimension::D2,
        vec![0; TEXELS * CHANNELS],
        TextureFormat::Rg8Unorm,
        RenderAssetUsages::default(),
    );
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
    image.sampler = ImageSampler::linear();
    image
}

/// What the maps are built from: the bodies, universe time and the planet the world streams.
#[derive(SystemParam)]
pub(super) struct Sources<'w> {
    /// The star system around the camera and universe time.
    scene: Scene<'w>,
    /// The universe of the world being played, which names the home planet.
    universe: Option<Res<'w, Universe>>,
    /// The planet the world streams, whose globe is already built.
    planet: Option<Res<'w, Planet>>,
}

impl Sources<'_> {
    /// Starts building the map of body `id` off the main thread, a task per face, or nothing
    /// for a body that is not there or has no deck of clouds. The streamed planet lends its own
    /// globe, and any other planet gets one that the first task to need it builds.
    fn start(&self, id: BodyId) -> Option<Build> {
        let spec = self.scene.local.0.as_ref()?.find(id)?;
        let home = self.universe.as_deref().map(Universe::home);
        let field = Field::of(spec, home == Some(id));
        if !field.deck {
            return None;
        }
        let globe = Arc::new(OnceLock::new());
        if let Some(planet) = self.planet.as_deref().filter(|planet| planet.body == id) {
            let _ = globe.set(planet.globe.clone());
        }
        let (spec, seconds) = (spec.clone(), self.scene.clock.seconds);
        let pool = AsyncComputeTaskPool::get_or_init(TaskPool::default);
        let tasks = (0..FACES)
            .map(|face| {
                let (spec, globe) = (spec.clone(), globe.clone());
                let work = async move {
                    let globe = globe.get_or_init(|| Arc::new(Globe::of(&spec)));
                    slab(&field, globe, face, seconds)
                };
                (face, pool.spawn(work))
            })
            .collect();
        Some(Build {
            field,
            seconds,
            tasks,
            slabs: Vec::new(),
        })
    }
}

impl Kept {
    /// Moves this map on to `seconds`: starts it, takes it from its tasks, starts it again
    /// after a jump in time, or works out `texels` texels of its next snapshot, and writes
    /// the texture when its bytes changed.
    fn refresh(
        &mut self,
        sources: &Sources,
        id: BodyId,
        texels: usize,
        images: &mut Assets<Image>,
    ) {
        let seconds = sources.scene.clock.seconds;
        let changed = match &mut self.stage {
            Stage::Asked => {
                if let Some(build) = sources.start(id) {
                    (self.image, self.stage) = (images.add(blank()), Stage::Building(build));
                }
                false
            }
            Stage::Building(build) => match build.poll() {
                Some(map) => {
                    (self.stage, self.shown) = (Stage::Ready(map), true);
                    true
                }
                None => false,
            },
            Stage::Ready(map) if map.stale(seconds) => {
                if let Some(build) = sources.start(id) {
                    self.stage = Stage::Building(build);
                }
                false
            }
            Stage::Ready(map) => map.advance(seconds, texels),
        };
        if let (true, Stage::Ready(map)) = (changed, &self.stage) {
            write(images, &self.image, map.bytes());
        }
    }
}

/// Writes `bytes` over the pixels of the texture `image`, which keeps its size.
fn write(images: &mut Assets<Image>, image: &Handle<Image>, bytes: &[u8]) {
    let Some(mut image) = images.get_mut(image) else {
        return;
    };
    if let Some(data) = image.data.as_mut().filter(|data| data.len() == bytes.len()) {
        data.copy_from_slice(bytes);
    }
}

/// Forgets every map when the world is left: the next world may reuse the names of its bodies.
pub(super) fn clear(mut maps: ResMut<CloudMaps>) {
    maps.0.clear();
}

/// Drops the maps nobody asked for lately and moves the others on with universe time. The
/// frame's [`budget`] of texels is shared by the maps that follow time.
pub(super) fn keep(
    sources: Sources,
    time: Res<Time>,
    mut maps: ResMut<CloudMaps>,
    mut images: ResMut<Assets<Image>>,
) {
    maps.0.retain(|_, kept| kept.idle < GRACE);
    let ready = maps
        .0
        .values()
        .filter(|kept| matches!(kept.stage, Stage::Ready(_)));
    let texels = budget(time.delta_secs()) / ready.count().max(1);
    for (id, kept) in &mut maps.0 {
        kept.idle = kept.idle.saturating_add(1);
        kept.refresh(&sources, *id, texels, &mut images);
    }
}
