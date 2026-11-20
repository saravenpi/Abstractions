//! The water a source churns: spray and bubbles in a pool of flecks, wakes and ripple rings in one
//! pooled mesh of foam, each thrown from the wash, the speed and the jet of the source.

mod churn;
mod drops;
mod foam;
mod material;
#[cfg(test)]
mod tests_cost;
#[cfg(test)]
mod tests_spray;
mod throw;
mod tune;
mod wash;

use bevy::asset::embedded_asset;
use bevy::pbr::MaterialPlugin;
use bevy::prelude::*;
use bevy::transform::TransformSystems;

pub use churn::Churn;

use super::eye::grounded;
use crate::render::warm_material;
use crate::screens::{Screen, playing};
use crate::world::{Liquid, Terrain};

/// Run condition: some drop may still fly.
fn awake(drops: Res<drops::Drops>) -> bool {
    drops.awake
}

/// Run condition: the planet under the anchor has a sea.
fn has_sea(terrain: Option<Res<Terrain>>) -> bool {
    terrain.is_some_and(|terrain| Liquid::of(terrain.globe()).shown)
}

/// Registers the pooled drops and foam and the systems that churn the sea.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "foam.wesl");
    app.init_resource::<drops::Drops>()
        .init_resource::<foam::Foam>()
        .add_plugins(MaterialPlugin::<material::FoamMaterial>::default())
        .add_systems(OnEnter(Screen::Gameplay), (drops::spawn, foam::spawn))
        .add_systems(
            PostUpdate,
            (churn::churn, drops::step.run_if(awake), material::tick)
                .chain()
                .run_if(playing)
                .run_if(grounded)
                .run_if(has_sea)
                .before(TransformSystems::Propagate),
        );
    warm_material::<material::FoamMaterial>(app);
}
