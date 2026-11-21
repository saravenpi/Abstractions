//! Tests of what the mesh holds against what the sky shows: every star in sight is in the mesh
//! until the next build, and a grain of dust is never as bright or as wide as a star.

use std::time::{Duration, Instant};

use bevy::math::DVec3;

use super::build::gather;
use super::specks::{own_light, shine, star_light};
use super::{MOST_STARS, REACH_LY};
use crate::render::scatter::luminance;
use crate::render::warp::{GRAIN, GRAIN_LIGHT, SIGHT, daylit};
use crate::universe::{LY, SECTOR_LY, StarSpec, Universe, system_reach};

/// The mesh is rebuilt once the camera has moved a sector from where it was built for, and the
/// build takes a moment at the one light-year a second of the cruise. The stars are gathered farther
/// than they are drawn by more than both, so no star in sight is ever missing from the mesh.
#[test]
fn the_mesh_reaches_past_the_edge_of_sight() {
    let spare = REACH_LY - f64::from(SIGHT.0) - SECTOR_LY;
    assert!(
        spare >= 5.0,
        "{spare} light-years to build the next mesh in"
    );
    let universe = Universe::new(7);
    let star = universe.system(universe.home().star).star;
    let seen = gather(&universe, star.id.galaxy, star.pos, None);
    let within = universe
        .stars_near(star.id.galaxy, star.pos, REACH_LY)
        .len();
    assert!(
        within <= MOST_STARS,
        "{within} stars within reach are cut short"
    );
    let stars = seen.len();
    assert_eq!(stars, within, "every star within reach, the own one too");
}

/// Seen from home, the mesh holds the stars of a sphere 55 light-years wide at the real density,
/// 0.004 stars per cubic light-year: 2,800 in the plain disc, more in an arm, and all of them
/// fit under the most the mesh takes.
#[test]
fn the_home_sky_holds_the_real_number_of_stars() {
    let universe = Universe::new(7);
    let star = universe.system(universe.home().star).star;
    let seen = gather(&universe, star.id.galaxy, star.pos, None);
    assert!(
        (1_500..=MOST_STARS).contains(&seen.len()),
        "{} stars in sight",
        seen.len()
    );
}

/// Gathering the stars of the home sky reads about 2,200 sectors and takes under 100 ms, so the
/// task that does it is short next to the ten seconds a cruise takes to leave a sector.
#[test]
fn gathering_the_home_sky_takes_under_100_ms() {
    let universe = Universe::new(7);
    let star = universe.system(universe.home().star).star;
    let start = Instant::now();
    let seen = gather(&universe, star.id.galaxy, star.pos, None);
    let took = start.elapsed();
    assert!(!seen.is_empty());
    assert!(
        took < Duration::from_millis(100),
        "the gather took {took:?}"
    );
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
    let seen = gather(&universe, star.id.galaxy, star.pos, None);
    let dimmest = seen
        .iter()
        .map(|speck| luminance(speck.color))
        .fold(f32::MAX, f32::min);
    assert!(
        GRAIN_LIGHT * 3.0 <= dimmest,
        "{GRAIN_LIGHT} against {dimmest}"
    );
}

/// The star of the system the camera is in is a point of the star field like the others, and
/// the only one that shows through the day: from its planets, as far out as the outermost one, it
/// is the sun. From the edge of its system it is its plain light, which at the real scale is
/// still far above the day: the Sun at 320 AU is brighter than the full moon. Like every other
/// star, the day sky hides it from a distance of the square root of its luminosity in light-years.
#[test]
fn the_own_star_is_the_one_point_the_day_does_not_hide() {
    let universe = Universe::new(7);
    let home = universe.planet(universe.home());
    let star = universe.system(home.id.star).star;
    let from = star.pos + DVec3::X * home.orbit_radius;
    assert_eq!(daylit(own_light(&star, from).1), 1.0);
    let system = universe.system(home.id.star);
    let outer = system.planets.last().unwrap().orbit_radius;
    assert_eq!(daylit(own_light(&star, star.pos + DVec3::X * outer).1), 1.0);
    let others = universe.stars_near(star.id.galaxy, from, 40.0);
    let hidden = |other: &&StarSpec| daylit(star_light(other, from).1) == 0.0;
    let rest = others.iter().filter(|other| other.id != star.id);
    assert_eq!(rest.clone().filter(hidden).count(), rest.count());
    let edge = star.pos + DVec3::X * 2.0 * system_reach(&star);
    let (_, plain) = star_light(&star, edge);
    assert_eq!(own_light(&star, edge).1, plain);
    assert_eq!(daylit(plain), 1.0);
    let hidden_from = star.pos + DVec3::X * star.luminosity.sqrt().max(1.0) * LY;
    assert_eq!(daylit(own_light(&star, hidden_from).1), 0.0);
}
