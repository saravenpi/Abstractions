//! The ground of the home base: kept free of props on its planet.

use std::sync::Arc;

use bevy::prelude::*;

use super::{Planet, Terrain};
use crate::home::Base;
use crate::universe::Universe;

/// Swaps in a globe that reserves the footprint of the base, once per visit of its planet. The
/// planet changes, so every chunk built before is dropped and built again around the base.
pub fn keep_base(
    mut commands: Commands,
    base: Option<Res<Base>>,
    planet: Res<Planet>,
    terrain: Res<Terrain>,
) {
    let on_planet = |base: &Res<Base>| base.body == planet.body && planet.globe.reserve().is_none();
    let Some(base) = base.filter(on_planet) else {
        return;
    };
    let reserve = base.reserve();
    let globe = Arc::new(planet.globe.as_ref().clone().reserving(reserve));
    commands.insert_resource(Terrain::on(globe.clone(), terrain.chart()));
    commands.insert_resource(Planet {
        globe,
        ..planet.clone()
    });
}

/// Run condition: detail chunks may be built, which on the home planet waits for the base to
/// be found and its ground reserved.
pub fn settled(universe: Option<Res<Universe>>, planet: Option<Res<Planet>>) -> bool {
    let (Some(universe), Some(planet)) = (universe, planet) else {
        return true;
    };
    planet.body != universe.home() || planet.globe.reserve().is_some()
}
