//! Procedural structures: ruins, cabins, temples and more, placed per cell and merged per chunk.

mod around;
#[cfg(test)]
mod audit;
mod brush;
mod cabin;
mod camp;
mod camp2;
mod canvas;
mod carry;
mod coast;
mod desert;
mod dock;
mod doors;
mod draw;
mod dress;
mod eaves;
mod fishing;
mod flame;
mod flat;
mod flicker;
#[cfg(test)]
mod gaps;
mod geo;
#[cfg(test)]
mod globes;
mod glow;
mod hold;
mod hut;
mod lights;
mod lights_sys;
#[cfg(test)]
mod measure;
mod memo;
mod pads;
mod palette;
mod parts;
mod plates;
#[cfg(test)]
mod probe;
mod roofs;
mod ruins;
mod rules;
mod shrine;
mod site;
mod smoke;
mod steps;
mod tags;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_air;
#[cfg(test)]
mod tests_class;
#[cfg(test)]
mod tests_doors;
#[cfg(test)]
mod tests_edges;
#[cfg(test)]
mod tests_embed;
#[cfg(test)]
mod tests_faces;
#[cfg(test)]
mod tests_globe;
#[cfg(test)]
mod tests_hang;
#[cfg(test)]
mod tests_headroom;
#[cfg(test)]
mod tests_interior;
#[cfg(test)]
mod tests_landing;
#[cfg(test)]
mod tests_lights;
#[cfg(test)]
mod tests_plates;
#[cfg(test)]
mod tests_river;
#[cfg(test)]
mod tests_roofs;
#[cfg(test)]
mod tests_streets;
#[cfg(test)]
mod tests_survey;
#[cfg(test)]
mod tests_village;
mod village;

use avian3d::prelude::*;
use bevy::prelude::*;

use super::Terrain;
use super::mesh::Frame;
use super::props::PropAssets;
use super::scatter::chunk_center;
use brush::{Build, Part, Shape};
use canvas::join;
use carry::Target;
use glow::LightPoint;
use site::sites_in_chunk;

pub use around::{AROUND, Surroundings, Survey, survey};
#[cfg(test)]
pub use globes::stilt_village;
pub use pads::{FloorKind, Nearby};
pub use site::{CELL, Kind, Site, origins_near, structure_in_cell};
pub use village::{Entry, entries};

/// The merged mesh, the collider and the lights of every structure in one chunk. Plain data, built off the
/// main thread.
pub struct StructureData {
    mesh: Mesh,
    collider: Option<Collider>,
    lights: Vec<LightPoint>,
    rigid: bool,
}

/// Marks the structure mesh and the lights of a chunk built by [`build_in`]: their geometry already is in
/// the frame of the chunk, so nothing may warp it again.
#[derive(Component, Debug, Clone, Copy)]
pub struct Embedded;

/// Draw functions indexed by `Kind as usize`.
const DRAW: [fn(&mut Build); 11] = [
    ruins::stone_circle,
    ruins::tower,
    cabin::cabin,
    coast::lighthouse,
    desert::pyramid,
    ruins::obelisk,
    dock::dock,
    hut::stilt_hut,
    shrine::shrine,
    camp::camp,
    village::draw,
];

/// Returns the compound collider of `parts`, or nothing when there is no solid part.
fn compound(parts: &[Part]) -> Option<Collider> {
    let shapes = parts.iter().map(|p| {
        let shape = match p.shape {
            Shape::Cuboid(size) => Collider::cuboid(size.x, size.y, size.z),
            Shape::Cylinder(radius, height) => Collider::cylinder(radius, height),
        };
        (p.pos, p.rot, shape)
    });
    (!parts.is_empty()).then(|| Collider::compound(shapes.collect()))
}

/// Builds the structures whose origin lies in chunk `coord` of the chart of `terrain`, carried to `target`.
/// Each site is drawn on the metric view around its own origin.
fn build(terrain: &Terrain, coord: IVec2, target: Target) -> Option<StructureData> {
    let sites = sites_in_chunk(terrain, coord);
    let (mut meshes, mut parts, mut lights) = (Vec::new(), Vec::new(), Vec::new());
    let mut count = 0;
    for site in &sites {
        let view = terrain.around(site.origin.x, site.origin.y);
        let mut build = Build::on((terrain, &view), chunk_center(coord), *site);
        build.count = count;
        DRAW[site.kind as usize](&mut build);
        count = build.count;
        let out = build.finish(target.clone());
        meshes.push(out.mesh);
        parts.extend(out.parts);
        lights.extend(out.lights);
    }
    (!sites.is_empty()).then(|| StructureData {
        mesh: join(meshes),
        collider: compound(&parts),
        lights,
        rigid: matches!(target, Target::Frame(_)),
    })
}

/// Builds the structures whose origin lies in chunk `coord`, in the chunk space of the chart of `terrain`:
/// `(x, z)` are chart offsets from the chunk center and `y` is the height above sea level. None when the
/// chunk has none.
///
/// On the globe every building is laid out rigid and upright at its own origin and then written in chart
/// coordinates, so that the point by point warp of `chunks::embed` puts it back where it belongs. Prefer
/// [`build_in`], which needs no warp. Runs on any thread and reads no world.
#[cfg(test)]
pub fn build_chunk(terrain: &Terrain, coord: IVec2) -> Option<StructureData> {
    let center = chunk_center(coord);
    let chart = Target::Chart {
        terrain: terrain.clone(),
        center,
    };
    build(terrain, coord, chart)
}

/// Builds the structures whose origin lies in chunk `coord` of the chart of `terrain` directly in `frame`,
/// the rigid frame of the chunk on the planet: mesh, collider and lights need no further embedding, and
/// [`spawn_structures`] marks what it spawns with [`Embedded`]. None when the chunk has none.
///
/// Runs on any thread and reads no world.
pub fn build_in(terrain: &Terrain, coord: IVec2, frame: &Frame) -> Option<StructureData> {
    build(terrain, coord, Target::Frame(*frame))
}

/// Spawns the structure mesh and the lights of `data` as children of the chunk. Returns the mesh child and
/// the compound collider it carries while the chunk is solid, which the caller attaches.
pub fn spawn_structures(
    commands: &mut Commands,
    chunk: Entity,
    assets: &PropAssets,
    meshes: &mut Assets<Mesh>,
    data: Option<StructureData>,
) -> Option<(Entity, Option<Collider>)> {
    let data = data?;
    lights::spawn_lights(commands, chunk, &data.lights, data.rigid);
    let part = (
        Mesh3d(meshes.add(data.mesh)),
        MeshMaterial3d(assets.material.clone()),
        Transform::default(),
        ChildOf(chunk),
    );
    let mut child = commands.spawn(part);
    if data.rigid {
        child.insert(Embedded);
    }
    Some((child.id(), data.collider))
}

/// Registers the structure lights: flame dressing, lamp choice, flicker and the smoke pool.
pub fn plugin(app: &mut App) {
    app.init_resource::<lights_sys::Fires>()
        .init_resource::<Surroundings>()
        .add_systems(Update, around::watch.run_if(crate::screens::playing))
        .add_systems(Startup, (lights::setup, smoke::spawn_puffs).chain())
        .add_systems(Update, lights::dress)
        .add_systems(
            Update,
            (lights_sys::select, lights_sys::animate, smoke::drift)
                .chain()
                .run_if(crate::screens::playing),
        );
}
