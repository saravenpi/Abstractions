//! Pipeline warm-up of the body: its meshes and materials are drawn once behind the boot screen,
//! since the body itself only appears once play begins.

use bevy::prelude::*;

use super::mesh::{fabric, glow, screen, segment};
use super::rig::Seg;
use super::spawn::{Stock, kit, paint};
use crate::render::Warm;

/// Returns the material of the fabric: lit `accent` seen from both sides.
pub(super) fn sheet(accent: Color) -> StandardMaterial {
    StandardMaterial {
        double_sided: true,
        cull_mode: None,
        ..paint(accent, true, None)
    }
}

/// Hands one of each mesh layout and material of the body to the pipeline warm-up. The body
/// only appears once play begins, after the warm-up ran.
pub fn warm(mut warm: ResMut<Warm>, mut stock: Stock) {
    let kit = kit(&mut stock);
    let shell = stock.materials.add(paint(Color::WHITE, true, None));
    let cloth = stock.materials.add(sheet(Color::WHITE));
    warm.add(stock.meshes.add(segment(Seg::Torso).into_mesh()), shell);
    warm.add(stock.meshes.add(screen()), kit.visor);
    warm.add(stock.meshes.add(glow(1.0)), kit.glow);
    warm.add(stock.meshes.add(fabric()), cloth);
}
