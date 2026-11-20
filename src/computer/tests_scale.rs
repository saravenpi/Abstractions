//! Computer tests: the charts at real scale. A system chart gives every orbit room, a
//! neighbourhood of stars fills the galaxy chart, a visit reveals a few stars, and a star
//! is found by name in the rows without reading the galaxy.

use bevy::math::DVec3;

use super::reveal::systems_around;
use super::scale::Radial;
use super::sift::arrange;
use super::*;
use crate::space::Frame;
use crate::universe::{AU, LY, Universe, system_reach};

/// The log scale puts the star at the middle, the farthest row at the edge, never goes
/// backward, and gives the innermost orbit of the Sun about a tenth of the chart radius where
/// the square root of the distance gave under three hundredths.
#[test]
fn the_radius_grows_with_the_log_of_the_distance() {
    let radial = Radial {
        unit: (0.03 * AU) as f32,
        reach: (40.0 * AU) as f32,
    };
    assert_eq!(radial.share(0.0), 0.0);
    assert!((radial.share(radial.reach) - 1.0).abs() < 1e-6);
    assert_eq!(radial.share(10.0 * radial.reach), 1.0);
    let inner = radial.share(radial.unit);
    assert!(
        inner > 0.09 && inner > 3.0 * (0.03f32 / 40.0).sqrt(),
        "{inner}"
    );
    let earth = radial.share(AU as f32);
    assert!(earth > 0.4 && earth < 0.6, "{earth}");
    let mut metres = 1.0e6;
    let mut before = 0.0;
    while metres < 1.0e14 {
        let now = radial.share(metres);
        assert!(now >= before);
        (metres, before) = (metres * 1.5, now);
    }
}

/// The orbits of the planets of real systems take distinct radii on the chart: the innermost
/// is off the star, and two neighbours are apart.
#[test]
fn every_orbit_of_a_system_has_room_on_the_chart() {
    for seed in [7, 11, 23] {
        let universe = Universe::new(seed);
        let atlas = Atlas::new(&universe);
        let star = universe.home().star;
        let ship = universe.star_position(star).unwrap();
        let view = View {
            universe: &universe,
            atlas: &atlas,
            ship,
            time: 0.0,
        };
        let lines = view.system(star);
        let planet = |row: usize| {
            let target = lines[row].target;
            matches!(target, Target::Body(_)) && !is_moon(target)
        };
        let dots = chart(Tab::System, &lines, ship);
        let radii: Vec<f32> = dots
            .iter()
            .filter_map(|(dot, at)| match dot {
                Dot::Row(row) if planet(*row) => Some(at.length()),
                _ => None,
            })
            .collect();
        assert!(radii.is_empty() || radii[0] > 0.05, "{seed}: {radii:?}");
        let apart = radii.windows(2).all(|pair| pair[1] - pair[0] > 0.01);
        assert!(apart, "{seed}: {radii:?}");
    }
}

/// Returns the rows of the galaxy chart of a new game of `universe`, with the ship at the home
/// star.
fn neighbourhood(universe: &Universe) -> (Vec<Line>, crate::space::UniPos) {
    let atlas = Atlas::new(universe);
    let galaxy = universe.home().star.galaxy;
    let ship = universe.star_position(universe.home().star).unwrap();
    let known = atlas.known_systems(universe, galaxy);
    let view = View {
        universe,
        atlas: &atlas,
        ship,
        time: 0.0,
    };
    (
        view.galaxy(galaxy, known.into_iter().map(|star| star.id)),
        ship,
    )
}

/// The galaxy chart of a new game is the neighbourhood of the home star, drawn to scale: it
/// reaches 10 light-years, so a chart unit is about 11 light-years, and a star is found by its
/// name in the rows, which a filter reads one by one and never a galaxy.
#[test]
fn the_galaxy_chart_is_a_neighbourhood_to_scale() {
    let (lines, ship) = neighbourhood(&Universe::new(7));
    assert!(
        lines.len() >= 2 && lines.len() < 100,
        "{} rows",
        lines.len()
    );
    let unit = span(Tab::Galaxy, &lines, ship).unwrap();
    assert!(
        unit > 0.0 && unit <= 1.001 * REVEAL_LY * LY / f64::from(FILL),
        "{unit}"
    );
    assert!(
        lines
            .windows(2)
            .all(|pair| pair[0].distance <= pair[1].distance)
    );
    let dots = chart(Tab::Galaxy, &lines, ship);
    assert!(dots.iter().all(|(_, at)| at.length() <= 1.0 + 1e-6));
    let last = lines.last().unwrap().title.to_lowercase();
    let found = arrange(lines.clone(), &last, Sort::Near);
    assert!(found.iter().any(|line| line.title.to_lowercase() == last));
}

/// A visit reveals between 5 and 60 systems for nearly every seed, nearest first.
#[test]
fn a_visit_reveals_a_neighbourhood_of_stars() {
    let mut good = 0;
    for seed in 1..=50 {
        let universe = Universe::new(seed);
        let home = universe.home().star;
        let origin = universe.star(home).unwrap().pos;
        let around = systems_around(&universe, home);
        let gap = |star: &crate::universe::StarSpec| star.pos.distance(origin);
        let sorted = around.windows(2).all(|pair| gap(&pair[0]) <= gap(&pair[1]));
        good += usize::from(sorted && (5..=60).contains(&around.len()));
    }
    assert!(good >= 48, "{good} of 50 seeds");
}

/// A ship inside a system is inside it up to the reach every part of the game shares, and the
/// reach it read is kept for the next frame.
#[test]
fn the_reach_of_a_system_is_read_once() {
    let universe = Universe::new(7);
    let star = universe.home().star;
    let reach = system_reach(&universe.star(star).unwrap());
    let mut memo = None;
    for (share, inside) in [(0.5, true), (1.5, false), (0.9, true)] {
        let at = DVec3::Y * share * reach;
        let found = locate::whereabouts_with(&universe, Frame::System(star), at, &mut memo);
        assert_eq!(found.1.is_some(), inside, "{share}");
        assert_eq!(memo, Some((star, reach)));
    }
}
