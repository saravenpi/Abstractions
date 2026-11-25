//! 3D geometric models for tools, minerals, wood, powders, ammunition, chests, torches, and tissues.

use bevy::prelude::*;

use super::models::ItemPreview;
use crate::craft::armor::ArmorSlot;
use crate::craft::build::{Figure, Look, Tint};
use crate::craft::gun::mesh::{Tri, gun_tris, push_box, push_prism};
use crate::craft::matter::{ChargeKind, Class, Mineral, PlaceableKind, WoodKind};
use crate::craft::pack::tune::STONE_TONES;

/// Maps [`Tint`] to linear RGB reflectance for tool geometry.
fn tint_rgb(tint: Tint) -> [f32; 3] {
    match tint {
        Tint::Stone => [0.76, 0.77, 0.80],
        Tint::Bone => [0.85, 0.81, 0.70],
        Tint::Horn => [0.36, 0.30, 0.25],
        Tint::Wood => [0.40, 0.23, 0.11],
    }
}

/// Builds faceted crystal spires or ore nodules tinted with [`STONE_TONES`].
fn mineral_tris(mineral: Mineral) -> Vec<Tri> {
    let base = STONE_TONES[mineral as usize];
    let hi = [
        (base[0] * 1.28 + 0.08).min(1.0),
        (base[1] * 1.28 + 0.08).min(1.0),
        (base[2] * 1.28 + 0.08).min(1.0),
    ];
    let crystal = matches!(
        mineral,
        Mineral::Quartz
            | Mineral::Amethyst
            | Mineral::Cryolite
            | Mineral::Ice
            | Mineral::Pyrite
            | Mineral::Obsidian
            | Mineral::Jade
            | Mineral::Corundum
            | Mineral::Moissanite
            | Mineral::Lonsdaleite
    );
    let mut out = Vec::with_capacity(64);
    if crystal {
        push_prism(
            &mut out,
            Vec3::new(0.0, -0.22, 0.0),
            Vec3::new(0.03, 0.32, 0.0),
            (0.14, 0.015),
            6,
            hi,
        );
        push_prism(
            &mut out,
            Vec3::new(-0.14, -0.20, 0.06),
            Vec3::new(-0.24, 0.16, 0.12),
            (0.10, 0.015),
            6,
            base,
        );
        push_prism(
            &mut out,
            Vec3::new(0.14, -0.20, -0.07),
            Vec3::new(0.24, 0.12, -0.14),
            (0.09, 0.015),
            6,
            base,
        );
        push_box(
            &mut out,
            Vec3::splat(-0.16),
            Vec3::new(0.16, -0.06, 0.16),
            [0.16, 0.15, 0.14],
        );
    } else {
        push_box(
            &mut out,
            Vec3::new(-0.20, -0.18, -0.18),
            Vec3::new(0.20, 0.16, 0.18),
            base,
        );
        push_prism(
            &mut out,
            Vec3::new(-0.14, -0.04, 0.12),
            Vec3::new(0.18, 0.24, -0.08),
            (0.14, 0.07),
            6,
            hi,
        );
        push_box(
            &mut out,
            Vec3::new(0.06, -0.12, -0.22),
            Vec3::new(0.24, 0.08, 0.06),
            hi,
        );
    }
    out
}

/// Builds a 3D crafted tool from its silhouette [`Look`].
fn tool_tris(look: Look) -> Vec<Tri> {
    let mut out = Vec::with_capacity(64);
    let shaft = tint_rgb(look.shaft);
    let head = tint_rgb(look.head);
    let len = 0.28 + 0.08 * f32::from(look.length);
    let a = Vec3::new(-len * 0.45, -len * 0.35, 0.0);
    let b = Vec3::new(len * 0.40, len * 0.38, 0.0);
    push_prism(&mut out, a, b, (0.032, 0.028), 8, shaft);
    let fig = look.figure.unwrap_or(Figure::Knife);
    match fig {
        Figure::Knife | Figure::Blade => {
            let tip = b + Vec3::new(0.16, 0.14, 0.0);
            push_prism(&mut out, b, tip, (0.055, 0.008), 4, head);
        }
        Figure::Axe => {
            let edge = b + Vec3::new(0.15, -0.12, 0.0);
            push_prism(
                &mut out,
                b - Vec3::new(0.03, 0.03, 0.0),
                edge,
                (0.05, 0.13),
                4,
                head,
            );
        }
        Figure::Spear | Figure::Javelin => {
            let tip = b + Vec3::new(0.22, 0.19, 0.0);
            push_prism(&mut out, b, tip, (0.055, 0.004), 6, head);
        }
        Figure::Hammer | Figure::Club => {
            push_box(
                &mut out,
                b - Vec3::new(0.09, 0.07, 0.07),
                b + Vec3::new(0.09, 0.07, 0.07),
                head,
            );
        }
    }
    out
}

