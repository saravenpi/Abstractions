//! The look of a ship as shared assets: every mesh and every material is made once, and each
//! ship that is drawn, the one the player flies or a puppet, is dressed from the same handles.

use bevy::prelude::*;

use super::{NOZZLE_Z, Skin, hinges, hull_meshes, leg_mesh, material, plume_mesh};
use crate::ship::cockpit::{Kit, fit};
use crate::ship::glow::Plume;
use crate::ship::legs::Leg;

/// The meshes and the materials of a ship. One set serves every ship.
#[derive(Resource, Debug, Clone)]
pub struct Looks {
    /// The material of each skin, in the order of [`Skin::ALL`].
    pub skins: [Handle<StandardMaterial>; 5],
    /// The hull, one mesh per skin.
    hull: [Handle<Mesh>; 5],
    /// One leg of the gear.
    leg: Handle<Mesh>,
    /// The exhaust.
    plume: Handle<Mesh>,
    /// The cockpit.
    cockpit: Kit,
}

impl Looks {
    /// Creates the meshes and the materials of a ship.
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        let skins = Skin::ALL.map(|skin| materials.add(material(skin)));
        Self {
            hull: hull_meshes().map(|mesh| meshes.add(mesh)),
            leg: meshes.add(leg_mesh()),
            plume: meshes.add(plume_mesh()),
            cockpit: Kit::new(meshes, materials, &skins),
            skins,
        }
    }

    /// Returns every mesh of a ship with its material, for the pipeline warm-up.
    pub fn parts(&self) -> Vec<(Handle<Mesh>, Handle<StandardMaterial>)> {
        let skin = |skin: Skin| self.skins[skin as usize].clone();
        let mut parts: Vec<_> = self.hull.iter().cloned().zip(self.skins.clone()).collect();
        parts.push((self.leg.clone(), skin(Skin::Dark)));
        parts.push((self.plume.clone(), skin(Skin::Glow)));
        parts.extend(self.cockpit.parts());
        parts
    }
}

/// Dresses the ship `entity` from the shared `looks`: the hull, the legs of the gear, the
/// exhaust, and the cockpit with its stick, its lever and its instruments. The engine glow
/// and the exhaust take `glow`, the one material whose colour follows the engine of a ship.
///
/// Nothing here reads the helm, the autopilot or the hatch: the parts are moved from the
/// [`Cockpit`](crate::ship::cockpit::Cockpit) component this adds, so the same call draws
/// the ship the player flies and a puppet whose state comes from elsewhere.
pub fn dress(entity: &mut EntityCommands, looks: &Looks, glow: Handle<StandardMaterial>) {
    let dark = looks.skins[Skin::Dark as usize].clone();
    for hinge in hinges() {
        let leg = (Mesh3d(looks.leg.clone()), MeshMaterial3d(dark.clone()));
        entity.with_child((Leg, leg, Transform::from_translation(hinge)));
    }
    for (index, mesh) in looks.hull.iter().enumerate() {
        let skin = match index == Skin::Glow as usize {
            true => glow.clone(),
            false => looks.skins[index].clone(),
        };
        entity.with_child((Mesh3d(mesh.clone()), MeshMaterial3d(skin)));
    }
    fit(entity, &looks.cockpit);
    let at = Transform::from_xyz(0.0, 0.0, NOZZLE_Z).with_scale(Vec3::new(1.0, 1.0, 0.02));
    entity.with_child((Plume, Mesh3d(looks.plume.clone()), MeshMaterial3d(glow), at));
}
