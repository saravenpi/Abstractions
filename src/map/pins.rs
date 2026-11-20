//! The markers on the map: the player, the friends in their colours, the pins, home and the
//! ship, each drawn as its icon with its name, over nothing: no square sits behind a marker.
//!
//! A fixed pool of nodes is spawned with the map and moved every frame, so a moving friend
//! costs no spawn. On a planet every marker on it is drawn; on a chart the friends and the
//! waypoints, on the mark of the body they are on.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::geo::{copy_near, to_chart};
use super::level::Level;
use super::me::Me;
use super::pins_draw::glyph;
use super::spots::place;
use super::surface::Surface;
use crate::computer::{Board, MapView};
use crate::menus::{MarkerKind, MarkerPlace, Markers};
use crate::space::{Anchor, Clock, Frame};
use crate::ui::{Icon, text, theme};
use crate::universe::Universe;

/// Markers drawn at most at once.
pub const POOL: usize = 32;

/// One marker to draw: its chart point and the marker it stands for, or nothing for the player.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pin {
    /// Chart point of the marker on the level shown.
    pub at: Vec2,
    /// Index of the marker in [`Markers`], or nothing for the player.
    pub marker: Option<usize>,
}

/// The markers drawn this frame, the player first when on the planet shown.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Pins(pub Vec<Pin>);

/// Marks the layer the pool is spawned in, over the planet and the chart.
#[derive(Component)]
pub struct PinLayer;

/// One node of the pool, at the point of the marker it draws.
#[derive(Component, Debug, Clone, Copy)]
pub struct PinSlot(pub usize);

/// The icon of a node of the pool.
#[derive(Component, Debug, Clone, Copy)]
pub struct PinDot(pub usize);

/// The name of a node of the pool.
#[derive(Component, Debug, Clone, Copy)]
pub struct PinLabel(pub usize);

/// Side in pixels of the footprint of a marker, which names keep clear of.
pub const DOT: f32 = 8.0;

/// Side in pixels of the icon of a marker that is not the selected one.
pub const GLYPH_SIZE: f32 = theme::ICON_SMALL;

/// Pixels from the point of a marker to the start of its name.
pub const NAME_GAP: f32 = 12.0;

/// The icon of a node of the pool, centred on its point.
fn dot(index: usize) -> impl Bundle {
    let node = Node {
        position_type: PositionType::Absolute,
        left: px(-0.5 * GLYPH_SIZE),
        top: px(-0.5 * GLYPH_SIZE),
        width: px(GLYPH_SIZE),
        height: px(GLYPH_SIZE),
        ..default()
    };
    (
        PinDot(index),
        node,
        glyph(Icon::MapPin, GLYPH_SIZE, theme::PIN),
    )
}

/// The name of a node of the pool, to the right of its icon.
fn name(index: usize) -> impl Bundle {
    let node = Node {
        position_type: PositionType::Absolute,
        left: px(NAME_GAP),
        top: px(-0.6 * theme::FONT_SMALL),
        ..default()
    };
    let words = text("", theme::FONT_SMALL, theme::TEXT);
    (PinLabel(index), node, words, TextLayout::no_wrap())
}

/// One hidden node of the pool.
fn slot(index: usize) -> impl Bundle {
    let anchor = Node {
        display: Display::None,
        position_type: PositionType::Absolute,
        ..default()
    };
    (PinSlot(index), anchor, children![dot(index), name(index)])
}

/// The layer of the markers over the whole map, with its pool of hidden nodes.
pub fn pin_layer() -> impl Bundle {
    let node = Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        ..default()
    };
    (
        PinLayer,
        node,
        Children::spawn(SpawnIter((0..POOL).map(slot))),
    )
}

/// What the markers are placed from.
#[derive(SystemParam)]
pub struct Source<'w, 's> {
    /// The level shown.
    level: Res<'w, Level>,
    /// Every marker of this frame.
    markers: Res<'w, Markers>,
    /// Where the player is.
    me: Me<'w, 's>,
    /// The rows of the chart.
    board: Res<'w, Board>,
    /// The dots of the chart.
    view: Res<'w, MapView>,
    /// The universe, its time and the render anchor.
    world: Option<(Res<'w, Universe>, Res<'w, Clock>, Res<'w, Anchor>)>,
}

impl Source<'_, '_> {
    /// Returns the chart point of the marker at `index` on the flat map of the planet shown.
    fn on_planet(&self, index: usize) -> Option<Vec2> {
        let Level::Planet(body) = *self.level else {
            return None;
        };
        let frame = self.world.as_ref().map(|(_, _, anchor)| anchor.frame);
        match self.markers.0.get(index)?.place {
            MarkerPlace::Body(on, pos) if on == body => Some(to_chart(pos)),
            MarkerPlace::Anchor(pos) if frame == Some(Frame::Surface(body)) => Some(to_chart(pos)),
            _ => None,
        }
    }

    /// Returns the chart point of the marker at `index` on the chart shown: friends and
    /// the active pin only, since the chart draws home and the ship itself.
    fn on_chart(&self, index: usize) -> Option<Vec2> {
        let marker = self.markers.0.get(index)?;
        let (universe, clock, anchor) = self.world.as_ref()?;
        let kinds = [MarkerKind::Friend, MarkerKind::Waypoint];
        let chart = (&*self.board, self.view.dots.as_slice(), self.view.span);
        let sky = (&**universe, anchor.frame, clock.seconds);
        kinds
            .contains(&marker.kind)
            .then(|| place(marker, chart, sky))?
    }
}

/// Finds where every marker is drawn on the level shown, the player first on its planet. The
/// list is gathered aside and written only when it changed, so the frames it stays the same
/// do not count as changes.
pub(super) fn collect(source: Source, mut pins: ResMut<Pins>, mut found: Local<Pins>) {
    let list = &mut found.0;
    list.clear();
    if let Level::Planet(body) = *source.level
        && let Some((_, at)) = source.me.spot().filter(|(on, _)| *on == body)
    {
        list.push(Pin {
            at: to_chart(at),
            marker: None,
        });
    }
    let planet = matches!(*source.level, Level::Planet(_));
    for index in 0..source.markers.0.len() {
        let at = match planet {
            true => source.on_planet(index),
            false => source.on_chart(index),
        };
        if let Some(at) = at.filter(|_| list.len() < POOL) {
            list.push(Pin {
                at,
                marker: Some(index),
            });
        }
    }
    if pins.0 != found.0 {
        pins.0.clone_from(&found.0);
    }
}

/// Returns the pixel of the map the chart point `at` is drawn at on the level shown: on a
/// planet, its copy nearest the longitude the lens looks at.
pub(super) fn pixel(level: Level, (surface, view): (&Surface, &MapView), at: Vec2) -> Vec2 {
    match level {
        Level::Chart => view.lens.to_map(at),
        Level::Planet(_) => surface.lens.to_map(copy_near(at, surface.lens.centre.x)),
    }
}