/// Builds 3D geometry for logs, planks, sticks, and charcoal.
fn wood_tris(kind: WoodKind) -> Vec<Tri> {
    let mut out = Vec::with_capacity(36);
    match kind {
        WoodKind::Log => {
            push_prism(
                &mut out,
                Vec3::new(-0.26, -0.05, 0.0),
                Vec3::new(0.26, 0.05, 0.0),
                (0.14, 0.14),
                8,
                [0.30, 0.16, 0.07],
            );
            push_prism(
                &mut out,
                Vec3::new(-0.265, -0.051, 0.0),
                Vec3::new(0.265, 0.051, 0.0),
                (0.10, 0.10),
                8,
                [0.68, 0.49, 0.28],
            );
        }
        WoodKind::Plank => {
            push_box(
                &mut out,
                Vec3::new(-0.28, -0.06, -0.14),
                Vec3::new(0.28, 0.06, 0.14),
                [0.58, 0.38, 0.18],
            );
            push_box(
                &mut out,
                Vec3::new(-0.26, 0.055, -0.11),
                Vec3::new(0.26, 0.075, 0.11),
                [0.68, 0.47, 0.24],
            );
        }
        WoodKind::Stick => {
            push_prism(
                &mut out,
                Vec3::new(-0.28, -0.18, 0.0),
                Vec3::new(0.28, 0.22, 0.0),
                (0.036, 0.022),
                8,
                [0.46, 0.29, 0.14],
            );
        }
        WoodKind::Charcoal => {
            push_box(
                &mut out,
                Vec3::new(-0.18, -0.14, -0.14),
                Vec3::new(0.18, 0.12, 0.16),
                [0.06, 0.06, 0.07],
            );
            push_prism(
                &mut out,
                Vec3::new(-0.08, -0.06, 0.10),
                Vec3::new(0.16, 0.20, -0.06),
                (0.11, 0.04),
                6,
                [0.10, 0.10, 0.12],
            );
        }
    }
    out
}

/// Builds 3D geometry for powders, bullets, chests, and torches.
fn supply_tris(item: ItemPreview) -> Vec<Tri> {
    let mut out = Vec::with_capacity(64);
    match item {
        ItemPreview::Charge(c) => {
            let dust = match c {
                ChargeKind::SparkPowder => [0.52, 0.56, 0.62],
                ChargeKind::BrimstonePowder => [0.92, 0.80, 0.14],
            };
            push_prism(
                &mut out,
                Vec3::new(0.0, -0.20, 0.0),
                Vec3::new(0.0, 0.08, 0.0),
                (0.19, 0.14),
                8,
                [0.32, 0.20, 0.11],
            );
            push_prism(
                &mut out,
                Vec3::new(0.0, 0.06, 0.0),
                Vec3::new(0.0, 0.22, 0.0),
                (0.12, 0.02),
                8,
                dust,
            );
        }
        ItemPreview::Bullet(m) => {
            let tip = STONE_TONES[m as usize];
            push_prism(
                &mut out,
                Vec3::new(0.0, -0.24, 0.0),
                Vec3::new(0.0, 0.04, 0.0),
                (0.09, 0.09),
                8,
                [0.74, 0.56, 0.16],
            );
            push_prism(
                &mut out,
                Vec3::new(0.0, 0.04, 0.0),
                Vec3::new(0.0, 0.24, 0.0),
                (0.085, 0.015),
                8,
                tip,
            );
        }
        ItemPreview::Placeable(PlaceableKind::Chest) => {
            push_box(
                &mut out,
                Vec3::new(-0.24, -0.20, -0.18),
                Vec3::new(0.24, 0.16, 0.18),
                [0.46, 0.27, 0.12],
            );
            push_box(
                &mut out,
                Vec3::new(-0.25, 0.02, -0.19),
                Vec3::new(0.25, 0.06, 0.19),
                [0.20, 0.21, 0.24],
            );
            push_box(
                &mut out,
                Vec3::new(-0.04, -0.02, 0.17),
                Vec3::new(0.04, 0.09, 0.22),
                [0.76, 0.58, 0.18],
            );
        }
        ItemPreview::Placeable(PlaceableKind::Torch) => {
            push_prism(
                &mut out,
                Vec3::new(0.0, -0.28, 0.0),
                Vec3::new(0.0, 0.10, 0.0),
                (0.045, 0.052),
                8,
                [0.38, 0.22, 0.10],
            );
            push_prism(
                &mut out,
                Vec3::new(0.0, 0.08, 0.0),
                Vec3::new(0.0, 0.29, 0.0),
                (0.085, 0.01),
                6,
                [0.98, 0.52, 0.08],
            );
        }
        _ => {}
    }
    out
}

