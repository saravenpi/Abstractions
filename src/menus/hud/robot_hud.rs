//! 2D robot silhouette health and rechargeable energy shield HUD widget in the bottom-left stack.

use bevy::ecs::spawn::SpawnIter;
use bevy::prelude::*;

use super::context::Element;
use super::fade::gate;
use crate::condition::{Condition, PARTS, Part, Shield};
use crate::craft::Pack;
use crate::ui::{text, theme};

/// Electric cyan color for the energy shield aura and bar (`#38D9FF`).
const CYAN: Color = Color::srgb(0.220, 0.851, 1.0);

/// Outer shield aura border container of the 2D robot HUD widget.
#[derive(Component)]
pub(super) struct RobotHud;

/// Bottom-anchored health fill bar inside one of the ten silhouette body part housings.
#[derive(Component)]
pub(super) struct PartFill(pub Part);

/// Top shield bar and readout container, shown when `shield.max > 0.0`.
#[derive(Component)]
pub(super) struct ShieldRow;

/// Horizontal cyan fill bar inside [`ShieldRow`].
#[derive(Component)]
pub(super) struct ShieldFill;

/// Compact shield readout text (`SHD 920/920` or `RECHARGING`).
#[derive(Component)]
pub(super) struct ShieldLabel;

/// Bottom integrity and equipped defense summary line (`100% · DEF 82%`).
#[derive(Component)]
pub(super) struct StatsLabel;

/// Returns the health color for a body part (`>=75%` teal, `>=45%` amber, `>=20%` orange, `<20%` crimson).
pub fn part_color(health: f32) -> Color {
    if health >= 0.75 {
        Color::srgb(0.306, 0.878, 0.627)
    } else if health >= 0.45 {
        Color::srgb(0.941, 0.753, 0.251)
    } else if health >= 0.20 {
        Color::srgb(0.933, 0.424, 0.173)
    } else {
        Color::srgb(0.878, 0.157, 0.220)
    }
}

/// Returns `(left, top, width, height)` in pixels for `part` inside the `96 x 92` silhouette canvas.
fn part_rect(part: Part) -> (f32, f32, f32, f32) {
    match part {
        Part::Head => (38.0, 2.0, 20.0, 16.0),
        Part::WingL => (4.0, 18.0, 14.0, 28.0),
        Part::ArmL => (20.0, 22.0, 10.0, 30.0),
        Part::Torso => (32.0, 20.0, 32.0, 34.0),
        Part::ArmR => (66.0, 22.0, 10.0, 30.0),
        Part::WingR => (78.0, 18.0, 14.0, 28.0),
        Part::LegL => (34.0, 56.0, 12.0, 22.0),
        Part::LegR => (50.0, 56.0, 12.0, 22.0),
        Part::BootL => (31.0, 80.0, 15.0, 10.0),
        Part::BootR => (50.0, 80.0, 15.0, 10.0),
    }
}

/// Spawns one recessed body part housing with a bottom-anchored fill node.
fn part_cell(part: Part) -> impl Bundle {
    let (left, top, width, height) = part_rect(part);
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(left),
            top: px(top),
            width: px(width),
            height: px(height),
            align_items: AlignItems::FlexEnd,
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.07, 0.10, 0.88)),
        children![(
            PartFill(part),
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            BackgroundColor(part_color(1.0)),
        )],
    )
}

/// Spawns the top energy shield bar and compact status label, hidden until shield is equipped.
fn shield_row() -> impl Bundle {
    let fill = (
        ShieldFill,
        Node {
            width: percent(0),
            height: percent(100),
            ..default()
        },
        BackgroundColor(CYAN),
    );
    let track = (
        Node {
            width: percent(100),
            height: px(5.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.16, 0.22, 0.85)),
        children![fill],
    );
    (
        ShieldRow,
        Node {
            display: Display::None,
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(2.0),
            ..default()
        },
        children![
            track,
            (ShieldLabel, text("", 11.0, CYAN), TextLayout::no_wrap())
        ],
    )
}

