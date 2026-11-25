//! Tests verifying 3D isometric previews render distinctly for every mineral, tool, gun, and supply.

use super::models::ItemPreview;
use super::render3d::{handle_3d, render_preview};
use crate::craft::armor::ArmorSlot;
use crate::craft::build::{Figure, Intent, Look, Tint};
use crate::craft::gun::GunKind;
use crate::craft::matter::{ChargeKind, Class, Mineral, PlaceableKind, WoodKind};

/// Counts opaque foreground pixels (`alpha == 255`) inside a rendered thumbnail buffer.
fn opaque_count(buf: &[u8]) -> usize {
    buf.as_chunks::<4>()
        .0
        .iter()
        .filter(|px| px[3] == 255)
        .count()
}

/// Every item class produces a non-empty shaded 3D isometric image and unique cache handle.
#[test]
fn every_item_kind_renders_distinct_3d_thumbnail() {
    let mut items = Vec::new();
    for m in Mineral::ALL {
        items.push(ItemPreview::Mineral(m));
    }
    for f in Figure::ALL {
        items.push(ItemPreview::Tool(Look {
            intent: Intent::Cut,
            figure: Some(f),
            length: 2,
            head: Tint::Stone,
            shaft: Tint::Wood,
        }));
    }
    items.push(ItemPreview::Gun(
        GunKind::FlintlockPistol,
        Mineral::Ironstone,
    ));
    items.push(ItemPreview::Gun(GunKind::MineralRifle, Mineral::Cobaltite));
    for w in [
        WoodKind::Log,
        WoodKind::Plank,
        WoodKind::Stick,
        WoodKind::Charcoal,
    ] {
        items.push(ItemPreview::Wood(w));
    }
    for c in [ChargeKind::SparkPowder, ChargeKind::BrimstonePowder] {
        items.push(ItemPreview::Charge(c));
    }
    items.push(ItemPreview::Bullet(Mineral::Copper));
    items.push(ItemPreview::Placeable(PlaceableKind::Chest));
    items.push(ItemPreview::Placeable(PlaceableKind::Torch));
    for cl in Class::ALL {
        items.push(ItemPreview::Tissue(cl));
    }
    for slot in ArmorSlot::ALL {
        items.push(ItemPreview::Armor {
            slot,
            mineral: Mineral::Ironstone,
            shielded: false,
        });
        items.push(ItemPreview::Armor {
            slot,
            mineral: Mineral::Lonsdaleite,
            shielded: true,
        });
    }

    let rendered: Vec<Vec<u8>> = items.iter().map(|i| render_preview(i, 32, 0)).collect();
    for (item, buf) in items.iter().zip(&rendered) {
        assert!(opaque_count(buf) >= 24, "{item:?} rendered too few pixels");
    }
    for (idx, a) in items.iter().enumerate() {
        for b in items.iter().skip(idx + 1) {
            assert_ne!(handle_3d(a, 32, 0), handle_3d(b, 32, 0));
        }
    }
}
