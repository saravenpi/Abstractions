//! Tests of what the mesh holds against what the sky shows: every star in sight is in the mesh
//! until the next build, and a grain of dust is never as bright or as wide as a star.

use super::build::gather;
use super::specks::shine;
use super::specks::star_light;
use super::{MOST_STARS, REACH_GLY};
use crate::render::scatter::luminance;
use crate::render::warp::{GRAIN, GRAIN_LIGHT, SIGHT, daylit};
use crate::universe::{GLY, SECTOR_GLY, Universe};

/// The mesh is rebuilt once the camera has moved a sector from where it was built for, and the
/// build takes a moment at up to five light-years a second. The stars are gathered farther
/// than they are drawn by more than both, so no star in sight is ever missing from the mesh.
#[test]
fn the_mesh_reaches_past_the_edge_of_sight() {
    let spare = REACH_GLY - f64::from(SIGHT.0) - SECTOR_GLY;
    assert!(
        spare >= 5.0,
        "{spare} light-years to build the next mesh in"
    );
    let universe = Universe::new(7);
    let star = universe.system(universe.home().star).star;
    let seen = gather(&universe, star.id.galaxy, star.pos);
    let within = universe
        .stars_near(star.id.galaxy, star.pos, REACH_GLY)
        .len();
    assert!(
        within <= MOST_STARS,
        "{within} stars within reach are cut short"
    );
    let stars = seen.iter().filter(|speck| speck.flux < 50.0).count();
    assert_eq!(stars, within, "every star within reach, the own one too");
}

/// A grain at its brightest shows under a third of the light of the faintest star the sky
/// draws, and it is thinner than the smallest star: nobody takes it for a star.
#[test]
fn a_grain_is_fainter_and_thinner_than_any_star() {
    let (smallest, faintest) = shine(-100.0);
    assert!(GRAIN < 0.6 * smallest, "{GRAIN} against {smallest}");
    assert!(
        GRAIN_LIGHT * 3.0 <= faintest,
        "{GRAIN_LIGHT} against {faintest}"
    );
    let universe = Universe::new(7);
    let star = universe.system(universe.home().star).star;
    let seen = gather(&universe, star.id.galaxy, star.pos);
    let stars = seen.iter().filter(|speck| speck.flux < 50.0);
    let dimmest = stars
        .map(|speck| luminance(speck.color))
        .fold(f32::MAX, f32::min);
    assert!(
        GRAIN_LIGHT * 3.0 <= dimmest,
        "{GRAIN_LIGHT} against {dimmest}"
    );
}

/// The star of the system the camera is in is a point of the star field like the others, and
/// the only one that shows through the day: from its planets it is the sun, and from a
/// light-year away, like every other star, the day sky hides it.
#[test]
fn the_own_star_is_the_one_point_the_day_does_not_hide() {
    let universe = Universe::new(7);
    let home = universe.planet(universe.home());
    let star = universe.system(home.id.star).star;
    let from = star.pos + bevy::math::DVec3::X * home.orbit_radius;
    assert_eq!(daylit(star_light(&star, from).1), 1.0);
    let system = universe.system(home.id.star);
    let outer = system.planets.last().unwrap().orbit_radius;
    assert_eq!(
        daylit(star_light(&star, star.pos + bevy::math::DVec3::X * outer).1),
        1.0
    );
    let others = universe.stars_near(star.id.galaxy, from, 40.0);
    let hidden = |other: &&crate::universe::StarSpec| daylit(star_light(other, from).1) == 0.0;
    assert_eq!(others.iter().filter(hidden).count(), others.len() - 1);
    assert_eq!(
        daylit(star_light(&star, star.pos + bevy::math::DVec3::X * GLY).1),
        0.0
    );
}
