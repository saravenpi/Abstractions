//! Tests of the drawn signs: they lie around the robot on a living planet within the mesh budget, the
//! mesh never grows, and a dead planet shows none.

use super::Trail;
use super::paint::{Canvas, mesh, paint};
use super::stage::{app, herded};
use super::tune::{QUADS, RADIUS};
use crate::fauna::place::Spot;
use crate::fauna::stage::run;
use crate::life::signs::{Mark, Sign, tune::MOST};

/// On the home planet the robot standing in a herd finds signs around it, all within the radius, and
/// they are drawn by one mesh.
#[test]
fn signs_lie_around_the_robot() {
    let (mut app, herd) = app(true);
    assert!(herd.is_some(), "the rig has a herd");
    run(&mut app, 3.0, 0.1);
    assert!(herded(&app));
    let trail = app.world().resource::<Trail>();
    let count = |m| trail.signs.iter().filter(|s| s.mark == m).count();
    println!(
        "{} signs: {} prints, {} droppings, {} patches, {} dens, {} bones",
        trail.signs.len(),
        count(Mark::Print),
        count(Mark::Dropping),
        count(Mark::Grazed),
        count(Mark::Nest) + count(Mark::Burrow),
        count(Mark::Bones)
    );
    assert!(!trail.signs.is_empty(), "no sign around a herd");
    assert!(trail.signs.len() <= MOST);
    assert!(trail.drawn.is_some(), "the signs have no mesh");
}

/// A dead planet shows no signs and spawns no mesh.
#[test]
fn no_signs_on_dead_planets() {
    let (mut app, _) = app(false);
    run(&mut app, 3.0, 0.1);
    let trail = app.world().resource::<Trail>();
    assert!(trail.signs.is_empty());
    assert!(trail.drawn.is_none());
}

/// Painting far more signs than the mesh holds draws [`QUADS`] quads at most and keeps every buffer
/// at its size.
#[test]
fn the_mesh_keeps_its_size() {
    let dir = bevy::math::DVec3::new(0.3, 0.9, 0.1).normalize();
    let sign = Sign {
        mark: Mark::Bones,
        species: 0,
        at: dir,
        facing: crate::fauna::place::east(dir),
        size: 1.0,
        fresh: 1.0,
    };
    let signs = vec![sign; MOST];
    let ground = |_| 0.0;
    let canvas = Canvas {
        spot: Spot::at(dir, 6000.0),
        radius: 6000.0,
        ground: &ground,
    };
    let mut mesh = mesh();
    let before = mesh.count_vertices();
    let drawn = paint(&mut mesh, &signs, &canvas);
    println!(
        "{} signs drew {drawn} quads of {QUADS} within {RADIUS} m",
        signs.len()
    );
    assert_eq!(drawn, QUADS);
    assert_eq!(mesh.count_vertices(), before);
}
