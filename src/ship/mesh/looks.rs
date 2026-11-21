//! The look of a ship as shared assets: every mesh and every material is made once, and each
//! ship that is drawn, the one the player flies or a puppet, is dressed from the same handles.

use bevy::camera::visibility::VisibilityRange;
use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;

use super::{FAR, Skin, far_mesh, hinges, hull_meshes, lamp_meshes, leg_mesh, material};
use crate::ship::cockpit::{Kit, fit};
use crate::ship::exhaust::{Exhaust, fit_lights, fit_plumes, jet_mesh};
use crate::ship::legs::Leg;

/// A mesh with the material it is drawn with.
type Part = (Handle<Mesh>, Handle<StandardMaterial>);

/// The meshes and the materials of a ship. One set serves every ship.
#[derive(Resource, Debug, Clone)]
pub struct Looks {
    /// The material of each skin, in the order of [`Skin::ALL`].
    pub skins: [Handle<StandardMaterial>; 5],
    /// The hull, one mesh per skin.
    hull: [Handle<Mesh>; 5],
    /// The far hull, one mesh with its plating painted in its vertex colours.
    far: Part,
    /// The red, green and white lights.
    lamps: [Part; 3],
    /// One leg of the gear.
    leg: Handle<Mesh>,
    /// The exhaust.
    jet: Handle<Mesh>,
    /// The cockpit.
    cockpit: Kit,
}

/// Returns the unlit material of a light of linear colour `rgb`, far above 1 so that it blooms.
fn lamp(rgb: [f32; 3]) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::linear_rgb(rgb[0], rgb[1], rgb[2]),
        unlit: true,
        opaque_render_method: OpaqueRendererMethod::Forward,
        ..default()
    }
}

impl Looks {
    /// Creates the meshes and the materials of a ship.
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        let skins = Skin::ALL.map(|skin| materials.add(material(skin)));
        let paint = StandardMaterial {
            perceptual_roughness: 0.6,
            ..default()
        };
        let colours = [[6.0, 0.3, 0.3], [0.3, 5.0, 0.6], [8.0, 8.0, 7.5]];
        let lamps: Vec<Part> = lamp_meshes()
            .into_iter()
            .zip(colours)
            .map(|(mesh, rgb)| (meshes.add(mesh), materials.add(lamp(rgb))))
            .collect();
        Self {
            hull: hull_meshes().map(|mesh| meshes.add(mesh)),
            far: (meshes.add(far_mesh()), materials.add(paint)),
            lamps: lamps
                .try_into()
                .expect("a mesh for each of the three lights"),
            leg: meshes.add(leg_mesh()),
            jet: meshes.add(jet_mesh()),
            cockpit: Kit::new(meshes, materials, &skins),
            skins,
        }
    }

    /// Returns every mesh of a ship with its material, for the pipeline warm-up.
    pub fn parts(&self) -> Vec<Part> {
        let skin = |skin: Skin| self.skins[skin as usize].clone();
        let mut parts: Vec<_> = self.hull.iter().cloned().zip(self.skins.clone()).collect();
        parts.push((self.leg.clone(), skin(Skin::Dark)));
        parts.push(self.far.clone());
        parts.extend(self.lamps.iter().cloned());
        parts.extend(self.cockpit.parts());
        parts
    }
}

/// Returns the visibility range of the detailed ship: it is drawn up to [`FAR`] metres from the
/// camera, with no fade, which would need a second set of pipelines.
fn near() -> VisibilityRange {
    VisibilityRange {
        start_margin: 0.0..0.0,
        end_margin: FAR..FAR,
        use_aabb: false,
    }
}

/// Returns the visibility range of the far ship: it takes over from [`FAR`] metres on.
fn far() -> VisibilityRange {
    VisibilityRange {
        start_margin: FAR..FAR,
        end_margin: f32::MAX..f32::MAX,
        use_aabb: false,
    }
}

/// Dresses the ship `entity` from the shared `looks`: the hull, the legs of the gear, the lights,
/// the exhaust and its lights, and the cockpit with its stick, its lever and its instruments.
/// The engine glow takes `glow`, the one material whose colour follows the engine of a ship. A
/// ship farther than [`FAR`] metres from the camera is drawn from one mesh instead.
///
/// Nothing here reads the helm, the autopilot or the hatch: the parts are moved from the
/// [`Cockpit`](crate::ship::cockpit::Cockpit) component this adds, and the exhaust from the
/// [`Exhaust`] component, so the same call draws the ship the player flies and a puppet whose
/// state comes from elsewhere.
pub fn dress(entity: &mut EntityCommands, looks: &Looks, glow: Handle<StandardMaterial>) {
    let dark = looks.skins[Skin::Dark as usize].clone();
    for hinge in hinges() {
        let leg = (Mesh3d(looks.leg.clone()), MeshMaterial3d(dark.clone()));
        entity.with_child((Leg, leg, Transform::from_translation(hinge), near()));
    }
    for (index, mesh) in looks.hull.iter().enumerate() {
        if index == Skin::Glass as usize {
            continue;
        }
        let skin = match index == Skin::Glow as usize {
            true => glow.clone(),
            false => looks.skins[index].clone(),
        };
        entity.with_child((Mesh3d(mesh.clone()), MeshMaterial3d(skin), near()));
    }
    for (mesh, material) in &looks.lamps {
        entity.with_child((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            near(),
        ));
    }
    let (mesh, material) = looks.far.clone();
    entity.with_child((Mesh3d(mesh), MeshMaterial3d(material), far()));
    fit(entity, &looks.cockpit);
    entity.insert(Exhaust::default());
    fit_plumes(entity, &looks.jet);
    fit_lights(entity);
}
