//! The tiles of blades around the camera: built nearest first a few per frame, dense inside the inner ring
//! and sparse out to the outer one, dropped past it, and all dropped when the camera flies high or the
//! terrain view changes.

use std::collections::HashMap;

use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::clear::{footprints, taken};
use super::tile::{Ground, grow};
use super::tune::{ABOVE, BUILDS, FAR, LINGER, NEAR, TILE};
use super::{BladeAssets, Cropped};
use crate::render::GameCamera;
use crate::screens::Screen;
use crate::space::{Anchor, Placed};
use crate::world::canopy::Canopy;
use crate::world::chunks::{Sited, pose};
use crate::world::focus::Frames;
use crate::world::globe::Chart;
use crate::world::mesh::Frame;
use crate::world::scatter::Carpet;
use crate::world::{CHUNK_SIZE, Terrain};

/// One tile of blades.
struct Tile {
    /// The entity that draws it, none where no blade grows.
    entity: Option<Entity>,
    /// True when it holds the dense blades.
    full: bool,
    /// True when a cropped patch changed over it since it was built.
    stale: bool,
}

/// The tiles around the camera, the turf of the chunks they lie on, and the chart they were built in.
#[derive(Resource, Default)]
pub struct Meadow {
    /// Every tile built, by its key: its low corner over [`TILE`], in whole tiles from the origin of the view.
    tiles: HashMap<IVec2, Tile>,
    /// The turf of every chunk a tile was built on, none where no ground cover grows.
    carpets: HashMap<IVec2, Option<Carpet>>,
    /// Seed of the globe and chart of the view the tiles were built in.
    view: Option<(u64, Chart)>,
    /// The cropped patches the tiles were built around.
    cropped: Vec<(DVec3, f32)>,
}

/// What the meadow reads and writes.
#[derive(SystemParam)]
pub struct Grower<'w, 's> {
    commands: Commands<'w, 's>,
    terrain: Res<'w, Terrain>,
    anchor: Res<'w, Anchor>,
    frames: Frames<'w>,
    assets: Res<'w, BladeAssets>,
    cropped: Res<'w, Cropped>,
    meshes: ResMut<'w, Assets<Mesh>>,
    meadow: ResMut<'w, Meadow>,
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    chunks: Query<'w, 's, (&'static Sited, &'static Canopy)>,
}

impl Meadow {
    /// Returns true when the tiles may stay for `view`, the view of now or none when the camera is high;
    /// otherwise despawns every tile and remembers `view`. When the view is only rooted elsewhere on the
    /// same face, the tiles stay on their ground under new keys and only the turf is read again.
    fn keep(&mut self, commands: &mut Commands, view: Option<(u64, Chart)>) -> bool {
        if self.view == view && view.is_some() {
            return true;
        }
        let planet = self.view.zip(view).filter(|(old, new)| old.0 == new.0);
        let carried = planet.and_then(|(old, new)| old.1.carried(&new.1));
        self.view = view;
        let Some(by) = carried else {
            self.clear(commands);
            return false;
        };
        let by = (by / f64::from(TILE)).round().as_ivec2();
        self.tiles = self
            .tiles
            .drain()
            .map(|(key, tile)| (key + by, tile))
            .collect();
        self.carpets.clear();
        true
    }

    /// Despawns every tile and forgets the turf.
    fn clear(&mut self, commands: &mut Commands) {
        for tile in self.tiles.values() {
            if let Some(entity) = tile.entity {
                commands.entity(entity).try_despawn();
            }
        }
        self.tiles.clear();
        self.carpets.clear();
    }

    /// Marks stale every tile within reach of a cropped patch that was or now is, and keeps the patches
    /// `cropped` of now.
    fn recrop(&mut self, terrain: &Terrain, cropped: &[(DVec3, f32)]) {
        let radius = terrain.globe().radius();
        let near = |key: &IVec2, (dir, r): &(DVec3, f32)| {
            let w = (key.as_vec2() + 0.5) * TILE;
            terrain.embed(w.x, 0.0, w.y).normalize().distance(*dir) * radius < f64::from(r + TILE)
        };
        let patches = self.cropped.iter().chain(cropped);
        for (key, tile) in &mut self.tiles {
            tile.stale |= patches.clone().any(|p| near(key, p));
        }
        self.cropped = cropped.to_vec();
    }