/// Builds 3D geometry for organic tissue resources.
fn tissue_tris(class: Class) -> Vec<Tri> {
    let mut out = Vec::with_capacity(40);
    match class {
        Class::Hide => push_box(
            &mut out,
            Vec3::new(-0.24, -0.05, -0.20),
            Vec3::new(0.24, 0.05, 0.20),
            [0.56, 0.34, 0.20],
        ),
        Class::Bone => {
            push_prism(
                &mut out,
                Vec3::new(-0.24, -0.10, 0.0),
                Vec3::new(0.24, 0.10, 0.0),
                (0.05, 0.05),
                8,
                [0.88, 0.85, 0.76],
            );
            push_box(
                &mut out,
                Vec3::new(-0.28, -0.15, -0.06),
                Vec3::new(-0.18, -0.05, 0.06),
                [0.90, 0.87, 0.78],
            );
            push_box(
                &mut out,
                Vec3::new(0.18, 0.05, -0.06),
                Vec3::new(0.28, 0.15, 0.06),
                [0.90, 0.87, 0.78],
            );
        }
        Class::Antler => {
            push_prism(
                &mut out,
                Vec3::new(-0.14, -0.22, 0.0),
                Vec3::new(0.10, 0.22, 0.0),
                (0.045, 0.025),
                6,
                [0.66, 0.56, 0.42],
            );
            push_prism(
                &mut out,
                Vec3::new(-0.04, -0.04, 0.0),
                Vec3::new(-0.22, 0.18, 0.06),
                (0.032, 0.012),
                6,
                [0.72, 0.62, 0.48],
            );
        }
        Class::Cuticle => push_prism(
            &mut out,
            Vec3::new(-0.22, -0.12, 0.0),
            Vec3::new(0.22, 0.14, 0.0),
            (0.18, 0.03),
            8,
            [0.78, 0.74, 0.70],
        ),
        Class::Keratin => push_prism(
            &mut out,
            Vec3::new(-0.20, -0.16, 0.0),
            Vec3::new(0.22, 0.20, 0.0),
            (0.11, 0.015),
            8,
            [0.34, 0.28, 0.22],
        ),
        Class::Wood => push_box(
            &mut out,
            Vec3::new(-0.24, -0.05, -0.12),
            Vec3::new(0.24, 0.05, 0.12),
            [0.48, 0.32, 0.16],
        ),
        Class::Glass => push_prism(
            &mut out,
            Vec3::new(0.0, -0.22, 0.0),
            Vec3::new(0.04, 0.28, 0.0),
            (0.12, 0.015),
            6,
            [0.76, 0.88, 0.96],
        ),
        Class::Resin => push_prism(
            &mut out,
            Vec3::new(0.0, -0.18, 0.0),
            Vec3::new(0.0, 0.18, 0.0),
            (0.15, 0.06),
            8,
            [0.84, 0.56, 0.14],
        ),
    }
    out
}

