//! The frame corner: the integrity of the whole body and one line for each part that is hurt, in
//! the bottom left. It shows nothing while the body is whole.

use bevy::ecs::spawn::SpawnIter;
use bevy::prelude::*;

use crate::condition::{Condition, PARTS};
use crate::ui::{text, theme};

/// One line of the frame corner: the first is the whole frame, the others the hurt parts, worst first.
#[derive(Component)]
pub(super) struct Row(usize);

/// The bottom left corner of the screen, with a hidden line for the frame and one for each part.
pub(super) fn corner() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            bottom: px(theme::GAP_M),
            left: px(theme::GAP_M),
            flex_direction: FlexDirection::Column,
            row_gap: px(theme::GAP_XS),
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
            ..default()
        },
        text("", theme::FONT_SMALL, theme::MUTED),
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

/// Writes the lines of the corner when the condition of the body changes.
pub(super) fn update(condition: Res<Condition>, mut rows: Query<(&Row, &mut Node, &mut Text)>) {
    let lines = lines(&condition);
    for (Row(at), mut node, mut text) in &mut rows {
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
