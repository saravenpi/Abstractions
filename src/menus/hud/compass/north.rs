//! The north mark of the compass: an N over the line at the bearing of the north pole of the
//! planet under the player, faded toward the ends like the other marks, hidden off a planet.

use bevy::prelude::*;

use super::aim::Around;
use super::{MARK, ROOM};
use crate::menus::hud::needle::{WIDTH, fade, slot};
use crate::ui::text;

/// Marks the room of the north mark, centred on its place on the bar.
#[derive(Component)]
pub struct North;

/// Marks the letter of the north mark.
#[derive(Component)]
pub struct Letter;

/// The north mark, hidden until the player is on a planet.
pub(super) fn mark() -> impl Bundle {
    (
        North,
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            width: px(ROOM),
            top: px(0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        children![(Letter, text("N", MARK, Color::WHITE), TextLayout::no_wrap())],
    )
}

/// Moves the north mark to the bearing of the pole, faded toward the ends of the bar, or hides
/// it off a planet, at a pole, or behind the player.
pub fn point(
    around: Around,
    mut marks: Query<&mut Node, With<North>>,
    mut letters: Query<&mut TextColor, With<Letter>>,
) {
    let shown = around
        .north()
        .and_then(|bearing| Some((bearing, fade(bearing)?)));
    for mut node in &mut marks {
        let display = shown.map_or(Display::None, |_| Display::Flex);
        let left = shown.map_or(node.left, |(bearing, _)| {
            px(slot(bearing) * WIDTH - ROOM * 0.5)
        });
        if (node.display, node.left) != (display, left) {
            (node.display, node.left) = (display, left);
        }
    }
    let alpha = shown.map_or(0.0, |(_, alpha)| alpha);
    for mut color in &mut letters {
        color.set_if_neq(TextColor(Color::WHITE.with_alpha(alpha)));
    }
}
