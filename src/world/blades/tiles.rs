//! The tiles of blades around the camera, each in bands of blades that thin out with the distance: a band is
//! built nearest first off the main thread and landed a few per frame where the distance keeps any of its
//! blades, and dropped past that. Only a change of cube face drops them all; a rebase of the anchor or of the
//! chart keeps every one where it stands.

mod plan;

use std::collections::HashMap;
use std::sync::Arc;

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

pub use self::plan::{Eye, within};
use super::flight::{Flight, fly, land};
use super::tune::TILE;
use super::{BladeAssets, Cropped};
use crate::render::GameCamera;
use crate::space::Anchor;
use crate::world::canopy::Canopy;
use crate::world::chunks::Sited;
use crate::world::focus::Frames;
use crate::world::globe::Chart;
use crate::world::scatter::Carpet;
use crate::world::{CHUNK_SIZE, Terrain};

/// One band of one tile: the tile's key, its low corner over [`TILE`] in whole tiles from the origin of the
/// view, and the index of the band in [`super::tune::BANDS`].
pub type Spot = (IVec2, usize);

/// One band of blades of one tile.
pub struct Tile {
    /// The entity that draws it, none where no blade grows.
    pub entity: Option<Entity>,
    /// True when a cropped patch changed over it since it was built.
    pub stale: bool,
}

/// The bands of tiles around the camera, the turf of the chunks they lie on, and the chart they were built in.
#[derive(Resource, Default)]
pub struct Meadow {
    /// Every band built, by its spot.
    pub(super) tiles: HashMap<Spot, Tile>,
    /// The turf of every chunk a band holds or is built on, none where no ground cover grows. A chunk no band
    /// holds any more is forgotten.
    carpets: HashMap<IVec2, Option<Arc<Carpet>>>,
    /// The bands being built off the main thread, by their spot.
    pub(super) flights: HashMap<Spot, Flight>,
    /// Seed of the globe and chart of the view the bands were built in.
    view: Option<(u64, Chart)>,
    /// The cropped patches the bands were built around.
    cropped: Vec<(DVec3, f32)>,
}

/// What the meadow reads and writes.
#[derive(SystemParam)]
pub struct Grower<'w, 's> {
    pub(super) commands: Commands<'w, 's>,
    pub(super) terrain: Res<'w, Terrain>,
    anchor: Res<'w, Anchor>,
    pub(super) frames: Frames<'w>,
    pub(super) assets: Res<'w, BladeAssets>,
    pub(super) cropped: Res<'w, Cropped>,
    pub(super) meshes: ResMut<'w, Assets<Mesh>>,
    pub(super) meadow: ResMut<'w, Meadow>,
    camera: Query<'w, 's, Entity, With<GameCamera>>,
    transforms: TransformHelper<'w, 's>,
    pub(super) chunks: Query<'w, 's, (&'static Sited, &'static Canopy)>,
}

/// Returns the chunk that holds the middle of tile `key`.
pub fn chunk_of(key: IVec2) -> IVec2 {
    ((key.as_vec2() + 0.5) * TILE / CHUNK_SIZE)
        .floor()
        .as_ivec2()
}

impl Meadow {
    /// Returns true when the bands may stay for `view`, the view of now; otherwise despawns every band and
    /// remembers `view`. When the view is only rooted elsewhere on the same face, the bands stay on their
    /// ground under new keys and only the turf is read again.
    pub(super) fn keep(&mut self, commands: &mut Commands, view: (u64, Chart)) -> bool {
        if self.view == Some(view) {
            return true;
        }
        self.flights.clear();
        let planet = self.view.filter(|old| old.0 == view.0);
        let carried = planet.and_then(|old| old.1.carried(&view.1));
        self.view = Some(view);
        let Some(by) = carried else {
            self.clear(commands);
            return false;
        };
        let by = (by / f64::from(TILE)).round().as_ivec2();
        self.tiles = self
            .tiles
            .drain()
            .map(|((key, band), tile)| ((key + by, band), tile))
            .collect();
        self.carpets.clear();
        true
    }

