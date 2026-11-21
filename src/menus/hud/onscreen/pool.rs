//! The pool of six marker nodes, spawned once: a room with an icon, an arrow and a distance.

use bevy::prelude::*;

use super::place::MAX;
use crate::ui::{ICON_FONT, Icon, text, theme};

/// Width of the room of one marker, centred on its icon.
pub const ROOM: f32 = 96.0;

/// Marks the room of the marker of the pool at this slot.
#[derive(Component)]
pub struct Slot(pub usize);

/// Marks the icon of the marker of the pool at this slot.
#[derive(Component)]
pub struct Glyph(pub usize);

/// Marks the arrow of the marker of the pool at this slot.
#[derive(Component)]
pub struct Arrow(pub usize);

/// Marks the distance of the marker of the pool at this slot.
#[derive(Component)]
pub struct Label(pub usize);

/// The nodes of the pool, by kind. The filters keep the queries apart.
pub type Nodes<'w, 's> = (
    Query<'w, 's, (&'static Slot, &'static mut Node), (Without<Arrow>, Without<Glyph>)>,
    Query<
        'w,
        's,
        (&'static Glyph, &'static mut Text, &'static mut TextColor),
        (Without<Arrow>, Without<Label>),
    >,
    Query<
        'w,
        's,
        (
            &'static Arrow,
            &'static mut Node,
            &'static mut UiTransform,
            &'static mut TextColor,
        ),
        (Without<Slot>, Without<Glyph>, Without<Label>),
    >,
    Query<
        'w,
        's,
        (&'static Label, &'static mut Text, &'static mut TextColor),
        (Without<Glyph>, Without<Arrow>),
    >,
);

/// The icon of a marker, hidden until a marker fills it.
fn glyph(at: usize) -> impl Bundle {
    (
        Glyph(at),
        Text::default(),
        TextFont::from_font_size(theme::ICON).with_font(ICON_FONT),
        TextLayout::no_wrap(),
        TextColor(Color::WHITE),
        TextShadow::default(),
    )
}

/// The arrow of a marker that sits on the border, over its icon, moved and turned by `draw`.
fn arrow(at: usize) -> impl Bundle {
    (
        Arrow(at),
        Text::new(Icon::Arrow.glyph()),
        TextFont::from_font_size(theme::ICON).with_font(ICON_FONT),
        TextLayout::no_wrap(),
        TextColor(Color::WHITE),
        TextShadow::default(),
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            left: px((ROOM - theme::ICON) * 0.5),
            top: px(0),
            ..default()
        },
        UiTransform::default(),
    )
}

/// The distance under a marker.
fn label(at: usize) -> impl Bundle {
    (
        Label(at),
        text("", theme::FONT_SMALL, theme::TEXT),
        TextLayout::no_wrap(),
        TextShadow::default(),
        Node {
            padding: UiRect::horizontal(px(theme::S1)),
            ..default()
        },
        BackgroundColor(theme::PANEL),
    )
}

/// One slot of the pool: a room with an icon, an arrow and a distance, hidden.
fn slot(at: usize) -> impl Bundle {
    (
        Slot(at),
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            width: px(ROOM),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
        children![glyph(at), arrow(at), label(at)],
    )
}

/// The slots of the pool, as the children of the layer.
pub fn slots() -> impl Bundle {
    Children::spawn(SpawnIter((0..MAX).map(slot)))
}
