//! Tests of the cache of tiles: zooming from the whole planet to the closest zoom and panning
//! there keeps it under its bound, in count and in bytes, and keeps the tiles asked for.

use bevy::math::DVec2;
use bevy::prelude::*;

use super::{KEPT, KEPT_BYTES, TileId, Tiles, trim, want};
use crate::map::level::{CLOSEST, WHOLE};
use crate::map::sight::Sight;
use crate::map::{Level, Surface};
use crate::universe::BodyId;

/// Returns an app that asks for the tiles of the lens and trims the cache, as the map does.
fn cache() -> App {
    let mut app = App::new();
    app.init_resource::<Tiles>()
        .insert_resource(Level::Planet(BodyId::default()))
        .init_resource::<Surface>()
        .add_systems(Update, (want, trim).chain());
    app
}

/// Looks through `lens`, and checks the cache is under its bound and holds every tile asked for.
fn look(app: &mut App, lens: Sight) {
    app.world_mut().resource_mut::<Surface>().lens = lens;
    app.update();
    let tiles = app.world().resource::<Tiles>();
    assert!(
        tiles.slots.len() <= KEPT,
        "{} tiles at {lens:?}",
        tiles.slots.len()
    );
    let (body, asked) = &tiles.wanted;
    let kept = |id: &TileId| tiles.slots.contains_key(&(body.expect("a body"), *id));
    assert!(
        asked.iter().all(|(_, id)| kept(id)),
        "a tile asked for was dropped at {lens:?}"
    );
}

/// Zooming in through every level, far from the middle of the chart, then panning at the closest
/// zoom keeps at most [`KEPT`] tiles, about 25 MB, and never drops one the map shows.
#[test]
fn the_cache_stays_bounded_through_every_level() {
    const { assert!(KEPT_BYTES <= 32 << 20) };
    let mut app = cache();
    let centre = DVec2::new(0.731, 0.183);
    let mut zoom = WHOLE;
    while zoom < CLOSEST {
        look(&mut app, Sight { zoom, centre });
        zoom *= 1.2;
    }
    let step = DVec2::new(1.5 / f64::from(CLOSEST), 0.0);
    for at in 0..60 {
        let centre = centre + step * f64::from(at);
        look(
            &mut app,
            Sight {
                zoom: CLOSEST,
                centre,
            },
        );
    }
}
