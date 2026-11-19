//! Tests of the body pictures: the cost of a pixel, the layout, and a headless paint.

use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use super::work::offset;
use super::{BYTES, LINE, Rasters, colour};
use crate::map::explored::Explored;
use crate::map::explored::grid::{COLS, ROWS, cell_of, centre};
use crate::screens::Screen;
use crate::universe::Universe;
use crate::world::Globe;

/// Most ground samples one pixel may take.
const PER_PIXEL: u64 = 8;

/// A pixel of the home planet reads the bare land only, a few ground samples and no grid.
#[test]
fn a_pixel_is_cheap() {
    let universe = Universe::new(7);
    let globe = Globe::of(&universe.planet(universe.home()));
    let cells: Vec<u32> = (0..64).map(|i| i * 2039 % (COLS * ROWS) as u32).collect();
    let before = Globe::sampled();
    let pixels: Vec<[u8; BYTES]> = cells
        .iter()
        .map(|&cell| colour(&globe, centre(cell)))
        .collect();
    let taken = Globe::sampled() - before;
    println!("64 map pixels: {taken} ground samples");
    assert!(taken <= PER_PIXEL * cells.len() as u64, "{taken} samples");
    assert!(pixels.iter().all(|pixel| pixel[3] == u8::MAX));
}

/// The north pole is at the top of the picture and longitude 0 at its left edge.
#[test]
fn the_north_is_at_the_top() {
    assert_eq!(offset(cell_of(Vec3::Y.as_dvec3())), 0);
    let south = cell_of(Vec3::new(1e-6, -1.0, 0.0).normalize().as_dvec3());
    assert_eq!(offset(south), (ROWS - 1) * COLS * BYTES);
    assert_eq!(
        offset(cell_of(Vec3::new(1.0, 0.0, -0.001).as_dvec3())) % (COLS * BYTES),
        0
    );
}

/// Runs frames until `done` holds. Fails after a minute.
fn wait(app: &mut App, done: impl Fn(&World) -> bool) {
    let start = Instant::now();
    while !done(app.world()) {
        assert!(start.elapsed() < Duration::from_secs(60), "timed out");
        app.update();
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Returns the pixel of `cell` in the picture of the home planet, once it exists.
fn pixel(world: &World, cell: u32) -> Option<[u8; BYTES]> {
    let universe = world.resource::<Universe>();
    let handle = world.resource::<Rasters>().image(universe.home())?;
    let data = world
        .resource::<Assets<Image>>()
        .get(&handle)?
        .data
        .as_ref()?;
    let at = offset(cell);
    data[at..at + BYTES].try_into().ok()
}

/// A requested picture paints the explored cells on a worker and leaves the rest dark.
#[test]
fn explored_cells_are_painted_and_the_rest_stays_dark() {
    let universe = Universe::new(7);
    let body = universe.home();
    let mut explored = Explored::default();
    let seen = (COLS * 100 + 7) as u32;
    explored.reveal_cells(body, &[seen]);
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin))
        .init_asset::<Image>()
        .init_state::<Screen>()
        .insert_resource(universe)
        .insert_resource(explored)
        .add_plugins(super::plugin);
    app.update();
    app.world_mut().resource_mut::<Rasters>().request(body);
    let painted = |world: &World| pixel(world, seen).is_some_and(|pixel| pixel[..3] != [0; 3]);
    wait(&mut app, painted);
    let dark = pixel(app.world(), seen + 1).expect("the picture exists");
    assert_eq!(dark, [0, 0, 0, u8::MAX]);
    let line = pixel(app.world(), (COLS * 100) as u32).expect("the picture exists");
    assert_eq!(line, [LINE, LINE, LINE, u8::MAX]);
}
