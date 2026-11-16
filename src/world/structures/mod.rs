//! Procedural structures: ruins, cabins, temples and more, placed per cell and merged per chunk.

#[cfg(test)]
mod audit;
mod brush;
mod cabin;
mod camp;
mod camp2;
mod coast;
mod desert;
mod dock;
mod draw;
mod dress;
mod fishing;
mod flat;
mod flicker;
mod glow;
mod hut;
mod lights;
mod lights_sys;
mod palette;
mod parts;
mod roofs;
mod ruins;
mod rules;
mod shrine;
mod site;
mod smoke;
mod steps;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_lights;
#[cfg(test)]
mod tests_village;
mod village;

use avian3d::prelude::*;
use bevy::prelude::*;

use super::Terrain;
use super::props::PropAssets;
use super::scatter::chunk_center;
use brush::{Build, Part, Shape};
use glow::LightPoint;
use site::sites_in_chunk;

pub use site::{CELL, Kind, Site, origins_near, structure_in_cell};

/// The merged mesh and solid parts of every structure in one chunk.
pub struct StructureData {
    mesh: Mesh,
    parts: Vec<Part>,
    lights: Vec<LightPoint>,
}

impl StructureData {
    /// Sets whether the merged mesh gets an acceleration structure.
    pub fn set_raytracing(&mut self, on: bool) {
        self.mesh.enable_raytracing = on;
    }
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
    camp::camp,
    village::draw,
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
        lights: build.lights,
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
    lights::spawn_lights(commands, chunk, &data.lights);
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

/// Registers the structure lights: flame dressing, lamp choice, flicker and the smoke pool.
pub fn plugin(app: &mut App) {
    app.init_resource::<lights_sys::Fires>()
        .add_systems(Startup, (lights::setup, smoke::spawn_puffs).chain())
        .add_systems(Update, lights::dress)
        .add_systems(
            Update,
            (lights_sys::select, lights_sys::animate, smoke::drift)
                .chain()
                .run_if(crate::screens::playing),
        );
}
