//! Tests of the body pictures: the cost of a pixel, the layout, and a headless paint.

use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use super::grid::{COLS, ROWS, cell_of, centre};
use super::work::offset;
use super::{BYTES, Rasters, UNSEEN, colour};
use crate::computer::{Computer, Lens, SIZE, direction};
use crate::map::explored::Explored;
use crate::map::{from_chart, place_words, to_chart};
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

/// Returns a headless app that paints the pictures of a world of seed 7 where `seen` is the
/// one explored cell of the home planet, the map in state `map`.
fn painter(seen: u32, map: Computer) -> App {
    let universe = Universe::new(7);
    let body = universe.home();
    let mut explored = Explored::default();
    let radius = universe.planet(body).radius;
    explored.reveal(body, centre(seen), 1e-7, radius);
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin))
        .init_asset::<Image>()
        .init_state::<Screen>()
        .insert_state(map)
        .insert_resource(universe)
        .insert_resource(explored)
        .add_plugins(super::plugin);
    app.update();
    app.world_mut().resource_mut::<Rasters>().request(body);
    app
}

/// A requested picture paints the explored cells on a worker and leaves the rest clear, so the
/// dim tone of the map shows where nobody has been.
#[test]
fn explored_cells_are_painted_and_the_rest_stays_dark() {
    let seen = (COLS * 100 + 7) as u32;
    let mut app = painter(seen, Computer::Open);
    let painted = |world: &World| pixel(world, seen).is_some_and(|pixel| pixel[..3] != [0; 3]);
    wait(&mut app, painted);
    let dark = pixel(app.world(), seen + 1).expect("the picture exists");
    assert_eq!(dark, UNSEEN);
    let far = pixel(app.world(), (COLS * 100) as u32).expect("the picture exists");
    assert_eq!(far, UNSEEN);
}

/// Nothing is painted while the map is closed; opening it paints what was explored, and the
/// picture stays painted when it closes again.
#[test]
fn a_closed_map_paints_nothing() {
    let seen = (COLS * 100 + 7) as u32;
    let mut app = painter(seen, Computer::Closed);
    for _ in 0..50 {
        app.update();
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(pixel(app.world(), seen), None);
    let set = |app: &mut App, map| {
        app.world_mut()
            .resource_mut::<NextState<Computer>>()
            .set(map)
    };
    set(&mut app, Computer::Open);
    let painted = |world: &World| pixel(world, seen).is_some_and(|pixel| pixel[..3] != [0; 3]);
    wait(&mut app, painted);
    set(&mut app, Computer::Closed);
    app.update();
    assert!(painted(app.world()));
}

/// The pixel of the picture under the pointer on the planet level is the cell of the place the
/// map reads there and says in words. The picture fills the middle half of the sheet, as
/// `planet_layer` lays it out.
#[test]
fn the_picture_lies_under_the_pointer_as_the_map_reads_it() {
    let lens = Lens::default();
    let spots = [(12.4, 103.2), (-60.0, 300.0), (0.3, 0.3), (80.0, 200.0)];
    for (lat, lon) in spots {
        let dir = direction(f64::to_radians(lon), f64::to_radians(lat));
        let pixel = lens.to_map(to_chart(dir));
        let at = from_chart(lens.to_chart(pixel)).expect("on the map");
        assert_eq!(cell_of(at), cell_of(dir), "{lat} {lon}");
        let across = (pixel.x / SIZE * COLS as f32) as usize;
        let down = ((pixel.y / SIZE - 0.25) * 2.0 * ROWS as f32) as usize;
        assert_eq!(
            (down * COLS + across) * BYTES,
            offset(cell_of(at)),
            "{lat} {lon}"
        );
    }
    let said = place_words(direction(f64::to_radians(103.2), f64::to_radians(12.4)));
    assert_eq!(said, "12.4° N  103.2° E");
}
