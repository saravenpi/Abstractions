//! The instrument block shown while seated in the ship, and the system that fills it.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::context::{Context, Element, Where};
use super::flight::{Flight, Gauge, reading, symbol};
use crate::gear::Gear;
use crate::menus::key_name;
use crate::player::Player;
use crate::settings::{Persistent, Settings};
use crate::ship::{Ship, ShipTelemetry, WarpState};
use crate::space::{Anchor, Frame, altitude};
use crate::ui::{Icon, Tone, cell, icon_sized, text, theme};
use crate::universe::{PlanetClass, Universe};
use crate::world::Planet;

/// Width of the label column of the instruments, in pixels: the longest label, "Stabiliser" or
/// "Highest point".
pub(super) const LABEL_WIDTH: f32 = 112.0;

/// Width of the bar that fills while the warp drive spools, in pixels.
const BAR_WIDTH: f32 = 160.0;

/// Marks the block of instruments.
#[derive(Component)]
pub(super) struct Instruments;

/// Marks the value text of an instrument.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Reading(Gauge);

/// Marks the icon of an instrument.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Symbol(Gauge);

/// Marks the bar that fills while the warp drive spools.
#[derive(Component)]
pub(super) struct SpoolBar;

/// Marks the track of the spool bar, hidden unless the drive spools.
#[derive(Component)]
pub(super) struct Track;

/// One instrument: its icon, its label and its value, hidden until it has something to show.
fn row((gauge, name): (Gauge, &'static str)) -> impl Bundle {
    (
        gauge,
        Node {
            display: Display::None,
            align_items: AlignItems::Center,
            column_gap: px(theme::S2),
            ..default()
        },
        children![
            (
                Symbol(gauge),
                icon_sized(Icon::Radius, Tone::Muted, theme::ICON_SMALL)
            ),
            cell(LABEL_WIDTH, text(name, theme::FONT_SMALL, theme::TEXT_2)),
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
        Track,
        Node {
            display: Display::None,
            width: px(BAR_WIDTH),
            height: px(4),
            margin: UiRect::top(px(theme::S2)),
            ..default()
        },
        BackgroundColor(theme::HOVER),
        children![(
            SpoolBar,
            Node {
                width: percent(0),
                height: percent(100),
                ..default()
            },
            BackgroundColor(theme::ACCENT),
        )],
    )
}

/// The instrument block: one row per instrument and the spool bar, on a panel. It sits in the
/// bottom-left stack, above the frame rows, and shows only while a row has something to say.
/// `refresh` fills it.
pub(super) fn block() -> impl Bundle {
    (
        Instruments,
        Node {
            display: Display::None,
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(px(theme::S4), px(theme::S3)),
            ..default()
        },
        BackgroundColor(theme::PANEL),
        Children::spawn((SpawnIter(Gauge::ALL.into_iter().map(row)), Spawn(bar()))),
    )
}

/// The world the instruments read besides the telemetry.
#[derive(SystemParam)]
pub(super) struct Source<'w, 's> {
    telemetry: Option<Res<'w, ShipTelemetry>>,
    universe: Option<Res<'w, Universe>>,
    ships: Query<'w, 's, &'static Ship>,
    glider: Glider<'w, 's>,
    settings: Option<Res<'w, Persistent<Settings>>>,
}

/// What the instruments read while the player glides on foot.
#[derive(SystemParam)]
pub(super) struct Glider<'w, 's> {
    now: Res<'w, Where>,
    players: Query<'w, 's, (&'static Transform, &'static Gear), With<Player>>,
    anchor: Option<Res<'w, Anchor>>,
    planet: Option<Res<'w, Planet>>,
}

impl Glider<'_, '_> {
    /// Returns the height above the sea and the airspeed of the glide, while the player glides.
    fn reading(&self) -> Option<(f64, f64)> {
        if self.now.0 != Context::Gliding {
            return None;
        }
        let ((at, gear), (anchor, planet)) = self
            .players
            .iter()
            .next()
            .zip(self.anchor.as_deref().zip(self.planet.as_deref()))?;
        let height = altitude(anchor, &planet.globe, at.translation);
        Some((f64::from(height), f64::from(gear.state.airspeed)))
    }
}

impl Source<'_, '_> {
    /// Gathers what the instruments show.
    pub(super) fn flight(&self) -> Flight {
        let telemetry = self.telemetry.as_deref().copied().unwrap_or_default();
        let giant = match (telemetry.frame, self.universe.as_deref()) {
            (Frame::Surface(body), Some(universe)) => {
                universe.planet(body).record.class == PlanetClass::GasGiant
            }
            _ => false,
        };
        Flight {
            telemetry,
            giant,
            glide: self.glider.reading(),
            gear: self.ships.iter().next().map(|ship| ship.gear),
            headlight: self.ships.iter().next().zip(self.settings.as_deref()).map(
                |(ship, settings)| {
                    (
                        ship.headlight,
                        key_name(settings.controls.keys.ship.headlight),
                    )
                },
            ),
        }
    }
}

/// Writes every instrument whose text changed, hides those the context leaves out or that
/// have nothing to show, and hides the block while no row shows.
pub(super) fn refresh(
    (source, now): (Source, Res<Where>),
    mut rows: Query<(&Gauge, &mut Node), Without<Instruments>>,
    mut texts: Query<(&Reading, &mut Text), Without<Symbol>>,
    mut symbols: Query<(&Symbol, &mut Text), Without<Reading>>,
    mut blocks: Query<&mut Node, (With<Instruments>, Without<Gauge>)>,
) {
    let flight = source.flight();
    let mut any = false;
    for (gauge, mut node) in &mut rows {
        let shown = now.has(gauge.element()) && reading(*gauge, &flight).is_some();
        any |= shown;
        let display = [Display::None, Display::Flex][usize::from(shown)];
        if node.display != display {
            node.display = display;
        }
    }
    any |= now.has(Element::Spool) && matches!(flight.telemetry.warp, WarpState::Spooling(_));
    let display = [Display::None, Display::Flex][usize::from(any)];
    for mut node in &mut blocks {
        if node.display != display {
            node.display = display;
        }
    }
    for (Symbol(gauge), mut text) in &mut symbols {
        text.set_if_neq(Text(symbol(*gauge, &flight).glyph()));
    }
    for (Reading(gauge), mut text) in &mut texts {
        if let Some(shown) = reading(*gauge, &flight) {
            text.set_if_neq(Text(shown));
        }
    }
}

/// Fills the spool bar with the progress of the warp drive and shows its track only while the
/// drive spools.
pub(super) fn spool(
    telemetry: Option<Res<ShipTelemetry>>,
    mut bars: Query<&mut Node, (With<SpoolBar>, Without<Track>)>,
    mut tracks: Query<&mut Node, (With<Track>, Without<SpoolBar>)>,
) {
    let warp = telemetry
        .as_deref()
        .map_or(WarpState::Off, |telemetry| telemetry.warp);
    let (spooled, display) = match warp {
        WarpState::Spooling(done) => ((done * 100.0).round() as f32, Display::Flex),
        _ => (0.0, Display::None),
    };
    for mut bar in &mut bars {
        if bar.width != percent(spooled) {
            bar.width = percent(spooled);
        }
    }
    for mut track in &mut tracks {
        if track.display != display {
            track.display = display;
        }
    }
}
