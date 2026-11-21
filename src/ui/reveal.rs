//! Bringing a node into view: a screen that moves its own cursor, rather than the keyboard focus,
//! marks the node under the cursor with [`Reveal`], and the scroll area around it scrolls once until
//! the node shows. It reads the layout of the last frame, so a node spawned this frame comes into
//! view on the next one. Once laid out and shown, the node loses the mark, so the wheel scrolls
//! the area freely again.

use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};

use super::scroll::{Scroller, reveal};

/// Marks a node its scroll area brings into view once.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Reveal;

/// Returns the top and bottom of a node on screen, in physical pixels.
fn ends(node: &ComputedNode, at: &UiGlobalTransform) -> (f32, f32) {
    let half = node.size().y * 0.5;
    (at.translation.y - half, at.translation.y + half)
}

/// Scrolls the column that holds each node marked [`Reveal`] until the node is in view, then
/// takes the mark off a node that has been laid out.
pub(super) fn keep_in_view(
    marked: Query<Entity, With<Reveal>>,
    parents: Query<&ChildOf>,
    nodes: Query<(&ComputedNode, &UiGlobalTransform)>,
    mut scrollers: Query<&mut ScrollPosition, With<Scroller>>,
    mut commands: Commands,
) {
    for target in &marked {
        let Some(holder) = parents
            .iter_ancestors(target)
            .find(|ancestor| scrollers.contains(*ancestor))
        else {
            continue;
        };
        let (Ok((item, item_at)), Ok((view, view_at))) = (nodes.get(target), nodes.get(holder))
        else {
            continue;
        };
        if item.size().y > 0.0 {
            commands.entity(target).remove::<Reveal>();
        }
        let Ok(mut position) = scrollers.get_mut(holder) else {
            continue;
        };
        let scale = view.inverse_scale_factor;
        let offset = position.y / scale.max(f32::EPSILON);
        let wanted = reveal(offset, ends(item, item_at), ends(view, view_at)) * scale;
        if (wanted - position.y).abs() > 0.5 {
            position.y = wanted.max(0.0);
        }
    }
}
