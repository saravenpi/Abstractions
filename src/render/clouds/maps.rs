//! Keeping the cover maps: one cube texture per planet whose clouds show, asked for by the two
//! cloud shells, built off the main thread, moved on with universe time a few texels per
//! frame, and dropped when nobody asks any more.

use std::collections::HashMap;
use std::sync::Arc;

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

use super::map::{CHANNELS, CoverMap, budget};
use crate::render::vantage::Scene;
use crate::universe::{BodyId, PlanetSpec, Universe};
use crate::weather::{FACE, Field, TEXELS, texel, wet};
use crate::world::{Globe, Planet};

/// Frames a map is kept after it was last asked for: its readers run in no fixed order
/// against the keeper, so one frame without a request is not yet nobody asking.
const GRACE: u8 = 3;

/// How far a map has got.
enum Stage {
    /// Asked for, not started.
    Asked,
    /// Being built off the main thread.
    Building(Task<CoverMap>),
    /// Built and following universe time.
    Ready(CoverMap),
}

/// One cover map and its texture.
struct Kept {
    /// The cube texture the shaders read.
    image: Handle<Image>,
    /// How far the map has got.
    stage: Stage,
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
            idle: 0,
        });
        kept.idle = 0;
        matches!(kept.stage, Stage::Ready(_)).then(|| kept.image.clone())
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

/// Builds the cover map of `spec` at `seconds` with the snapshot after it ready: the wetness
/// of the ground under every texel, read from `globe` or from a globe of its own, then the
/// field. It reads only its arguments and is safe on any thread.
fn build(spec: &PlanetSpec, globe: Option<Arc<Globe>>, field: Field, seconds: f64) -> CoverMap {
    let globe = globe.unwrap_or_else(|| Arc::new(Globe::of(spec)));
    let ground = (0..TEXELS).map(|index| wet(&globe, texel(index))).collect();
    let mut map = CoverMap::new(field, ground);
    map.advance(seconds, 0);
    map.advance(seconds, TEXELS);
    map
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
    /// Starts building the map of body `id` off the main thread, or nothing for a body that
    /// is not there or has no deck of clouds. The streamed planet lends its own globe.
    fn start(&self, id: BodyId) -> Option<Task<CoverMap>> {
        let spec = self.scene.local.0.as_ref()?.find(id)?;
        let home = self.universe.as_deref().map(Universe::home);
        let field = Field::of(spec, home == Some(id));
        if !field.deck {
            return None;
        }
        let streamed = self.planet.as_deref().filter(|planet| planet.body == id);
        let globe = streamed.map(|planet| planet.globe.clone());
        let (spec, seconds) = (spec.clone(), self.scene.clock.seconds);
        let pool = AsyncComputeTaskPool::get_or_init(TaskPool::default);
        Some(pool.spawn(async move { build(&spec, globe, field, seconds) }))
    }
}

impl Kept {
    /// Moves this map on to `seconds`: starts it, takes it from its task, or works out
    /// `texels` texels of its next snapshot, and writes the texture when its bytes changed.
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
                if let Some(task) = sources.start(id) {
                    (self.image, self.stage) = (images.add(blank()), Stage::Building(task));
                }
                false
            }
            Stage::Building(task) => match block_on(poll_once(task)) {
                Some(mut map) => {
                    map.advance(seconds, texels);
                    self.stage = Stage::Ready(map);
                    true
                }
                None => false,
            },
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

/// Drops the maps nobody asked for lately and moves the others on with universe time.
pub(super) fn keep(
    sources: Sources,
    time: Res<Time>,
    mut maps: ResMut<CloudMaps>,
    mut images: ResMut<Assets<Image>>,
) {
    maps.0.retain(|_, kept| kept.idle < GRACE);
    let texels = budget(time.delta_secs());
    for (id, kept) in &mut maps.0 {
        kept.idle = kept.idle.saturating_add(1);
        kept.refresh(&sources, *id, texels, &mut images);
    }
}
