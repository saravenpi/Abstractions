//! The space view as data: the lens that keeps the nearest body and the destination in sight,
//! and the chart of a real system.

use bevy::math::{DVec3, Vec2};

use crate::computer::{Atlas, Dot, Lens, SIZE, Target, View, ship_position};
use crate::menus::hud::minimap::orbit::{CLOSEST, Scene, lens, scene, signature};
use crate::space::Frame;
use crate::universe::{BodyId, Universe};

/// Returns true when the lens draws the chart point `at` inside the map, clear of its edge.
fn in_sight(lens: Lens, at: Vec2) -> bool {
    let pixel = lens.to_map(at);
    pixel.min_element() > 0.05 * SIZE && pixel.max_element() < 0.95 * SIZE
}

/// Next to a planet the lens closes in as far as the minimap allows, with the planet and the
/// ship in sight. A destination across the system opens it until both ends fit.
#[test]
fn the_lens_fits_the_nearest_body_and_the_destination() {
    let (near, far, ship) = (
        Vec2::new(0.3, 0.0),
        Vec2::new(-0.8, 0.1),
        Vec2::new(0.32, 0.01),
    );
    let dots = [
        (Dot::Row(0), Vec2::ZERO),
        (Dot::Orbit, Vec2::new(0.0, 0.3)),
        (Dot::Row(1), near),
        (Dot::Row(2), far),
        (Dot::Ship, ship),
    ];
    let close = lens(&dots, None);
    assert!((close.zoom - CLOSEST).abs() < 1e-3, "{close:?}");
    assert!(in_sight(close, near) && in_sight(close, ship));
    assert!(!in_sight(close, far));
    let wide = lens(&dots, Some(2));
    assert!(wide.zoom >= 1.0 && wide.zoom < 1.5, "{wide:?}");
    assert!(in_sight(wide, near) && in_sight(wide, ship) && in_sight(wide, far));
    assert_eq!(wide, wide.clamped());
    assert_eq!(lens(&dots, Some(1)), close);
    assert_eq!(lens(&[], None), lens(&[(Dot::Ship, Vec2::ZERO)], None));
}

/// Returns the chart of the home system for a ship `above` metres over the home planet, with
/// what `atlas` knows and the destination `chosen`.
fn charted(universe: &Universe, atlas: &Atlas, above: f64, chosen: Option<Target>) -> Scene {
    let home = universe.home();
    let at = universe.planet_position(home, 0.0) + DVec3::X * above;
    let view = View {
        universe,
        atlas,
        ship: ship_position(universe, Frame::System(home.star), at, 0.0).unwrap(),
        time: 0.0,
    };
    scene(&view, home.star, chosen)
}

/// The chart of the home system has its star and every planet, the classes of a system that
/// was visited and none of one that was not, and the ship beside the home planet.
#[test]
fn the_chart_shows_the_system_around_the_ship() {
    let universe = Universe::new(7);
    let home = universe.home();
    let planets = universe.system(home.star).planets.len();
    let plain = charted(&universe, &Atlas::new(&universe), 5.0e4, None);
    assert_eq!(plain.lines.len(), planets + 1);
    assert_eq!(plain.lines[0].target, Target::Star(home.star));
    assert!(plain.worlds[0].is_none() && plain.worlds[1..].iter().all(Option::is_some));
    let row = Dot::Row(usize::from(home.index) + 1);
    let (ship, planet) = (plain.place(Dot::Ship).unwrap(), plain.place(row).unwrap());
    assert!(ship.distance(planet) < 0.02 && plain.chosen.is_none());
    assert!(in_sight(plain.lens, ship) && in_sight(plain.lens, planet));
    let unknown = charted(&universe, &Atlas::default(), 5.0e4, None);
    assert!(unknown.worlds.iter().all(Option::is_none));
    assert_eq!(signature(&plain), signature(&unknown));
}

/// A destination in the system is a row of the chart, in sight with the ship, and changes
/// what is drawn.
#[test]
fn the_chart_keeps_the_destination_in_sight() {
    let universe = Universe::new(7);
    let (home, atlas) = (universe.home(), Atlas::new(&universe));
    let planets = universe.system(home.star).planets.len() as u8;
    let index = (0..planets).find(|index| *index != home.index).unwrap();
    let target = Target::Body(BodyId { index, ..home });
    let bound = charted(&universe, &atlas, 5.0e4, Some(target));
    let row = usize::from(index) + 1;
    let (goal, ship) = (bound.place(Dot::Row(row)), bound.place(Dot::Ship));
    assert_eq!(bound.chosen, Some(row));
    assert!(in_sight(bound.lens, goal.unwrap()) && in_sight(bound.lens, ship.unwrap()));
    let plain = charted(&universe, &atlas, 5.0e4, None);
    assert_ne!(signature(&bound), signature(&plain));
}
