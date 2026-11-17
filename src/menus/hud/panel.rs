//! The instrument block shown while seated in the ship, and the system that fills it.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::flight::{Flight, Gauge, reading};
use crate::computer::{Destination, RouteStatus, ship_position, target_position};
use crate::ship::{Ship, ShipTelemetry, WarpState};
use crate::space::Clock;
use crate::ui::{cell, text, theme};
use crate::universe::Universe;

/// Width of the label column of the instruments, in pixels.
const LABEL_WIDTH: f32 = 64.0;

/// Width of the bar that fills while the warp drive spools, in pixels.
const BAR_WIDTH: f32 = 120.0;

/// Marks the block of instruments.
#[derive(Component)]
pub(super) struct Instruments;

/// Marks the value text of an instrument.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Reading(Gauge);

/// Marks the bar that fills while the warp drive spools.
#[derive(Component)]
pub(super) struct SpoolBar;

/// One instrument: its label and its value, hidden until it has something to show.
fn row((gauge, name): (Gauge, &'static str)) -> impl Bundle {
    (
        gauge,
        Node {
            display: Display::None,
            column_gap: px(theme::GAP_S),
            ..default()
        },
        children![
            cell(LABEL_WIDTH, text(name, theme::FONT_SMALL, theme::MUTED)),
            (
                Reading(gauge),
                text("", theme::FONT_SMALL, theme::TEXT),
                TextLayout::no_wrap()
            ),
        ],
    )
}

/// The thin bar that fills while the warp drive spools.
fn bar() -> impl Bundle {
    (
        Node {
            width: px(BAR_WIDTH),
            height: px(2),
            margin: UiRect::top(px(theme::GAP_XS)),
            ..default()
        },
        BackgroundColor(theme::HAIRLINE),
        children![(
            SpoolBar,
            Node {
                width: percent(0),
                height: percent(100),
                ..default()
            },
            BackgroundColor(theme::TEXT),
        )],
    )
}

/// The instrument block in the bottom-left corner: one row per instrument and the spool bar.
/// `refresh` fills it.
pub(super) fn block() -> impl Bundle {
    (
        Instruments,
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            left: px(theme::EDGE),
            bottom: px(theme::EDGE),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(theme::GAP_S)),
            ..default()
        },
        BackgroundColor(theme::STRIP),
        Children::spawn((SpawnIter(Gauge::ALL.into_iter().map(row)), Spawn(bar()))),
    )
}

/// The world the instruments read besides the telemetry.
#[derive(SystemParam)]
pub(super) struct Source<'w, 's> {
    telemetry: Res<'w, ShipTelemetry>,
    route: Res<'w, RouteStatus>,
    destination: Res<'w, Destination>,
    universe: Option<Res<'w, Universe>>,
    clock: Option<Res<'w, Clock>>,
    ships: Query<'w, 's, &'static Ship>,
}

impl Source<'_, '_> {
    /// Gathers what the instruments show.
    pub(super) fn flight(&self) -> Flight {
        let time = self.clock.as_deref().map_or(0.0, |clock| clock.seconds);
        let found = self.destination.0.zip(self.universe.as_deref());
        let destination = found.and_then(|(target, universe)| {
            let ship = self.ships.iter().next()?;
            let here = ship_position(universe, ship.frame, ship.pos, time)?;
            let there = target_position(universe, target, time)?;
            Some((target.name(universe), there.distance(here)))
        });
        Flight {
            telemetry: *self.telemetry,
            route: *self.route,
            destination,
        }
    }
}

/// Writes every instrument whose text changed and hides those with nothing to show.
pub(super) fn refresh(
    source: Source,
    mut rows: Query<(&Gauge, &mut Node)>,
    mut texts: Query<(&Reading, &mut Text)>,
) {
    let flight = source.flight();
    for (gauge, mut node) in &mut rows {
        let display = match reading(*gauge, &flight) {
            Some(_) => Display::Flex,
            None => Display::None,
        };
        if node.display != display {
            node.display = display;
        }
    }
    for (Reading(gauge), mut text) in &mut texts {
        if let Some(shown) = reading(*gauge, &flight) {
            text.set_if_neq(Text(shown));
        }
    }
}

/// Fills the spool bar with the progress of the warp drive, and empties it otherwise.
pub(super) fn spool(telemetry: Res<ShipTelemetry>, mut bars: Query<&mut Node, With<SpoolBar>>) {
    let spooled = match telemetry.warp {
        WarpState::Spooling(done) => (done * 100.0).round() as f32,
        _ => 0.0,
    };
    for mut bar in &mut bars {
        if bar.width != percent(spooled) {
            bar.width = percent(spooled);
        }
    }
}
