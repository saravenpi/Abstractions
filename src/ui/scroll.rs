//! A column that scrolls: it follows the keyboard focus and the mouse wheel. A column that
//! knows where the pointer is scrolls only while the pointer is over it, so two columns on one
//! screen, or a column beside a picture that zooms, never move together. A [`scroll_area`] adds
//! a thin bar that shows how much there is and where the view is in it.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, RelativeCursorPosition, UiGlobalTransform};

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

/// Width in pixels of the bar of a scroll area.
pub const BAR: f32 = theme::S1;

/// Pixels between the content of a scroll area and its bar, which sits in the padding of the
/// panel around it and never over the text.
pub const BAR_GAP: f32 = theme::S2;

/// Marks the box of a scroll area: its scrolling column and its bar.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ScrollArea;

/// Marks the bar of a scroll area, hidden while everything fits.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Thumb;

/// A column that takes the room its parent column has left and scrolls `content`, its children,
/// inside it, under the pointer only. Its bar shows beside it while the content is taller.
pub fn scroll_area(content: impl Bundle) -> impl Bundle {
    let area = Node {
        width: percent(100),
        flex_grow: 1.0,
        flex_basis: px(0),
        min_height: px(0),
        ..default()
    };
    let column = Node {
        width: percent(100),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        overflow: Overflow::scroll_y(),
        ..default()
    };
    let bar = Node {
        display: Display::None,
        position_type: PositionType::Absolute,
        right: px(-(BAR_GAP + BAR)),
        width: px(BAR),
        ..default()
    };
    let view = (
        Scroller,
        ScrollPosition::default(),
        RelativeCursorPosition::default(),
    );
    (
        ScrollArea,
        area,
        children![
            (view, column, content),
            (Thumb, bar, BackgroundColor(theme::TEXT_2))
        ],
    )
}

/// Returns where the bar of a view `view` pixels tall sits over content `content` pixels tall
/// scrolled by `offset` pixels: its top and its height as shares of the view, or nothing when
/// the content fits.
pub fn thumb(view: f32, content: f32, offset: f32) -> Option<(f32, f32)> {
    if content <= view + 0.5 || view <= 0.0 {
        return None;
    }
    let share = view / content;
    let top = (offset / content).clamp(0.0, 1.0 - share);
    Some((top, share))
}

/// Places the bar of every scroll area from its column: shown only while the content is taller
/// than the view, as tall as the share in view, as far down as the view is scrolled.
pub(super) fn bars(
    areas: Query<&Children, With<ScrollArea>>,
    views: Query<(&ComputedNode, &ScrollPosition), With<Scroller>>,
    mut thumbs: Query<&mut Node, With<Thumb>>,
) {
    for children in &areas {
        let Some((view, offset)) = children.into_iter().find_map(|c| views.get(*c).ok()) else {
            continue;
        };
        let scrolled = offset.y / view.inverse_scale_factor.max(f32::EPSILON);
        let place = thumb(view.size().y, view.content_size.y, scrolled);
        let wanted = match place {
            Some((top, share)) => (Display::Flex, percent(top * 100.0), percent(share * 100.0)),
            None => (Display::None, percent(0), percent(0)),
        };
        for child in children {
            let Ok(mut node) = thumbs.get_mut(*child) else {
                continue;
            };
            if (node.display, node.top, node.height) != wanted {
                (node.display, node.top, node.height) = wanted;
            }
        }
    }
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

/// A scrolling column: its offset, its size and where the pointer is, if it knows.
type Column = (
    &'static mut ScrollPosition,
    &'static ComputedNode,
    Option<&'static RelativeCursorPosition>,
);

/// Scrolls the columns with the mouse wheel, inside their content: a column that knows where
/// the pointer is only while the pointer is over it, the others always.
pub(super) fn wheel(
    wheel: Res<AccumulatedMouseScroll>,
    mut scrollers: Query<Column, With<Scroller>>,
) {
    if wheel.delta.y == 0.0 {
        return;
    }
    let step = match wheel.unit {
        MouseScrollUnit::Line => wheel.delta.y * theme::ROW_HEIGHT,
        MouseScrollUnit::Pixel => wheel.delta.y,
    };
    for (mut position, node, pointer) in &mut scrollers {
        if pointer.is_some_and(|pointer| !pointer.cursor_over) {
            continue;
        }
        let room = (node.content_size.y - node.size().y).max(0.0) * node.inverse_scale_factor;
        let wanted = (position.y - step).clamp(0.0, room);
        if wanted != position.y {
            position.y = wanted;
        }
    }
}

#[cfg(test)]
#[path = "scroll_tests.rs"]
mod tests;
