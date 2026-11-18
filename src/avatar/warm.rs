//! Pipeline warm-up of the body: its meshes and materials are drawn once behind the boot screen,
//! since the body itself only appears once play begins.

use bevy::prelude::*;

use super::looks::{paint, sheet};
use super::mesh::{fabric, glow, screen, segment};
use super::puffs;
use super::rig::Seg;
use super::spawn::{Stock, kit};
use crate::render::Warm;

/// Hands one of each mesh layout and material of the body and of the puffs it kicks up to the
/// pipeline warm-up. The body only appears once play begins, after the warm-up ran.
pub fn warm(mut warm: ResMut<Warm>, mut stock: Stock) {
    let kit = kit(&mut stock);
    let shell = stock.materials.add(paint(Color::WHITE, true, None));
    let cloth = stock.materials.add(sheet(Color::WHITE));
    warm.add(stock.meshes.add(segment(Seg::Torso).into_mesh()), shell);
    warm.add(stock.meshes.add(screen()), kit.visor);
    warm.add(stock.meshes.add(glow(1.0)), kit.glow);
    warm.add(stock.meshes.add(fabric()), cloth);
    let dust = stock.materials.add(puffs::material([1.0; 4]));
    warm.add(stock.meshes.add(Rectangle::new(1.0, 1.0)), dust);
}
