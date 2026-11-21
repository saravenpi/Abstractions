//! The dust the jets of a ship raise off loose ground: the lift nozzles in a hover or a landing,
//! and the main engines when they blow on the ground close under a tail. Each jet is followed
//! from its nozzle to where its flow meets the ground; within its reach, which grows with its
//! thrust, it raises sand, earth, snow or the dust of a bare world, never rock, water or ice. The
//! puffs fly out along the ground as fast as the jet still blows there, slow to the wind, settle
//! toward the centre of the planet at the speed of their grains and fade over a few seconds.
//!
//! Cost: a fixed pool of [`COUNT`] puffs, made once, one shared disc mesh and one material per
//! soil, so the puffs of one soil draw in one batch. Nothing is spawned or allocated per frame;
//! the ground is read at most twice per jet per frame and once per puff every few metres it
//! drifts.

mod fly;
mod jet;
mod look;
mod puff;
mod soil;
mod tune;

#[cfg(test)]
mod rig;
#[cfg(test)]
mod tests_draw;
#[cfg(test)]
mod tests_jet;
#[cfg(test)]
mod tests_puff;

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

pub use fly::{drift, raise};
pub use jet::JETS;

use super::Exhaust;
use super::tune::OFF;
use crate::effects::soft_disc;
use crate::screens::Screen;
use crate::space::{Anchor, Rebased};
use crate::universe::PlanetClass;
use crate::wind::Wind;
use crate::world::Terrain;
use puff::Mote;
use tune::COUNT;

/// The dust in the air: how many puffs live, and the materials they wear.
#[derive(Resource, Debug, Clone)]
pub struct Cloud {
    /// Puffs with a life left.
    live: usize,
    /// One material per soil, in the order of [`crate::avatar::Soil::ALL`].
    soils: [Handle<StandardMaterial>; crate::avatar::Soil::ALL.len()],
    /// The class of planet the materials are coloured for.
    class: Option<PlanetClass>,
}

/// The planet, its wind and the anchor, each when there is one.
type Place<'w> = (
    Option<Res<'w, Terrain>>,
    Option<Res<'w, Wind>>,
    Option<Res<'w, Anchor>>,
);

/// Run condition: the pool is made and there is a planet, its wind and an anchor to fly in.
pub fn ready(cloud: Option<Res<Cloud>>, (terrain, wind, anchor): Place) -> bool {
    cloud.is_some() && terrain.is_some() && wind.is_some() && anchor.is_some()
}

/// Run condition: some ship blows its lift nozzles or its main engines.
pub fn blowing(ships: Query<&Exhaust>) -> bool {
    ships
        .iter()
        .any(|exhaust| exhaust.lift > OFF || exhaust.main > OFF)
}

/// Run condition: some puffs are in the air.
pub fn drifting(cloud: Option<Res<Cloud>>) -> bool {
    cloud.is_some_and(|cloud| cloud.live > 0)
}

/// Builds the pool when gameplay starts, unless it is there: parked puffs at no size.
pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    old: Query<(), With<Mote>>,
) {
    if !old.is_empty() {
        return;
    }
    let disc = meshes.add(soft_disc());
    let class = PlanetClass::Island;
    let soils = look::soils(class, &mut materials);
    for _ in 0..COUNT {
        let puff = (Mesh3d(disc.clone()), MeshMaterial3d(soils[0].clone()));
        let marks = (Rebased, NotShadowCaster, DespawnOnEnter(Screen::Title));
        commands.spawn((
            Mote::default(),
            puff,
            Transform::from_scale(Vec3::ZERO),
            marks,
        ));
    }
    commands.insert_resource(Cloud {
        live: 0,
        soils,
        class: Some(class),
    });
}
