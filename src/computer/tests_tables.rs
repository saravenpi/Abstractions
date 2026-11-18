//! Computer tests: distance formatting, the tables, their navigation and where the ship is.

use bevy::math::DVec3;

use super::*;
use crate::space::Frame;
use crate::universe::{GalaxyId, Universe};

/// Distances switch unit at 1000 of the smaller one and keep three digits.
#[test]
fn distances_cover_each_unit() {
    let cases = [
        (-5.0, "0 m"),
        (0.4, "0 m"),
        (999.4, "999 m"),
        (999.5, "1.00 km"),
        (1500.0, "1.50 km"),
        (12_345.0, "12.3 km"),
        (999_499.0, "999 km"),
        (999_500.0, "1.00 Mm"),
        (4.0e7, "40.0 Mm"),
        (9.994e8, "999 Mm"),
        (9.995e8, "0.10 gly"),
        (1.0e10, "1.00 gly"),
        (6.0e11, "60.0 gly"),
        (1.5e13, "1500 gly"),
    ];
    for (metres, shown) in cases {
        assert_eq!(distance(metres), shown, "{metres}");
    }
    assert_eq!(gravity(crate::universe::HOME_GRAVITY), "1.00 g");
}

/// Returns the finished chart of the systems `atlas` reveals.
fn charted(universe: &Universe, atlas: &Atlas) -> Known {
    let mut known = Known::default();
    while !known.complete(atlas) {
        known.absorb(universe, atlas);
    }
    known
}

/// The galaxy table lists exactly the charted systems, nearest first, and hides the planets of
/// systems that were not visited.
#[test]
fn the_galaxy_table_lists_only_known_systems() {
    let universe = Universe::new(7);
    let home = universe.home().star;
    let atlas = Atlas::new(&universe);
    let known = charted(&universe, &atlas);
    let expected = atlas.known_systems(&universe, home.galaxy);
    assert_eq!(known.stars().count(), expected.len());
    let ship = universe.star_position(home).unwrap();
    let (universe, atlas) = (&universe, &atlas);
    let view = View {
        universe,
        atlas,
        ship,
        time: 0.0,
    };
    let stars = view.galaxy(home.galaxy, known.stars());
    assert_eq!(stars.len(), expected.len());
    assert_eq!(stars[0].target, Target::Star(home));
    assert!(
        stars
            .windows(2)
            .all(|pair| pair[0].distance <= pair[1].distance)
    );
    assert!(stars[0].visited && stars[0].detail.ends_with("planets - visited"));
    let unvisited = |line: &Line| line.detail.ends_with("planets unknown");
    assert!(stars[1..].iter().all(unvisited));
    let elsewhere = GalaxyId(bevy::math::IVec3::ONE);
    assert!(view.galaxy(elsewhere, known.stars()).is_empty());
}

/// The system table lists the star and its planets, and the universe table the known galaxies.
#[test]
fn the_system_and_universe_tables_follow_the_atlas() {
    let universe = Universe::new(7);
    let home = universe.home();
    let atlas = Atlas::new(&universe);
    let ship = universe.star_position(home.star).unwrap();
    let (universe, atlas) = (&universe, &atlas);
    let view = View {
        universe,
        atlas,
        ship,
        time: 0.0,
    };
    let system = view.system(home.star);
    let planets = universe.system(home.star).planets;
    let moons: usize = planets.iter().map(|planet| usize::from(planet.moons)).sum();
    assert_eq!(system.len(), 1 + planets.len() + moons);
    let visited = system.iter().filter(|line| line.visited);
    let visited: Vec<Target> = visited.map(|line| line.target).collect();
    assert_eq!(visited, [Target::Star(home.star), Target::Body(home)]);
    assert!(system[1].detail.contains(" C - radius ") && system[1].detail.contains(" g"));
    let galaxies = view.universe();
    assert_eq!(galaxies.len(), 1 + HOME_NEIGHBOURS);
    assert_eq!(galaxies[0].target, Target::Galaxy(home.star.galaxy));
    assert!(
        galaxies
            .windows(2)
            .all(|pair| pair[0].distance <= pair[1].distance)
    );
}

/// A table of 500 rows keeps the cursor drawn while stepping, paging, jumping and scrolling.
#[test]
fn a_long_table_keeps_the_cursor_in_view() {
    let line = |i: usize| Line {
        title: format!("Star {i}"),
        detail: String::new(),
        target: Target::Galaxy(GalaxyId::default()),
        distance: i as f64,
        visited: false,
        at: default(),
    };
    let mut board = Board::default();
    board.show(Tab::Galaxy);
    board.fill("Home galaxy".to_string(), (0..500).map(line).collect());
    for by in [1, 1, 30, -7, 1_000_000, -3, -1_000_000, 499, 12, -12, -1] {
        board.step(by);
        assert!((board.top..board.top + VISIBLE_ROWS).contains(&board.cursor));
        assert!(board.top + VISIBLE_ROWS <= 500);
    }
    assert_eq!((board.cursor, board.top), (486, 486));
    board.scroll(-100);
    assert_eq!((board.cursor, board.top), (397, 386));
    board.scroll(1000);
    assert_eq!((board.cursor, board.top), (488, 488));
    board.pick(3);
    assert_eq!(board.at(3).unwrap().title, "Star 491");
    assert_eq!(board.cursor, 491);
    board.fill(String::new(), (0..5).map(line).collect());
    assert!((board.cursor, board.top) == (4, 0) && board.at(5).is_none());
    assert_eq!(
        (Tab::System.step(-1), Tab::Universe.step(1)),
        (Tab::Universe, Tab::System)
    );
}

/// The frame says what the ship is inside: a planet, a system within reach, a galaxy within its disc.
#[test]
fn whereabouts_follow_the_frame() {
    let universe = Universe::new(7);
    let home = universe.home();
    let (star, galaxy) = (home.star, home.star.galaxy);
    let on_planet = whereabouts(&universe, Frame::Surface(home), DVec3::ZERO);
    assert_eq!(on_planet, (Some(home), Some(star), Some(galaxy)));
    let near = whereabouts(
        &universe,
        Frame::System(star),
        DVec3::X * 0.5 * SYSTEM_REACH,
    );
    assert_eq!(near, (None, Some(star), Some(galaxy)));
    let far = whereabouts(
        &universe,
        Frame::System(star),
        DVec3::X * 2.0 * SYSTEM_REACH,
    );
    assert_eq!(far, (None, None, Some(galaxy)));
    let radius = universe.galaxy(galaxy).unwrap().radius;
    let inside = whereabouts(&universe, Frame::Galaxy(galaxy), DVec3::Z * 0.5 * radius);
    assert_eq!(inside, (None, None, Some(galaxy)));
    let outside = whereabouts(&universe, Frame::Galaxy(galaxy), DVec3::Z * 2.0 * radius);
    assert_eq!(outside, (None, None, None));
    let ship = ship_position(&universe, Frame::System(star), DVec3::X * 5.0e6, 0.0).unwrap();
    let sun = target_position(&universe, Target::Star(star), 0.0).unwrap();
    assert!((ship.distance(sun) - 5.0e6).abs() < 1.0);
}
