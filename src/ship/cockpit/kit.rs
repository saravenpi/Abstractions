//! The meshes and materials of a cockpit, made once and shared by every ship.

use bevy::prelude::*;

use super::shapes::{gear_mesh, pedal_mesh, strap_mesh, switch_mesh};
use crate::ship::mesh::{Skin, cabin_meshes, canopy_meshes, lever_mesh, stick_mesh, tub_mesh};

/// A mesh with the material it is drawn with.
pub type Drawn = (Handle<Mesh>, Handle<StandardMaterial>);

/// Returns the lining of the tub: a matte dark grey, lighter than the frame of the furniture
/// and darker than its plates and the body of the pilot, so that all three read against it.
fn lining() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb(0.22, 0.23, 0.26),
        perceptual_roughness: 0.9,
        ..default()
    }
}

/// The meshes and materials of a cockpit.
#[derive(Debug, Clone)]
pub struct Kit {
    /// The fixed interior: the frame, the plates and the accents of the cabin, and the tub.
    pub(super) fixed: Vec<Drawn>,
    /// The centre stick, the throttle lever, the gear lever and the warp switch.
    pub(super) levers: [Drawn; 4],
    /// The rudder pedal.
    pub(super) pedal: Drawn,
    /// The loose strap.
    pub(super) strap: Drawn,
    /// The glass and the frame of the canopy, built around its hinge.
    pub(super) canopy: [Drawn; 2],
    /// The unit square, centred, that the displays are drawn on.
    pub(super) screen: Handle<Mesh>,
}

impl Kit {
    /// Creates the meshes and the materials of the cockpit, with the `skins` of the ship.
    pub fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        skins: &[Handle<StandardMaterial>; 5],
    ) -> Self {
        let trim = skins[Skin::Trim as usize].clone();
        let dark = skins[Skin::Dark as usize].clone();
        let mut fixed: Vec<Drawn> = cabin_meshes()
            .into_iter()
            .map(|(skin, mesh)| (meshes.add(mesh), skins[skin as usize].clone()))
            .collect();
        fixed.push((meshes.add(tub_mesh()), materials.add(lining())));
        let mut add =
            |mesh: Mesh, skin: &Handle<StandardMaterial>| (meshes.add(mesh), skin.clone());
        Self {
            fixed,
            levers: [
                add(stick_mesh(), &trim),
                add(lever_mesh(), &trim),
                add(gear_mesh(), &trim),
                add(switch_mesh(), &trim),
            ],
            pedal: add(pedal_mesh(), &dark),
            strap: add(strap_mesh(), &trim),
            canopy: canopy_meshes()
                .map(|(skin, mesh)| (meshes.add(mesh), skins[skin as usize].clone())),
            screen: meshes.add(Rectangle::new(1.0, 1.0)),
        }
    }

    /// Returns every part of the cockpit that has a material of its own, for the pipeline
    /// warm-up.
    pub fn parts(&self) -> impl Iterator<Item = Drawn> + '_ {
        let moving = self
            .levers
            .iter()
            .chain([&self.pedal, &self.strap])
            .chain(&self.canopy);
        self.fixed.iter().chain(moving).cloned()
    }
}