/// Spawns the 2D robot silhouette health and shield HUD widget (`~108 x 148 px`).
pub(super) fn widget() -> impl Bundle {
    let canvas = (
        Node {
            position_type: PositionType::Relative,
            width: px(96.0),
            height: px(92.0),
            ..default()
        },
        Children::spawn(SpawnIter(Part::ALL.into_iter().map(part_cell))),
    );
    let stats = (
        StatsLabel,
        text("100% · DEF 0%", 11.0, theme::TEXT),
        TextLayout::no_wrap(),
    );
    let inner = (
        Node {
            width: percent(100),
            min_height: px(144.0),
            padding: UiRect::all(px(4.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(4.0),
            ..default()
        },
        children![shield_row(), canvas, stats],
    );
    (
        RobotHud,
        gate(Element::Integrity),
        Node {
            width: px(108.0),
            border: UiRect::all(px(2.0)),
            ..default()
        },
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.16)),
        children![inner],
    )
}

/// Run condition: the 2D robot HUD was just spawned and needs initial state painted.
pub(super) fn fresh(huds: Query<(), Added<RobotHud>>) -> bool {
    !huds.is_empty()
}

/// Computes `(aura_color, shield_label, stats_label)` from current condition, shield, and pack.
fn readouts(condition: &Condition, shd: Shield, pack: Option<&Pack>) -> (Color, String, String) {
    let has = shd.max > 0.0;
    let aura = if has {
        Color::srgba(0.220, 0.851, 1.0, 0.25 + 0.75 * shd.share())
    } else {
        Color::srgba(1.0, 1.0, 1.0, 0.12)
    };
    let shd_text = if has && shd.delay <= 0.0 && shd.now < shd.max - 0.5 {
        "RECHARGING".to_string()
    } else {
        format!("SHD {}/{}", shd.now.round() as u32, shd.max.round() as u32)
    };
    let avg_def = pack.map_or(0.0, |p| {
        Part::ALL
            .into_iter()
            .map(|pt| p.defense_for(pt))
            .sum::<f32>()
            / PARTS as f32
    });
    let stats_text = format!(
        "{}% · DEF {}%",
        (condition.integrity() * 100.0).round() as u32,
        (avg_def * 100.0).round() as u32
    );
    (aura, shd_text, stats_text)
}

/// Shield bar and row container query.
type ShieldNodes<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Node,
        Option<&'static ShieldRow>,
        Option<&'static ShieldFill>,
    ),
    (Or<(With<ShieldRow>, With<ShieldFill>)>, Without<PartFill>),
>;

/// Updates shield aura border, shield bar, all ten part fills, and the bottom defense readout.
pub(super) fn update(
    (condition, shield, pack): (Res<Condition>, Option<Res<Shield>>, Option<Res<Pack>>),
    mut auras: Query<&mut BorderColor, With<RobotHud>>,
    mut parts: Query<(&PartFill, &mut Node, &mut BackgroundColor), Without<ShieldFill>>,
    mut shields: ShieldNodes,
    mut labels: Query<(&mut Text, Option<&ShieldLabel>, Option<&StatsLabel>)>,
) {
    let shd = shield.as_deref().copied().unwrap_or_default();
    let (aura, shd_text, stats_text) = readouts(&condition, shd, pack.as_deref());
    let disp = [Display::None, Display::Flex][usize::from(shd.max > 0.0)];
    for mut border in &mut auras {
        *border = BorderColor::all(aura);
    }
    for (PartFill(part), mut node, mut bg) in &mut parts {
        let hp = condition.of(*part).clamp(0.0, 1.0);
        node.height = percent(hp * 100.0);
        bg.0 = part_color(hp);
    }
    for (mut node, is_row, is_fill) in &mut shields {
        if is_row.is_some() {
            node.display = disp;
        }
        if is_fill.is_some() {
            node.width = percent(shd.share() * 100.0);
        }
    }
    for (mut label, is_shd, is_stats) in &mut labels {
        if is_shd.is_some() {
            label.set_if_neq(Text(shd_text.clone()));
        } else if is_stats.is_some() {
            label.set_if_neq(Text(stats_text.clone()));
        }
    }
}
