//! Tests of the tiles that cover the view: at every zoom each tile in view is painted or held by
//! a coarser painted tile, zooming and panning keep the view whole or make it whole within a
//! few paintings, and tiles never painted go before those that show older ground.

use bevy::math::DVec2;
use bevy::prelude::*;

use super::super::cover::ancestor;
use super::super::tile::{TileId, ZOOM_FROM, level_for, visible};
use super::{Tiles, trim, want};
use crate::map::level::CLOSEST;
use crate::map::sight::Sight;
use crate::map::{Level, Surface};
use crate::universe::BodyId;

/// Most rounds of painting, each as many tiles as there are workers, that a view may take to
/// be whole after a move: the coarse level painted first is at most six tiles.
const ROUNDS: usize = 3;

/// The point of the chart the tests look at, far from the middle of the chart.
const CENTRE: DVec2 = DVec2::new(0.731, 0.183);

/// Returns an app that asks for the tiles of the lens and trims the cache, as the map does.
fn cache() -> App {
    let mut app = App::new();
    app.init_resource::<Tiles>()
        .insert_resource(Level::Planet(BodyId::default()))
        .init_resource::<Surface>()
        .add_systems(Update, (want, trim).chain());
    app
}

/// Looks through `lens` for one frame.
fn look(app: &mut App, lens: Sight) {
    app.world_mut().resource_mut::<Surface>().lens = lens;
    app.update();
}

/// Paints, as the workers would at once, the tiles the cache hands to its free workers, at
/// ground version `version`. Returns how many it painted.
fn bake(app: &mut App, version: u64) -> usize {
    let mut tiles = app.world_mut().resource_mut::<Tiles>();
    let body = BodyId::default();
    let todo = tiles.todo(body, version);
    for id in &todo {
        if let Some(slot) = tiles.slots.get_mut(&(body, *id)) {
            (slot.image, slot.seen) = (Some(Handle::default()), version);
        }
    }
    todo.len()
}

/// Returns true when every tile `lens` looks at on the level of its zoom is drawn by itself or
/// by one of its ancestors, and the cache says the view is covered.
fn whole(app: &App, lens: Sight) -> bool {
    let (drawn, covered) = app.world().resource::<Tiles>().ready(lens);
    let level = level_for(lens.zoom);
    let held = |tile: (i64, TileId)| {
        drawn.iter().any(|(col, id, _)| {
            id.level <= level && ancestor(tile, level - id.level) == (*col, *id)
        })
    };
    covered && visible(lens, level).into_iter().all(held)
}

/// Paints round after round until the view of `lens` is whole, at most [`ROUNDS`] rounds.
fn whole_within_rounds(app: &mut App, lens: Sight) -> bool {
    for _ in 0..ROUNDS {
        if whole(app, lens) {
            return true;
        }
        bake(app, 1);
    }
    whole(app, lens)
}

/// At every zoom from the first one with tiles to the closest, a view starts with nothing and
/// is whole once the coarse level is painted, before any tile of its own level.
#[test]
fn every_tile_in_view_is_baked_or_covered_at_every_zoom() {
    let mut zoom = ZOOM_FROM;
    while zoom <= CLOSEST {
        let mut app = cache();
        let lens = Sight {
            zoom,
            centre: CENTRE,
        };
        look(&mut app, lens);
        assert!(!whole(&app, lens), "whole before any painting at {zoom}");
        assert!(whole_within_rounds(&mut app, lens), "a hole at zoom {zoom}");
        zoom *= 1.25;
    }
}

/// Zooming in from a painted view is whole at once, drawn by the coarser tiles of the view
/// before. Panning at the closest zoom and zooming back out are whole within a few rounds.
#[test]
fn panning_and_zooming_never_leave_a_hole() {
    let mut app = cache();
    let mut lens = Sight {
        zoom: 8.0,
        centre: CENTRE,
    };
    look(&mut app, lens);
    while bake(&mut app, 1) > 0 {}
    while lens.zoom < CLOSEST {
        lens.zoom = (lens.zoom * 1.2).min(CLOSEST);
        look(&mut app, lens);
        assert!(whole(&app, lens), "a hole zooming in to {}", lens.zoom);
        while bake(&mut app, 1) > 0 {}
    }
    let step = DVec2::new(0.6 / f64::from(CLOSEST), 0.25 / f64::from(CLOSEST));
    for at in 0..40 {
        lens.centre = CENTRE + step * f64::from(at);
        look(&mut app, lens);
        assert!(whole_within_rounds(&mut app, lens), "a hole after pan {at}");
        while bake(&mut app, 1) > 0 {}
    }
    while lens.zoom > ZOOM_FROM {
        lens.zoom = (lens.zoom / 1.2).max(ZOOM_FROM);
        look(&mut app, lens);
        assert!(
            whole_within_rounds(&mut app, lens),
            "a hole zooming out to {}",
            lens.zoom
        );
        while bake(&mut app, 1) > 0 {}
    }
}

/// After a pan, the workers go to the tiles never painted before any tile that only shows an
/// older version of the explored ground, so a moving player never starves the new tiles.
#[test]
fn tiles_never_painted_come_before_stale_ones() {
    let mut app = cache();
    let mut lens = Sight {
        zoom: 200.0,
        centre: CENTRE,
    };
    look(&mut app, lens);
    while bake(&mut app, 1) > 0 {}
    lens.centre.x += 1.0 / f64::from(lens.zoom);
    look(&mut app, lens);
    let tiles = app.world().resource::<Tiles>();
    let body = BodyId::default();
    let todo = tiles.todo(body, 2);
    assert!(!todo.is_empty());
    let fresh = |id: &TileId| tiles.slots[&(body, *id)].image.is_none();
    assert!(todo.iter().all(fresh), "a stale tile went first: {todo:?}");
}
