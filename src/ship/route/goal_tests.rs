//! Tests of the end of a route: the lock spheres of far destinations, the way home from another
//! system and another galaxy, and the stars at the centre of a galaxy.

use super::goal::{abroad, resolve};
use crate::computer::{Target, target_position};
use crate::ship::warp::{GALAXY_CAP, JUMP_CAP, Tier};
use crate::ship::wells::{PLANET_LOCK, STAR_LOCK};
use crate::space::Frame;
use crate::universe::{BodyId, GalaxyId, LY, Universe, system_reach};

/// Returns the first planet of the star the goal of `galaxy` is.
fn planet_of(universe: &Universe, galaxy: GalaxyId) -> BodyId {
    let Target::Star(star) = resolve(universe, Target::Galaxy(galaxy)).target else {
        panic!("no star at the centre of {galaxy:?}");
    };
    universe.system(star).planets[0].id
}

/// Returns the metres from the planet `from` to the home planet at time zero.
fn course(universe: &Universe, from: BodyId) -> f64 {
    let at = |body| target_position(universe, Target::Body(body), 0.0).unwrap();
    (at(universe.home()) - at(from)).length()
}

/// A galaxy is flown to as a star near its centre, found in the home galaxy and in the galaxies
/// around it, and well inside each of them.
#[test]
fn a_galaxy_goal_is_a_star_at_the_edge_of_its_void() {
    let universe = Universe::new(7);
    let others = universe.galaxies_near(universe.home().star.galaxy, 1);
    assert!(others.len() > 3);
    for galaxy in others {
        let id = galaxy.id;
        let Target::Star(star) = resolve(&universe, Target::Galaxy(id)).target else {
            panic!("no star at the centre of {id:?}");
        };
        let at = universe.star(star).unwrap().pos;
        assert!(at.length() < 0.2 * galaxy.radius, "{} ly", at.length() / LY);
    }
}

/// The way home is one route to the home planet from another system of the galaxy and from
/// another galaxy: it has a lock sphere and an arrival outside it, a finite course, and the tier
/// of its length, the jump to another galaxy for the other galaxy.
#[test]
fn home_is_one_route_from_another_system_and_another_galaxy() {
    let universe = Universe::new(7);
    let home = universe.home();
    let goal = resolve(&universe, Target::Body(home));
    assert_eq!(goal.target, Target::Body(home));
    let planet = universe.planet(home);
    assert!((goal.lock - planet.radius * PLANET_LOCK).abs() < 1.0 && goal.arrival > goal.lock);
    let at = universe.star(home.star).unwrap().pos;
    let near = universe.stars_near(home.star.galaxy, at, 12.0);
    let star = near.iter().find(|star| star.id != home.star).unwrap();
    let neighbour = universe.system(star.id).planets[0].id;
    let gap = course(&universe, neighbour);
    assert!(gap > 0.0 && gap < 13.0 * LY, "{} ly", gap / LY);
    assert!(!abroad(Frame::Surface(neighbour), Target::Body(home)));
    assert_eq!(Tier::of(gap, false), Tier::Cruise);
    let galaxies = universe.galaxies_near(home.star.galaxy, 1);
    let other = galaxies.iter().find(|galaxy| galaxy.id != home.star.galaxy);
    let origin = planet_of(&universe, other.unwrap().id);
    let far = course(&universe, origin);
    assert!(far > 1.0e5 * LY && far.is_finite(), "{} ly", far / LY);
    assert!(abroad(Frame::Surface(origin), Target::Body(home)));
    assert_eq!(Tier::of(far, true), Tier::Galaxy);
    assert_eq!(Tier::of(1.0e5 * LY, false), Tier::Jump);
    assert!(Tier::Jump.cap() == JUMP_CAP && Tier::Galaxy.cap() == GALAXY_CAP);
}

/// A route to a star never ends inside its lock sphere: it drops out beyond the lock, and the
/// ship enters the system of the star, at its reach, before it gets to the lock.
#[test]
fn a_route_ends_outside_the_lock_sphere_of_twenty_stars() {
    let universe = Universe::new(7);
    let home = universe.home();
    let at = universe.star(home.star).unwrap().pos;
    let stars = universe.stars_near(home.star.galaxy, at, 30.0);
    assert!(stars.len() >= 20, "{}", stars.len());
    for star in stars.iter().take(20) {
        let goal = resolve(&universe, Target::Star(star.id));
        assert!(goal.lock == star.radius * STAR_LOCK && goal.arrival >= goal.lock);
        assert!(system_reach(star) > goal.lock, "{:?}", star.id);
    }
}
