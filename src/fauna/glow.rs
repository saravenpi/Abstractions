//! Bioluminescence: the glowing parts of species whose life has a glow light up as night falls.
//!
//! A glowing species gets its own material, emissive in its glow colour. Its accent parts glow,
//! or every part when it has no accent. A few times a second the emissive strength follows the
//! darkness of the sky, so the glow is plain only at night.

use bevy::prelude::*;

use super::looks::Looks;
use super::tune::NIGHT_GLOW;
use crate::life::Species;
use crate::render::{DayClock, daylight};

/// Returns the material of the glowing parts of `species`, unlit until [`shine`] runs, or nothing
/// for a species with no glow.
pub fn lamp(
    species: &Species,
    materials: &mut Assets<StandardMaterial>,
) -> Option<Handle<StandardMaterial>> {
    species.glow?;
    Some(materials.add(StandardMaterial {
        perceptual_roughness: 0.8,
        emissive: LinearRgba::BLACK,
        ..default()
    }))
}

/// Returns, for each part of `species`, true when it glows: its accent parts, or all of them when it
/// has no accent part.
pub fn lit(species: &Species) -> Vec<bool> {
    let parts = &species.body.parts;
    let accents = parts.iter().any(|p| p.tissue > 0);
    parts.iter().map(|p| !accents || p.tissue > 0).collect()
}

/// Returns the emissive colour of `species` under `light`, 0 at night to 1 at noon.
pub fn emissive(species: &Species, light: f32) -> LinearRgba {
    let Some(glow) = species.glow else {
        return LinearRgba::BLACK;
    };
    let k = NIGHT_GLOW * glow.strength * (1.0 - light).powi(2);
    LinearRgba::rgb(glow.colour[0] * k, glow.colour[1] * k, glow.colour[2] * k)
}

/// Sets the emissive colour of every glowing species' material from the time of day.
pub fn shine(
    clock: Res<DayClock>,
    terrain: Res<crate::world::Terrain>,
    looks: Res<Looks>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let globe = terrain.globe().clone();
    let Some(life) = globe.life() else {
        return;
    };
    let light = daylight(clock.phase);
    for (index, handle) in looks.lamps() {
        let (Some(species), Some(mut material)) =
            (life.species.get(index), materials.get_mut(handle))
        else {
            continue;
        };
        let colour = emissive(species, light);
        if material.emissive != colour {
            material.emissive = colour;
        }
    }
}
