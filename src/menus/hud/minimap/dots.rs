//! The coloured marks of the ground view: friends as dots, waypoints as diamonds and
//! favourites as small squares, each in its own colour and waiting on the rim while out of
//! range. They come from the markers, in a pool that grows when more show than it holds.

use std::f32::consts::FRAC_PI_4;

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::marks::{lens, put};
use super::pose::Reader;
use super::scope::Scope;
use super::sight::spot;
use crate::computer::spot as corner;
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};
use crate::ui::theme;
use crate::universe::BodyId;

/// Marks the layer the coloured marks sit on.
#[derive(Component)]
pub struct Dots;

/// One coloured mark of the pool and the slot it shows among the marks of this frame.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dot(pub usize);

/// Returns the side in pixels of the mark of a marker of `kind`, and whether it is turned an
/// eighth of a turn into a diamond, or nothing for a kind the minimap leaves to its own marks.
pub fn shape(kind: MarkerKind) -> Option<(f32, bool)> {
    match kind {
        MarkerKind::Friend => Some((8.0, false)),
        MarkerKind::Waypoint => Some((9.0, true)),
        MarkerKind::Favourite => Some((6.0, false)),
        MarkerKind::Herd => Some((5.0, false)),
        MarkerKind::Home | MarkerKind::Ship => None,
    }
}

/// One hidden mark of the pool at `slot`.
fn dot(slot: usize) -> impl Bundle {
    let node = Node {
        display: Display::None,
        border: UiRect::all(px(theme::LINE)),
        ..corner(Vec2::ZERO, Vec2::ZERO, Vec2::ZERO)
    };
    (
        Dot(slot),
        node,
        UiTransform::default(),
        BackgroundColor(Color::WHITE),
        BorderColor::all(theme::INK),
    )
}

/// The layer of the coloured marks, empty until markers show.
pub fn layer() -> impl Bundle {
    let node = Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        ..default()
    };
    (Dots, node)
}

/// The nodes of the coloured marks.
#[derive(SystemParam)]
pub struct Pool<'w, 's> {
    /// Spawns marks when the pool is too small.
    commands: Commands<'w, 's>,
    /// The layer the marks sit on.
    layers: Query<'w, 's, Entity, With<Dots>>,
    /// The marks of the pool.
    dots: Query<
        'w,
        's,
        (
            &'static Dot,
            &'static mut Node,
            &'static mut UiTransform,
            &'static mut BackgroundColor,
        ),
    >,
}

/// Returns the point of `marker` on the planet the player is on, in metres from its centre,
/// when the minimap draws it.
fn on_ground(marker: &Marker, body: BodyId) -> Option<DVec3> {
    shape(marker.kind)?;
    match marker.place {
        MarkerPlace::Body(on, at) if on == body => Some(at),
        MarkerPlace::Anchor(at) => Some(at),
        MarkerPlace::Body(..) => None,
    }
}

/// Sizes, turns and colours the mark `dot` for `marker`.
fn draw(dot: (Mut<Node>, Mut<UiTransform>, Mut<BackgroundColor>), marker: &Marker) {
    let (mut node, mut turn, mut fill) = dot;
    let Some((side, diamond)) = shape(marker.kind) else {
        return;
    };
    let (size, back) = (px(side), px(-0.5 * side));
    if (node.width, node.height, node.margin.left) != (size, size, back) {
        (node.width, node.height) = (size, size);
        node.margin = UiRect {
            left: back,
            top: back,
            ..default()
        };
    }
    let rotation = Rot2::radians(if diamond { FRAC_PI_4 } else { 0.0 });
    turn.set_if_neq(UiTransform {
        rotation,
        ..UiTransform::IDENTITY
    });
    fill.set_if_neq(BackgroundColor(marker.color));
}

/// Puts every friend, waypoint and favourite of the planet the player is on where it lies on
/// the ground view, in its colour and shape, and hides the rest of the pool.
pub fn place(
    reader: Reader,
    scope: Res<Scope>,
    markers: Res<Markers>,
    mut pool: Pool,
    mut seen: Local<Vec<(usize, Vec2)>>,
) {
    seen.clear();
    if let Some((pose, patch)) = lens(&reader, &scope) {
        let points = markers.0.iter().enumerate();
        let drawn =
            points.filter_map(|(index, marker)| Some((index, on_ground(marker, pose.body)?)));
        seen.extend(drawn.map(|(index, at)| (index, spot(&pose, &patch, scope.radius, at))));
    }
    let held = pool.dots.iter().count();
    for layer in pool.layers.iter().filter(|_| held < seen.len()) {
        for slot in held..seen.len() {
            pool.commands.entity(layer).with_child(dot(slot));
        }
    }
    for (dot, mut node, turn, fill) in &mut pool.dots {
        let mark = seen.get(dot.0).map(|&(index, at)| (&markers.0[index], at));
        put(&mut node, mark.map(|(_, at)| at));
        if let Some((marker, _)) = mark {
            draw((node, turn, fill), marker);
        }
    }
}
