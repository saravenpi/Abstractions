//! What a planet carries on the chart besides its mark: the tiny orbits of its moons, a tick
//! per moon while they are too close to tell apart, and the thin line of its rings.
//!
//! The board computer and the space view of the minimap draw them the same way.

use bevy::prelude::*;

use super::chart::Dot;
use super::map::spot;
use super::moons::{MOON_ZOOM, count, host};
use super::rows::Line;
use crate::ui::theme;
use crate::universe::{PlanetClass, PlanetSpec};

/// How many times wider the cells of a gas giant are than those of a solid planet.
const GIANT: f32 = 1.5;

/// Width in pixels of the line of the rings, across the mark of the planet.
const BAND: f32 = 14.0;

/// Side in pixels of one moon tick, and pixels from one tick to the next.
const TICK: (f32, f32) = (2.0, 3.0);

/// Pixels from the middle of a planet mark down to its ticks.
const TICK_DROP: f32 = 6.0;

/// Most ticks drawn under one planet.
const TICKS: usize = 6;

/// A chart to draw on: its dots, its rows and, for each row, the planet whose record is known.
pub type Drawn<'a> = (&'a [(Dot, Vec2)], &'a [Line], &'a [Option<PlanetSpec>]);

/// Returns the side in pixels of one cell of the mark of `planet` drawn `side` pixels wide:
/// half of it, and half again as much for a gas giant, the largest of worlds.
pub fn cell(side: f32, planet: &PlanetSpec) -> f32 {
    match planet.record.class {
        PlanetClass::GasGiant => side * 0.5 * GIANT,
        _ => side * 0.5,
    }
}

/// Returns a hairline circle `radius` chart units around the chart point `centre`.
pub fn circle(centre: Vec2, radius: f32) -> impl Bundle {
    let middle = (Vec2::new(centre.x, -centre.y) * 0.5 + 0.5 - radius * 0.5) * 100.0;
    let node = Node {
        position_type: PositionType::Absolute,
        left: percent(middle.x),
        top: percent(middle.y),
        width: percent(radius * 100.0),
        height: percent(radius * 100.0),
        border: UiRect::all(px(theme::LINE)),
        border_radius: BorderRadius::MAX,
        ..default()
    };
    (node, BorderColor::all(theme::HAIRLINE))
}

/// Spawns one tick per moon in a row under the planet mark at `at`, at most [`TICKS`].
fn ticks(sheet: &mut ChildSpawnerCommands, at: Vec2, moons: usize) {
    let shown = moons.min(TICKS);
    let (side, pitch) = TICK;
    let left = -0.5 * (pitch * shown as f32 - (pitch - side));
    for tick in 0..shown {
        let corner = Vec2::new(left + pitch * tick as f32, TICK_DROP);
        sheet.spawn((
            spot(at, corner, Vec2::splat(side)),
            BackgroundColor(theme::TEXT),
        ));
    }
}

/// Spawns the line of the rings across the planet mark at `at`, drawn before the mark so the
/// planet sits on it.
fn band(sheet: &mut ChildSpawnerCommands, at: Vec2) {
    let corner = Vec2::new(-0.5 * BAND, -0.5 * theme::LINE);
    let node = spot(at, corner, Vec2::new(BAND, theme::LINE));
    sheet.spawn((node, BackgroundColor(theme::MUTED)));
}

/// Spawns, for every row of a chart seen at `zoom`, the rings of a planet whose record is
/// known, and its moons: their orbits once they part from it, a tick for each before.
pub fn satellites(sheet: &mut ChildSpawnerCommands, (dots, lines, worlds): Drawn, zoom: f32) {
    let point = |index: usize| {
        let found = dots.iter().find(|(dot, _)| *dot == Dot::Row(index));
        found.map(|(_, at)| *at)
    };
    let apart = zoom >= MOON_ZOOM;
    for index in 0..lines.len() {
        let Some(at) = point(index) else {
            continue;
        };
        let world = worlds.get(index).and_then(Option::as_ref);
        if world.is_some_and(|planet| planet.rings.is_some()) {
            band(sheet, at);
        }
        if let Some(centre) = host(lines, index).and_then(point).filter(|_| apart) {
            sheet.spawn(circle(centre, at.distance(centre)));
        }
        let moons = count(lines, index);
        if moons > 0 && !apart {
            ticks(sheet, at, moons);
        }
    }
}
