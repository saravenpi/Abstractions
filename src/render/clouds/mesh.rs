//! Keeping the mesh of the cloud shell: it is built over the part of the sphere the camera sees,
//! and built again when the camera sees past it, and the system parameter that holds it with
//! the material of the shell.

use std::f32::consts::PI;

use bevy::asset::AssetId;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::material::{CloudMaterial, CloudUniform};
use super::shell::{Cap, GROWTH, MARGIN, build, reach};

/// What the mesh of the shell in the asset store was last built for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Built {
    /// The mesh asset that holds it.
    mesh: AssetId<Mesh>,
    /// Radius of the shell in metres.
    radius: f32,
    /// The part of the sphere it covers.
    cap: Cap,
}

impl Built {
    /// Returns true while this mesh still covers a camera under direction `up` that sees `need`
    /// radians of the shell, and is not much wider than that.
    fn fits(&self, mesh: AssetId<Mesh>, radius: f32, up: Vec3, need: f32) -> bool {
        let leaning = self.cap.centre.angle_between(up);
        self.mesh == mesh
            && self.radius.to_bits() == radius.to_bits()
            && leaning + need <= self.cap.half
            && self.cap.half <= 3.0 * (need + MARGIN)
    }
}

/// Builds the mesh of the shell into `handle` again when the camera has seen past what `built`
/// covers, or the shell is another one, and remembers what it built.
fn refresh(
    built: &mut Option<Built>,
    meshes: &mut Assets<Mesh>,
    handle: &Handle<Mesh>,
    sight: (f32, f32, f32),
    up: Vec3,
) {
    let (ground, altitude, radius) = sight;
    let need = reach(ground, altitude, radius);
    if built.is_some_and(|built| built.fits(handle.id(), radius, up, need)) {
        return;
    }
    let cap = Cap {
        centre: up,
        half: (need * GROWTH + MARGIN).min(PI),
    };
    if let Some(mut mesh) = meshes.get_mut(handle) {
        *mesh = build(radius, cap);
        *built = Some(Built {
            mesh: handle.id(),
            radius,
            cap,
        });
    }
}

/// The assets of the cloud shell: its material, its mesh and what the mesh was built for.
#[derive(SystemParam)]
pub struct Shell<'w, 's> {
    /// The material of the shell.
    materials: ResMut<'w, Assets<CloudMaterial>>,
    /// The meshes, one of which is the mesh of the shell.
    meshes: ResMut<'w, Assets<Mesh>>,
    /// What the mesh of the shell was last built for.
    built: Local<'s, Option<Built>>,
}

impl Shell<'_, '_> {
    /// Builds the mesh `handle` again when the camera has seen past what it covers: `sight` is
    /// the radius of the planet, the altitude of the camera and the radius of the shell in
    /// metres, and `up` the unit direction under the camera.
    pub fn remesh(&mut self, handle: &Handle<Mesh>, sight: (f32, f32, f32), up: Vec3) {
        refresh(&mut self.built, &mut self.meshes, handle, sight, up);
    }

    /// Gives the material `handle` its shader values and the cover map it reads.
    pub fn paint(
        &mut self,
        handle: &Handle<CloudMaterial>,
        clouds: CloudUniform,
        cover: &Option<Handle<Image>>,
    ) {
        if let Some(mut material) = self.materials.get_mut(handle) {
            material.clouds = clouds;
            material.cover.clone_from(cover);
        }
    }
}
