//! The exhaust of a ship, as the real exhaust of its engines: the plumes of the main engines,
//! the lift nozzles and the attitude jets, the heat of the nozzles, the lights they cast, and
//! the dust the lift nozzles blow off the ground. Everything is driven by the real state of the
//! ship, its thrust and the air around it, and every ship has it, the one this machine flies
//! and the puppets of other players, from the thrust the records carry.
//!
//! The plumes, lights and dust are pooled: nothing is spawned or allocated per frame.

mod drive;
mod dust;
mod jets;
mod lights;
mod plume;
mod shape;
mod shells;
mod state;
mod tune;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_jets;
#[cfg(test)]
mod tests_lamps;
#[cfg(test)]
mod tests_lift;
#[cfg(test)]
mod tests_width;

use bevy::prelude::*;

pub use jets::fit as fit_plumes;
pub use lights::{Lamp as HullLamp, TIP_COLORS, fit as fit_lights, place as lamp_place};
pub use shells::jet_mesh;
pub use state::Exhaust;
#[cfg(test)]
pub(super) use state::{demand, lift};

use super::ShipSet;
use super::glow;
use crate::render::{GameCamera, Warm};
use crate::screens::Screen;

/// Hands the plume and its material to the pipeline warm-up, which runs before any ship has
/// its plumes, so that the first plume does not compile its shader in play.
fn warm(
    mut warm: ResMut<Warm>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<plume::ExhaustMaterial>>,
) {
    warm.add(
        meshes.add(jet_mesh()),
        materials.add(plume::ExhaustMaterial::default()),
    );
}

/// Registers the material of the plumes, the pool of dust and the systems that drive them.
pub(super) fn plugin(app: &mut App) {
    plume::plugin(app);
    let rendered = resource_exists::<Warm>
        .and_then(resource_exists::<Assets<Mesh>>)
        .and_then(resource_exists::<Assets<plume::ExhaustMaterial>>);
    app.add_systems(Startup, warm.run_if(rendered));
    let drawn = (
        drive::paint,
        drive::stretch,
        lights::glare,
        dust::raise.run_if(dust::blowing),
        dust::drift
            .run_if(dust::drifting)
            .run_if(any_with_component::<GameCamera>),
    );
    let run = (plume::equip.run_if(plume::bare), state::feed, drawn);
    app.add_systems(OnEnter(Screen::Gameplay), dust::spawn)
        .add_systems(
            Update,
            run.chain()
                .after(glow::throttle)
                .run_if(any_with_component::<Exhaust>)
                .in_set(ShipSet::Report),
        );
}