/// Builds 3D geometry for mineral plate and shielded Aegis armor pieces.
fn armor_tris(slot: ArmorSlot, mineral: Mineral, shielded: bool) -> Vec<Tri> {
    let base = STONE_TONES[mineral as usize];
    let hi = [
        (base[0] * 1.24 + 0.06).min(1.0),
        (base[1] * 1.24 + 0.06).min(1.0),
        (base[2] * 1.24 + 0.06).min(1.0),
    ];
    let trim = if shielded { [0.22, 0.85, 1.0] } else { hi };
    let mut out = Vec::with_capacity(64);
    match slot {
        ArmorSlot::Head => {
            push_box(
                &mut out,
                Vec3::new(-0.20, -0.14, -0.18),
                Vec3::new(0.20, 0.22, 0.18),
                base,
            );
            push_box(
                &mut out,
                Vec3::new(-0.14, 0.02, 0.15),
                Vec3::new(0.14, 0.12, 0.21),
                trim,
            );
            push_box(
                &mut out,
                Vec3::new(-0.04, 0.18, -0.16),
                Vec3::new(0.04, 0.27, 0.16),
                trim,
            );
        }
        ArmorSlot::Chest => {
            push_box(
                &mut out,
                Vec3::new(-0.22, -0.24, -0.13),
                Vec3::new(0.22, 0.22, 0.13),
                base,
            );
            push_box(
                &mut out,
                Vec3::new(-0.32, 0.06, -0.14),
                Vec3::new(-0.18, 0.24, 0.14),
                hi,
            );
            push_box(
                &mut out,
                Vec3::new(0.18, 0.06, -0.14),
                Vec3::new(0.32, 0.24, 0.14),
                hi,
            );
            push_box(
                &mut out,
                Vec3::new(-0.08, -0.04, 0.11),
                Vec3::new(0.08, 0.12, 0.16),
                trim,
            );
        }
        ArmorSlot::Legs => {
            push_box(
                &mut out,
                Vec3::new(-0.22, 0.12, -0.13),
                Vec3::new(0.22, 0.24, 0.13),
                hi,
            );
            push_box(
                &mut out,
                Vec3::new(-0.20, -0.26, -0.11),
                Vec3::new(-0.03, 0.14, 0.11),
                base,
            );
            push_box(
                &mut out,
                Vec3::new(0.03, -0.26, -0.11),
                Vec3::new(0.20, 0.14, 0.11),
                base,
            );
            push_box(
                &mut out,
                Vec3::new(-0.17, -0.10, 0.09),
                Vec3::new(-0.06, 0.00, 0.14),
                trim,
            );
            push_box(
                &mut out,
                Vec3::new(0.06, -0.10, 0.09),
                Vec3::new(0.17, 0.00, 0.14),
                trim,
            );
        }
        ArmorSlot::Boots => {
            push_box(
                &mut out,
                Vec3::new(-0.22, -0.22, -0.12),
                Vec3::new(-0.04, 0.10, 0.10),
                base,
            );
            push_box(
                &mut out,
                Vec3::new(0.04, -0.22, -0.12),
                Vec3::new(0.22, 0.10, 0.10),
                base,
            );
            push_box(
                &mut out,
                Vec3::new(-0.22, -0.24, -0.12),
                Vec3::new(-0.04, -0.10, 0.20),
                hi,
            );
            push_box(
                &mut out,
                Vec3::new(0.04, -0.24, -0.12),
                Vec3::new(0.22, -0.10, 0.20),
                hi,
            );
            push_box(
                &mut out,
                Vec3::new(-0.23, 0.04, -0.13),
                Vec3::new(-0.03, 0.11, 0.11),
                trim,
            );
            push_box(
                &mut out,
                Vec3::new(0.03, 0.04, -0.13),
                Vec3::new(0.23, 0.11, 0.11),
                trim,
            );
        }
    }
    out
}

/// Dispatches 3D preview triangles for `item`.
pub(super) fn build_tris(item: ItemPreview) -> Vec<Tri> {
    match item {
        ItemPreview::Tool(look) => tool_tris(look),
        ItemPreview::Gun(kind, barrel) => gun_tris(kind, barrel),
        ItemPreview::Mineral(m) => mineral_tris(m),
        ItemPreview::Wood(w) => wood_tris(w),
        ItemPreview::Tissue(c) => tissue_tris(c),
        ItemPreview::Armor {
            slot,
            mineral,
            shielded,
        } => armor_tris(slot, mineral, shielded),
        other => supply_tris(other),
    }
}
