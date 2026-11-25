//! Slot detail inspector for the 24+6 grid view: large 3D thumbnail, properties, and source hints.

use bevy::prelude::*;

use super::detail::{Detail, panel as legacy_panel};
use super::models::ItemPreview;
use super::render3d::preview_badge;
use super::sheet::{blows, gauges, sheet};
use super::tune::{FULL_HARDNESS, FULL_STIFFNESS, FULL_STRENGTH, FULL_TOUGHNESS};
use crate::craft::build::Place;
use crate::craft::matter::{Material, Stuff};
use crate::craft::pack::{BAR_SLOTS, Pack, SlotEntry, called};
use crate::ui::units::fixed;
use crate::ui::{Tone, label, meter, stat, text, theme};

/// Returns location/usage guidance text for a stack item in the slot inspector.
fn stack_hint(stuff: &Stuff) -> String {
    match *stuff {
        Stuff::Mineral(m) => m.where_found().to_string(),
        Stuff::Wood(_) => "Used for planks, chests, sticks & weapon stocks".to_string(),
        Stuff::Charge(_) => "Propellant combined with minerals for bullets".to_string(),
        Stuff::Bullet(_) => "Press R with pistol or rifle held to load".to_string(),
        Stuff::Placeable(_) => "Hold in hotbar and press Interact/Click to place".to_string(),
        Stuff::Tissue { .. } => "Organic binding and structural tool material".to_string(),
    }
}

/// Renders four material property bars inside `parent`.
fn material_bars(parent: &mut ChildSpawnerCommands, m: &Material) {
    let bar = |label_str, val: f32, full: f32, unit, dec| {
        let share = (val / full).clamp(0.0, 1.0);
        stat(label_str, share, format!("{} {unit}", fixed(val, dec)))
    };
    parent.spawn(bar("Hardness", m.hardness, FULL_HARDNESS, "Mohs", 1));
    parent.spawn(bar("Toughness", m.toughness, FULL_TOUGHNESS, "MPa", 2));
    parent.spawn(bar("Strength", m.strength, FULL_STRENGTH, "MPa", 0));
    parent.spawn(bar("Stiffness", m.stiffness, FULL_STIFFNESS, "GPa", 1));
}

/// Spawns header (64px 3D thumbnail, item title, subtitle) in the inspector.
fn header(parent: &mut ChildSpawnerCommands, preview: ItemPreview, title: &str, subtitle: &str) {
    let row = Node {
        align_items: AlignItems::Center,
        column_gap: px(theme::S3),
        margin: UiRect::bottom(px(theme::S3)),
        ..default()
    };
    let words = Node {
        flex_direction: FlexDirection::Column,
        ..default()
    };
    parent.spawn(row).with_children(|r| {
        r.spawn(preview_badge(preview, 64.0, false));
        r.spawn(words).with_children(|w| {
            w.spawn(label(title, theme::FONT_BODY, Tone::Plain));
            w.spawn(label(subtitle, theme::FONT_SMALL, Tone::Muted));
        });
    });
}

/// Renders details for explicitly selected slot `slot_idx`, or delegates to `legacy_panel`.
pub fn inspect_slot(
    parent: &mut ChildSpawnerCommands,
    pack: &Pack,
    slot_idx: Option<usize>,
    place: &Place,
    fallback: Option<&Detail>,
) {
    let Some(idx) = slot_idx else {
        legacy_panel(parent, fallback);
        return;
    };
    let Some(entry) = pack.grid.get(idx).and_then(|s| s.as_ref()) else {
        parent.spawn(label(
            if idx < BAR_SLOTS {
                format!("Hotbar slot #{} (empty)", idx + 1)
            } else {
                format!("Pack slot #{} (empty)", idx - BAR_SLOTS + 1)
            },
            theme::FONT_BODY,
            Tone::Muted,
        ));
        return;
    };
    match *entry {
        SlotEntry::Tool(id) => {
            let Some(item) = pack.item(id) else { return };
            let Some(preview) = ItemPreview::from_slot(pack, entry) else {
                return;
            };
            let slot_tag = if idx < BAR_SLOTS {
                format!("Hotbar slot #{}", idx + 1)
            } else {
                "Pack storage".to_string()
            };
            header(parent, preview, &item.name(), &slot_tag);
            let (worn, _) = item.worn(place);
            parent.spawn((
                Node {
                    margin: UiRect::bottom(px(theme::S2)),
                    column_gap: px(theme::S2),
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    text("Condition", theme::FONT_SMALL, theme::TEXT_2),
                    meter(theme::span(2), (1.0 - worn).clamp(0.0, 1.0))
                ],
            ));
            if let Some(build) = item.build() {
                let s = sheet(&build, item.intent(), place);
                for g in gauges(&s, item.intent()) {
                    parent.spawn(stat(g.label, g.share, g.value));
                }
                parent.spawn(label(
                    format!("Joint life: {:.0} blows", blows(s.life)),
                    theme::FONT_SMALL,
                    Tone::Muted,
                ));
            }
        }
        SlotEntry::Gun(id) => {
            let Some(gun) = pack.gun(id) else { return };
            let preview = ItemPreview::Gun(gun.kind, gun.barrel);
            let st = gun.kind.stats();
            let sub = format!(
                "{}/{} rounds  {:.2} kg",
                gun.loaded,
                st.capacity,
                gun.mass()
            );
            header(parent, preview, &gun.name(), &sub);
            parent.spawn(stat(
                "Velocity",
                (st.speed / 200.0).clamp(0.0, 1.0),
                format!("{:.0} m/s", st.speed),
            ));
            parent.spawn(stat(
                "Damage",
                (st.damage / 100.0).clamp(0.0, 1.0),
                format!("{:.0}", st.damage),
            ));
            parent.spawn(stat(
                "Reload",
                (1.0 - st.reload / 3.0).clamp(0.1, 1.0),
                format!("{:.1}s", st.reload),
            ));
            parent.spawn(label(
                "Left-click shoot  Right-click aim  R reload",
                theme::FONT_SMALL,
                Tone::Muted,
            ));
        }
        SlotEntry::Stack {
            stuff,
            material,
            count,
        } => {
            let preview = ItemPreview::from_stuff(&stuff);
            let kg = f32::from(count) * stuff.unit_mass();
            let sub = format!("x{count} ({:.2} kg)  {}", kg, stack_hint(&stuff));
            header(parent, preview, called(&stuff), &sub);
            material_bars(parent, &material);
        }
    }
}