    /// Drops the tiles past the outer ring of chart position `at`, and returns the nearest tiles that want
    /// building: missing, stale, or holding dense blades where sparse ones are wanted or the other way.
    fn plan(&mut self, commands: &mut Commands, at: Vec2, wanted: &mut Vec<(f32, IVec2, bool)>) {
        let centre = |key: &IVec2| (key.as_vec2() + 0.5) * TILE;
        self.tiles.retain(|key, tile| {
            let keep = centre(key).distance(at) < FAR + TILE + LINGER;
            if let (false, Some(entity)) = (keep, tile.entity) {
                commands.entity(entity).try_despawn();
            }
            keep
        });
        wanted.clear();
        let reach = ((FAR + TILE) / TILE).ceil() as i32;
        let home = (at / TILE).floor().as_ivec2();
        for (x, y) in (-reach..=reach).flat_map(|x| (-reach..=reach).map(move |y| (x, y))) {
            let key = home + IVec2::new(x, y);
            let away = centre(&key).distance(at);
            let dense = away < NEAR + TILE;
            let held = self.tiles.get(&key);
            let fits = held.is_some_and(|t| {
                !t.stale && (t.full == dense || (t.full && away < NEAR + TILE + LINGER))
            });
            if away < FAR + TILE && !fits {
                wanted.push((away, key, dense));
            }
        }
        wanted.sort_by(|a, b| a.0.total_cmp(&b.0));
    }
}

/// Returns the mesh of the blades of tile `key`, dense when `full`, clear of the footprints `near`, and the
/// frame it is built in, or nothing where no blade grows.
fn blades_of(
    g: &mut Grower,
    key: IVec2,
    full: bool,
    near: &[(DVec3, f32)],
) -> Option<(Mesh, Frame)> {
    let w = (key.as_vec2() + 0.5) * TILE;
    let chunk = (w / CHUNK_SIZE).floor().as_ivec2();
    let terrain = g.terrain.clone();
    let carpets = &mut g.meadow.carpets;
    let carpet = carpets
        .entry(chunk)
        .or_insert_with(|| Carpet::of(&terrain, chunk));
    let radius = terrain.globe().radius();
    let ground = Ground::read(&terrain, key);
    let clear = |p| taken(p, near, &g.cropped, radius);
    let (blades, frame) = grow((&terrain, &ground), key, carpet.as_ref()?, full, &clear);
    Some((blades.into_mesh()?, frame))
}

/// Spawns the entity that draws `mesh` in `frame`, posed from the anchor of now.
fn spawn_tile(g: &mut Grower, mesh: Mesh, frame: Frame) -> Entity {
    let sited = Sited::from(frame);
    let alone = Placed {
        pos: sited.pos,
        rot: sited.rot,
    };
    let link = g.frames.link();
    let (placed, transform) = link.map_or((alone, Transform::default()), |(anchor, link)| {
        pose(&anchor, &link, &sited)
    });
    let look = (
        Mesh3d(g.meshes.add(mesh)),
        MeshMaterial3d(g.assets.material.clone()),
        NotShadowCaster,
    );
    let at = (sited, placed, transform, Visibility::default());
    let name = (Name::new("Blades"), DespawnOnEnter(Screen::Title));
    g.commands.spawn((name, look, at)).id()
}

/// Builds tile `key`, dense when `full`, clear of the footprints `near`, and spawns or replaces its entity.
fn build(g: &mut Grower, key: IVec2, full: bool, near: &[(DVec3, f32)]) {
    if let Some(entity) = g.meadow.tiles.remove(&key).and_then(|t| t.entity) {
        g.commands.entity(entity).try_despawn();
    }
    let drawn = blades_of(g, key, full, near);
    let entity = drawn.map(|(mesh, frame)| spawn_tile(g, mesh, frame));
    let tile = Tile {
        entity,
        full,
        stale: false,
    };
    g.meadow.tiles.insert(key, tile);
}

/// Keeps the tiles of blades around the camera: drops them all when the view changed or the camera is high
/// over the ground, marks those under a changed cropped patch, then builds the nearest that want it.
pub fn tend(
    mut g: Grower,
    mut wanted: Local<Vec<(f32, IVec2, bool)>>,
    mut near: Local<Vec<(DVec3, f32)>>,
) {
    let view = Some((g.terrain.globe().seed(), g.terrain.chart()));
    let Ok(camera) = g.camera.single().map(|c| c.translation()) else {
        return;
    };
    let at = g.terrain.locate(g.anchor.to_frame(camera));
    let high = at.y - g.terrain.height(at.x, at.z) > ABOVE;
    let Grower {
        commands,
        terrain,
        cropped,
        meadow,
        ..
    } = &mut g;
    if !meadow.keep(commands, view.filter(|_| !high)) {
        return;
    }
    if cropped.is_changed() && cropped.0 != meadow.cropped {
        meadow.recrop(terrain, &cropped.0);
    }
    meadow.plan(commands, at.xz(), &mut wanted);
    for &(_, key, full) in wanted.iter().take(BUILDS) {
        let centre = (key.as_vec2() + 0.5) * TILE;
        let planet = g.terrain.embed(centre.x, 0.0, centre.y);
        footprints(&g.chunks, planet, &mut near);
        build(&mut g, key, full, &near);
    }
}
