//! A column that scrolls: it follows the keyboard focus and the mouse wheel.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};

use super::theme;

/// Marks a column that scrolls vertically.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Scroller;

/// Full-width column of at most `height` pixels that scrolls when its content is taller.
pub fn scroller(height: f32) -> impl Bundle {
    (
        Scroller,
        ScrollPosition::default(),
        Node {
            width: percent(100),
            max_height: px(height),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::scroll_y(),
            ..default()
        },
    )
}

/// Returns the scroll offset that brings the span `item` (top, bottom) inside `view` (top,
/// bottom), starting from `offset`. All values are in the same unit.
pub fn reveal(offset: f32, item: (f32, f32), view: (f32, f32)) -> f32 {
    if item.0 < view.0 {
        offset - (view.0 - item.0)
    } else if item.1 > view.1 {
        offset + (item.1 - view.1)
    } else {
        offset
    }
}

/// Returns the top and bottom of a node on screen, in physical pixels.
fn span(node: &ComputedNode, at: &UiGlobalTransform) -> (f32, f32) {
    let half = node.size().y * 0.5;
    (at.translation.y - half, at.translation.y + half)
}

/// Scrolls the column that holds the focused button until the button is in view.
pub(super) fn follow_focus(
    focus: Res<InputFocus>,
    parents: Query<&ChildOf>,
    nodes: Query<(&ComputedNode, &UiGlobalTransform)>,
    mut scrollers: Query<&mut ScrollPosition, With<Scroller>>,
) {
    let Some(focused) = focus.get() else {
        return;
    };
    let Some(holder) = parents
        .iter_ancestors(focused)
        .find(|ancestor| scrollers.contains(*ancestor))
    else {
        return;
    };
    let (Ok((item, item_at)), Ok((view, view_at))) = (nodes.get(focused), nodes.get(holder)) else {
        return;
    };
    let Ok(mut position) = scrollers.get_mut(holder) else {
        return;
    };
    let scale = view.inverse_scale_factor;
    let offset = position.y / scale.max(f32::EPSILON);
    let wanted = reveal(offset, span(item, item_at), span(view, view_at)) * scale;
    if (wanted - position.y).abs() > 0.5 {
        position.y = wanted.max(0.0);
    }
}

/// Scrolls every visible column with the mouse wheel, inside its content.
pub(super) fn wheel(
    wheel: Res<AccumulatedMouseScroll>,
    mut scrollers: Query<(&mut ScrollPosition, &ComputedNode), With<Scroller>>,
) {
    if wheel.delta.y == 0.0 {
        return;
    }
    let step = match wheel.unit {
        MouseScrollUnit::Line => wheel.delta.y * theme::ROW_HEIGHT,
        MouseScrollUnit::Pixel => wheel.delta.y,
    };
    for (mut position, node) in &mut scrollers {
        let room = (node.content_size.y - node.size().y).max(0.0) * node.inverse_scale_factor;
        let wanted = (position.y - step).clamp(0.0, room);
        if wanted != position.y {
            position.y = wanted;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A span above the view scrolls up, one below scrolls down, one inside stays.
    #[test]
    fn reveal_moves_only_as_far_as_needed() {
        assert_eq!(reveal(40.0, (90.0, 110.0), (100.0, 300.0)), 30.0);
        assert_eq!(reveal(40.0, (290.0, 320.0), (100.0, 300.0)), 60.0);
        assert_eq!(reveal(40.0, (150.0, 180.0), (100.0, 300.0)), 40.0);
    }
}
