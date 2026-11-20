//! The reach of a system: where a ship between the stars enters it and leaves it.

use bevy::math::{DQuat, DVec3};

use crate::ship::body::Ship;
use crate::ship::transit::{enter_system, leave_system};
use crate::space::Frame;
use crate::universe::{Universe, system_reach};

/// Returns the star of the home planet of seed 7.
fn home_star() -> crate::universe::StarSpec {
    let universe = Universe::new(7);
    universe.star(universe.home().star).unwrap()
}

/// A ship 0.9 reaches of a system from its star between the stars enters the system, and at 1.1
/// reaches it does not, whatever the star: the reach is the one of `system_reach`, the single
/// definition of the size of a system.
#[test]
fn a_ship_enters_a_system_inside_its_reach_only() {
    let star = home_star();
    let at = |reaches: f64| {
        let pos = star.pos + DVec3::X * system_reach(&star) * reaches;
        let mut ship = Ship::at_rest(Frame::Galaxy(star.id.galaxy), pos, DQuat::IDENTITY);
        enter_system(&mut ship, [&star]);
        ship.frame
    };
    assert_eq!(at(0.9), Frame::System(star.id));
    assert_eq!(at(1.1), Frame::Galaxy(star.id.galaxy));
}

/// A ship leaves a system at 1.05 reaches and not before, so it does not change frame back and
/// forth at the edge: entering at the reach and leaving beyond it leave a margin much wider
/// than the step of a position in the frame of a galaxy, 1e5 m at 5e20 m.
#[test]
fn a_ship_leaves_a_system_past_its_reach_with_a_margin() {
    let star = home_star();
    let reach = system_reach(&star);
    let at = |reaches: f64| {
        let mut ship = Ship::at_rest(
            Frame::System(star.id),
            DVec3::X * reach * reaches,
            DQuat::IDENTITY,
        );
        leave_system(&mut ship, &star);
        ship.frame
    };
    assert_eq!(at(1.0), Frame::System(star.id));
    assert_eq!(at(1.1), Frame::Galaxy(star.id.galaxy));
    assert!(0.05 * reach > 1.0e5 * 1.0e3);
}