    /// Despawns every band and forgets the turf.
    fn clear(&mut self, commands: &mut Commands) {
        for tile in self.tiles.values() {
            if let Some(entity) = tile.entity {
                commands.entity(entity).try_despawn();
            }
        }
        self.tiles.clear();
        self.carpets.clear();
        self.flights.clear();
    }

    /// Returns the turf of `chunk`, read once and kept while a band holds the chunk.
    pub(super) fn carpet(&mut self, terrain: &Terrain, chunk: IVec2) -> Option<Arc<Carpet>> {
        let read = || Carpet::of(terrain, chunk).map(Arc::new);
        self.carpets.entry(chunk).or_insert_with(read).clone()
    }

    /// Forgets the turf of every chunk that no band holds or builds on.
    fn evict(&mut self) {
        let (tiles, flights) = (&self.tiles, &self.flights);
        self.carpets.retain(|chunk, _| {
            tiles
                .keys()
                .chain(flights.keys())
                .any(|(k, _)| chunk_of(*k) == *chunk)
        });
    }

    /// Marks stale every band within reach of a cropped patch that was or now is, stops the bands being
    /// built there, and keeps the patches `cropped` of now.
    fn recrop(&mut self, terrain: &Terrain, cropped: &[(DVec3, f32)]) {
        let radius = terrain.globe().radius();
        let near = |(key, _): &Spot, (dir, r): &(DVec3, f32)| {
            let w = (key.as_vec2() + 0.5) * TILE;
            terrain.embed(w.x, 0.0, w.y).normalize().distance(*dir) * radius < f64::from(r + TILE)
        };
        let patches = self.cropped.iter().chain(cropped);
        for (spot, tile) in &mut self.tiles {
            tile.stale |= patches.clone().any(|p| near(spot, p));
        }
        self.flights
            .retain(|spot, _| !patches.clone().any(|p| near(spot, p)));
        self.cropped = cropped.to_vec();
    }
}

/// Returns the chart position and height above sea level in `terrain` of `camera`, from the transforms of this
/// frame. On the frame of a rebase the anchor is new while the `GlobalTransform` of the camera still holds its
/// render position under the old anchor, which would put the camera as far off as the rebase moved it.
pub fn seen(
    terrain: &Terrain,
    anchor: &Anchor,
    transforms: &TransformHelper,
    camera: Entity,
) -> Option<Vec3> {
    let pose = transforms.compute_global_transform(camera).ok()?;
    Some(terrain.locate(anchor.to_frame(pose.translation())))
}

/// Keeps the bands of blades around the camera: drops them all when the camera changed cube face, marks
/// those under a changed cropped patch, drops those past their reach, lands the bands that finished building,
/// then starts building the nearest that want it.
pub fn tend(
    mut g: Grower,
    (mut wanted, mut gone): (Local<Vec<(f32, Spot)>>, Local<Vec<Entity>>),
    mut near: Local<Vec<(DVec3, f32)>>,
) {
    let view = (g.terrain.globe().seed(), g.terrain.chart());
    let camera = g.camera.single().ok();
    let Some(at) = camera.and_then(|c| seen(&g.terrain, &g.anchor, &g.transforms, c)) else {
        return;
    };
    let eye = Eye::new(at.xz(), at.y - g.terrain.height(at.x, at.z));
    let Grower {
        commands,
        terrain,
        cropped,
        meadow,
        ..
    } = &mut g;
    if !meadow.keep(commands, view) {
        return;
    }
    if cropped.is_changed() && cropped.0 != meadow.cropped {
        meadow.recrop(terrain, &cropped.0);
    }
    meadow.plan(eye, &mut wanted, &mut gone);
    for entity in gone.drain(..) {
        commands.entity(entity).try_despawn();
    }
    land(&mut g);
    fly(&mut g, &wanted, &mut near);
}
