//! Tests of the tiles in a headless app: with no terrain loaded, the tiles of the view are
//! painted from the planet itself, seed and all.

use std::time::{Duration, Instant};

use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use super::tile::SIDE;
use super::{BYTES, Rasters, Tiles, UNSEEN, colour};
use crate::computer::Computer;
use crate::map::explored::Explored;
use crate::map::explored::grid::{centre, depth_for, leaf_of};
use crate::map::sight::Sight;
use crate::map::{Level, Surface, to_chart};
use crate::screens::Screen;
use crate::universe::Universe;
use crate::world::{Globe, Terrain};

/// Returns a headless app that paints the map of a world of seed 7 where only the leaf at
/// `dir` of the home planet is explored, the map open on it at `zoom`, and no terrain loaded.
fn painter(dir: DVec3, zoom: f32) -> App {
    let universe = Universe::new(7);
    let body = universe.home();
    let mut explored = Explored::default();
    explored.reveal(body, dir, 1e-7, universe.planet(body).radius);
    let lens = Sight {
        zoom,
        centre: to_chart(dir),
    };
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin))
        .init_asset::<Image>()
        .init_state::<Screen>()
        .insert_state(Computer::Open)
        .insert_resource(universe)
        .insert_resource(explored)
        .insert_resource(Level::Planet(body))
        .insert_resource(Surface {
            lens,
            goal: lens,
            ..default()
        })
        .add_plugins(super::plugin);
    app.update();
    app.world_mut().resource_mut::<Rasters>().request(body);
    app
}

/// Runs frames until the tiles cover the view of the map. Fails after two minutes.
fn covered(app: &mut App) -> Sight {
    let lens = app.world().resource::<Surface>().lens;
    let start = Instant::now();
    while !app.world().resource::<Tiles>().ready(lens).1 {
        assert!(start.elapsed() < Duration::from_secs(120), "never covered");
        app.update();
        std::thread::sleep(Duration::from_millis(2));
    }
    lens
}

/// With no terrain loaded at all, the tiles of the view are painted from the planet's own
/// ground: the explored pixel has the colour of a planet built afresh from its seed, and a
/// pixel far from it stays clear.
#[test]
fn tiles_are_painted_from_the_planet_without_loaded_terrain() {
    let universe = Universe::new(7);
    let spec = universe.planet(universe.home());
    let dir = centre(leaf_of(
        DVec3::new(0.6, 0.3, -0.7).normalize(),
        depth_for(spec.radius),
    ));
    let mut app = painter(dir, 3.5);
    let lens = covered(&mut app);
    assert!(!app.world().contains_resource::<Terrain>());
    let (drawn, _) = app.world().resource::<Tiles>().ready(lens);
    let (_, id, handle) = drawn
        .iter()
        .find(|(_, id, _)| id.pixel(dir).is_some())
        .expect("a tile holds the explored leaf");
    let images = app.world().resource::<Assets<Image>>();
    let data = images
        .get(handle)
        .and_then(|image| image.data.as_ref())
        .expect("painted");
    let (px, py) = id.pixel(dir).expect("inside");
    let at = (py * SIDE + px) * BYTES;
    let globe = Globe::of(&spec);
    let wanted = colour(&globe, id.direction((px as f64 + 0.5, py as f64 + 0.5)));
    assert_eq!(data[at..at + BYTES], wanted);
    let far = ((py + SIDE / 2) % SIDE * SIDE + (px + SIDE / 2) % SIDE) * BYTES;
    assert_eq!(data[far..far + BYTES], UNSEEN);
}
