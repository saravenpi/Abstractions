//! Tissue numbers: a species' tissue as a material, its class anchor scaled by how it differs from home.

use super::{Class, Grade, Material};
use crate::life::Tissue;
use crate::life::tissue::HOME;

/// Returns the material of `tissue` harvested as `class` from a kill of `grade`, or nothing when the
/// tissue is not of the kind the class comes from.
///
/// Density, toughness and conductivity scale with the tissue's own values over the home values of
/// its kind; stiffness scales inversely with elasticity. Strength follows hardness, since hardness is
/// about three times the yield strength (Tabor, https://en.wikipedia.org/wiki/Indentation_hardness).
/// The grade multiplies strength and toughness.
pub fn material(class: Class, tissue: &Tissue, grade: Grade) -> Option<Material> {
    if class.kind() != tissue.kind {
        return None;
    }
    let [h, t, d, e, _, c] = HOME[tissue.kind as usize];
    let anchor = class.anchor();
    let hard = tissue.hardness / h;
    let quality = grade.quality();
    Some(Material {
        density: anchor.density * tissue.density / d,
        stiffness: anchor.stiffness * e / tissue.elasticity.max(0.01),
        strength: anchor.strength * hard * quality,
        toughness: anchor.toughness * tissue.toughness / t * quality,
        hardness: (anchor.hardness * hard).min(10.0),
        conductivity: anchor.conductivity * tissue.conductivity / c,
    })
}
