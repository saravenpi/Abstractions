//! Reads the plants within reach of the particle box from the canopy and paints their materials.

use bevy::prelude::*;

use super::{MARGIN, SLOTS, Source};
use crate::effects::eye::Eye;
use crate::effects::pool::{ABOVE, BELOW, HALF_WIDTH, Pool};
use crate::world::CHUNK_SIZE;
use crate::world::canopy::Canopy;

/// Returns the plant of `crown` at render position `foot` as the eye sees it, when its crown reaches
/// into the particle box.
fn seen(eye: &Eye, foot: Vec3, crown: &crate::world::canopy::Crown) -> Option<Source> {
    let shed = crown.shed?;
    let at = eye.tilt.inverse() * (foot - eye.at);
    let wide = at.x.abs().max(at.z.abs()) < HALF_WIDTH + crown.reach;
    let tall = at.y + crown.height > -BELOW && at.y < ABOVE;
    (wide && tall).then_some(Source {
        at,
        height: crown.height,
        reach: crown.reach,
        shed,
        leaf: 0.0,
        petal: 0.0,
        pollen: 0.0,
    })
}

/// Sets the base colour of `handle` to the linear RGB `rgb`, only when it changes.
fn tint(
    materials: &mut Assets<StandardMaterial>,
    handle: &Handle<StandardMaterial>,
    rgb: [f32; 3],
) {
    let colour = Color::linear_rgb(rgb[0], rgb[1], rgb[2]);
    if materials
        .get(handle)
        .is_some_and(|m| m.base_color != colour)
        && let Some(mut material) = materials.get_mut(handle)
    {
        material.base_color = colour;
    }
}

/// Reads the plants within reach of the particle box from the canopy of the chunks near the eye, keeps the
/// nearest [`SLOTS`], and paints the leaf and petal material of each slot in the colours of its plant.
pub fn gather(
    eye: Res<Eye>,
    chunks: Query<(&Canopy, &GlobalTransform)>,
    pool: Res<Pool>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sensed: ResMut<crate::effects::sense::Sensed>,
) {
    let near = CHUNK_SIZE * std::f32::consts::FRAC_1_SQRT_2 + HALF_WIDTH + MARGIN;
    let shed = &mut sensed.shed.sources;
    shed.clear();
    for (canopy, place) in &chunks {
        if place.translation().distance(eye.at) > near {
            continue;
        }
        let crowns = canopy.0.iter();
        shed.extend(crowns.filter_map(|c| seen(&eye, place.transform_point(c.at), c)));
    }
    shed.sort_by(|a, b| {
        a.at.xz()
            .length_squared()
            .total_cmp(&b.at.xz().length_squared())
    });
    shed.truncate(SLOTS);
    for (i, source) in shed.iter().enumerate() {
        if let Some(leaf) = source.shed.leaf {
            tint(&mut materials, &pool.leaves[i], leaf);
        }
        if let Some(flower) = source.shed.flower {
            tint(&mut materials, &pool.petals[i], flower);
        }
    }
}
