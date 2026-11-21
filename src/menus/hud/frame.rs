//! The frame rows: the integrity of the whole body and one line for each part that is hurt.
//! They sit in the bottom-left stack, under the instrument block, so the two never overlap.
//! Nothing shows while the body is whole.

use bevy::ecs::spawn::SpawnIter;
use bevy::prelude::*;

use super::context::Element;
use super::fade::gate;
use crate::condition::{Condition, PARTS};
use crate::ui::{text, theme};

/// One line of the frame corner: the first is the whole frame, the others the hurt parts, worst first.
#[derive(Component)]
pub(super) struct Row(usize);

/// The frame rows, a hidden line for the frame and one for each part, in a column that sits in
/// the bottom-left stack.
pub(super) fn corner() -> impl Bundle {
    (
        gate(Element::Integrity),
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: px(theme::S1),
            ..default()
        },
        Children::spawn(SpawnIter((0..=PARTS).map(row))),
    )
}

/// Run condition: the corner was just spawned and has written nothing yet.
pub(super) fn fresh(rows: Query<(), Added<Row>>) -> bool {
    !rows.is_empty()
}

/// One hidden line of the corner.
fn row(at: usize) -> impl Bundle {
    (
        Row(at),
        Node {
            display: Display::None,
            padding: UiRect::axes(px(theme::S3), px(theme::S2)),
            ..default()
        },
        BackgroundColor(theme::PANEL_DEEP),
        text("", theme::FONT_SMALL, theme::TEXT),
        TextLayout::no_wrap(),
    )
}

/// Returns the lines the corner shows for `condition`: the integrity of the frame, then each hurt
/// part with its health, worst first. Nothing for a body that is whole.
pub fn lines(condition: &Condition) -> Vec<String> {
    let hurt = condition.damaged();
    if hurt.is_empty() {
        return Vec::new();
    }
    let share = |health: f32| (health * 100.0).round();
    let mut lines = vec![format!("Frame {}%", share(condition.integrity()))];
    lines.extend(
        hurt.into_iter()
            .map(|part| format!("{} {}%", part.name(), share(condition.of(part)))),
    );
    lines
}

/// Returns the colour of the frame rows for a body of `integrity`: the warning colour under half
/// and the danger colour under a quarter.
pub fn tone(integrity: f32) -> Color {
    match integrity {
        low if low < 0.25 => theme::DANGER,
        mid if mid < 0.5 => theme::WARN,
        _ => theme::TEXT,
    }
}

/// Writes the lines of the corner when the condition of the body changes.
pub(super) fn update(
    condition: Res<Condition>,
    mut rows: Query<(&Row, &mut Node, &mut Text, &mut TextColor)>,
) {
    let lines = lines(&condition);
    let tone = tone(condition.integrity());
    for (Row(at), mut node, mut text, mut color) in &mut rows {
        color.set_if_neq(TextColor(tone));
        let display = match lines.get(*at) {
            Some(line) => {
                text.set_if_neq(Text(line.clone()));
                Display::Flex
            }
            None => Display::None,
        };
        if node.display != display {
            node.display = display;
        }
    }
}
