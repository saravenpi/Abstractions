//! Procedural structures: ruins, cabins, temples and more, placed per cell and merged per chunk.

mod brush;
mod cabin;
mod camp;
mod coast;
mod desert;
mod dock;
mod draw;
mod hut;
mod palette;
mod parts;
mod ruins;
mod rules;
mod shrine;
mod site;
#[cfg(test)]
mod tests;

use avian3d::prelude::*;
use bevy::prelude::*;

use super::Terrain;
use super::props::PropAssets;
use super::scatter::chunk_center;
use brush::{Build, Part, Shape};
use site::sites_in_chunk;

pub use site::{CELL, Kind, Site, origins_near, structure_in_cell};

/// The merged mesh and solid parts of every structure in one chunk.
pub struct StructureData {
    mesh: Mesh,
    parts: Vec<Part>,
}

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
    cabin::village,
    camp::camp,
];

/// Builds the structures whose origin lies in chunk `coord`, in chunk space. None when it has none.
pub fn build_chunk(terrain: &Terrain, coord: IVec2) -> Option<StructureData> {
    let sites = sites_in_chunk(terrain, coord);
    let mut build = Build::new(terrain, chunk_center(coord), *sites.first()?);
    for site in sites {
        build.begin(site);
        DRAW[site.kind as usize](&mut build);
    }
    Some(StructureData {
        mesh: build.mesh.into_mesh(),
        parts: build.parts,
    })
}

/// Spawns the structure mesh with its compound collider as a child of the chunk.
pub fn spawn_structures(
    commands: &mut Commands,
    chunk: Entity,
    assets: &PropAssets,
    meshes: &mut Assets<Mesh>,
    data: Option<StructureData>,
) {
    let Some(data) = data else {
        return;
    };
    let shapes = data.parts.iter().map(|p| {
        let shape = match p.shape {
            Shape::Cuboid(size) => Collider::cuboid(size.x, size.y, size.z),
            Shape::Cylinder(radius, height) => Collider::cylinder(radius, height),
        };
        (p.pos, p.rot, shape)
    });
    let collider = Collider::compound(shapes.collect());
    let mesh = meshes.add(data.mesh);
    commands.entity(chunk).with_children(|c| {
        c.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(assets.material.clone()),
            Transform::default(),
            collider,
        ));
    });
}
