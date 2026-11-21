//! The tier of a course across a galaxy: the galaxy tier holds while the ship is between the galaxies, even in
//! the frame of the galaxy of the destination, and the jump tier starts only inside the galaxy.

use super::goal::{between_galaxies, resolve};
use crate::computer::Target;
use crate::ship::warp::Tier;
use crate::space::Frame;
use crate::universe::Universe;

/// A ship in the frame of the target galaxy, 2e22 m from a star of it, which is the width of a grid cell
/// and far outside the galaxy, flies the galaxy tier; a ship inside the galaxy flies the jump tier.
#[test]
fn the_galaxy_tier_holds_until_the_ship_is_inside_the_galaxy() {
    let universe = Universe::new(7);
    let home = universe.home().star.galaxy;
    let others = universe.galaxies_near(home, 1);
    let other = others
        .iter()
        .find(|galaxy| galaxy.id != home)
        .expect("another galaxy");
    let Target::Star(star) = resolve(&universe, Target::Galaxy(other.id)).target else {
        panic!("no star at the centre of the galaxy");
    };
    let (frame, target) = (Frame::Galaxy(other.id), Target::Star(star));
    let far = 2.0e22;
    assert!(between_galaxies(&universe, frame, (target, far)));
    assert_eq!(
        Tier::of(far, between_galaxies(&universe, frame, (target, far))),
        Tier::Galaxy
    );
    let inside = other.radius;
    assert!(!between_galaxies(&universe, frame, (target, inside)));
    assert_eq!(Tier::of(inside, false), Tier::Jump);
    let from_home = Frame::Galaxy(home);
    assert!(between_galaxies(&universe, from_home, (target, inside)));
}
