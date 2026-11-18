//! The mark of the ship on a chart: an arrow drawn with squares that turns to where the nose
//! points, seen from above the plane of the chart.
//!
//! The charts put east to the right and north up, so a nose along +X points right and a nose
//! along -Z points up. A nose straight up or down has no direction on the plane: the arrow then
//! takes the way the top of the ship faces, which is where the nose was heading.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::scene::Scene;
use crate::ui::theme;

/// Shortest length a direction keeps on the plane of the chart and still counts as one.
const FLAT: f32 = 1e-4;

/// Marks the arrow of the ship on the map of the computer, which turns every frame.
#[derive(Component)]
pub(super) struct ShipArrow;

/// Returns the clockwise turn in radians from up of an arrow that shows a ship with its nose
/// along `nose` and its top along `top`, both in the axes of the chart.
pub fn bearing(nose: DVec3, top: DVec3) -> f32 {
    let flat = |along: DVec3| Vec2::new(along.x as f32, -along.z as f32);
    let ahead = match flat(nose) {
        seen if seen.length() > FLAT => seen,
        _ => flat(top) * -(nose.y.signum() as f32),
    };
    ahead.x.atan2(ahead.y)
}

/// Returns the shape of an arrow that points up, three bars that widen downward, `unit` pixels
/// for the thinnest step. It turns around its middle.
pub fn arrow(unit: f32) -> impl Bundle {
    let bar = move |steps: f32| {
        let node = Node {
            width: px(steps * unit),
            height: px((1.5 * unit).round()),
            ..default()
        };
        (node, BackgroundColor(theme::TEXT))
    };
    let node = Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        width: px(5.0 * unit),
        height: px(5.0 * unit),
        ..default()
    };
    (
        node,
        UiTransform::default(),
        children![bar(1.0), bar(3.0), bar(5.0)],
    )
}

/// Turns the arrow of the ship on the map to where the nose points, every frame, so it turns
/// as smoothly as the ship does.
pub(super) fn turn(scene: Scene, mut arrows: Query<&mut UiTransform, With<ShipArrow>>) {
    let Some(angle) = scene.facing() else {
        return;
    };
    let wanted = Rot2::radians(angle);
    for mut arrow in &mut arrows {
        if arrow.rotation != wanted {
            arrow.rotation = wanted;
        }
    }
}
