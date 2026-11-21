//! How the dust looks: one see-through material per soil, coloured for the planet, worn by soft
//! round discs.

use bevy::prelude::*;

use super::Cloud;
use crate::avatar::Soil;
use crate::universe::PlanetClass;

/// Returns the lit, see-through, two-sided material of a puff of `tint`, an sRGB colour with
/// its alpha. The disc it is drawn on fades it to clear at the rim.
pub fn material([r, g, b, a]: [f32; 4]) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgba(r, g, b, a),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        double_sided: true,
        cull_mode: None,
        ..default()
    }
}

/// Returns one material per soil, in the order of [`Soil::ALL`], coloured for a planet of `class`.
pub fn soils(
    class: PlanetClass,
    materials: &mut Assets<StandardMaterial>,
) -> [Handle<StandardMaterial>; Soil::ALL.len()] {
    Soil::ALL.map(|soil| materials.add(material(soil.tint(class))))
}

/// Colours the materials of the dust for the planet of `class`, once per planet.
pub fn recolour(cloud: &mut Cloud, class: PlanetClass, materials: &mut Assets<StandardMaterial>) {
    if cloud.class == Some(class) {
        return;
    }
    cloud.class = Some(class);
    for (handle, soil) in cloud.soils.iter().zip(Soil::ALL) {
        if let Some(mut paint) = materials.get_mut(handle) {
            paint.base_color = material(soil.tint(class)).base_color;
        }
    }
}
