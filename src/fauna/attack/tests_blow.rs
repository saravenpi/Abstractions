//! Tests of what a blow of an animal leaves in the robot: the robot is a free body that is thrown,
//! a charge is as fast as its run allows, and a trample lands with the whole body.

use super::force::{contact, joules, meets};
use super::kinds::Kind;
use crate::condition::{Condition, Part};
use crate::fauna::senses::tune::ROBOT_KG;

/// Gravity of the tests, in metres per second squared.
const G: f32 = 9.81;

/// Returns the energy of a charge of a horned body of `mass` kilograms and top speed `top` after a
/// run of `run` metres, and the speed it throws the robot at.
fn charge(mass: f32, top: f32, run: f32) -> (f32, f32) {
    let speed = contact(Kind::Charge, (top, run), (1.5, G));
    (
        joules(Kind::Charge, mass, speed),
        meets(Kind::Charge, mass) * speed / ROBOT_KG,
    )
}

/// A charge is slow after a short run, grows as the square root of the run and never passes the top
/// speed of the body; a trample with no run still drops the reared body on its target.
#[test]
fn a_charge_is_as_fast_as_its_run_allows() {
    let at = |run| contact(Kind::Charge, (16.0, run), (1.5, G));
    assert!((at(4.0) - 6.86).abs() < 0.05, "{}", at(4.0));
    assert!((at(1.0) - 0.5 * at(4.0)).abs() < 1e-3);
    assert_eq!(at(0.0), 0.0);
    assert_eq!(at(400.0), 16.0);
    let drop = contact(Kind::Trample, (14.0, 0.0), (2.2, G));
    assert!((drop - 3.49).abs() < 0.05, "{drop}");
}

/// The robot stands free, so no blow lands with more than its own mass, however heavy the animal;
/// a trample lands with all of the animal, the ground holding the robot under it.
#[test]
fn a_free_robot_meets_no_more_than_its_own_mass() {
    assert!((meets(Kind::Charge, 300.0) - 52.17).abs() < 0.05);
    for mass in [5.0, 50.0, 300.0, 600.0, 6000.0] {
        assert!(meets(Kind::Charge, mass) < ROBOT_KG);
        assert!(meets(Kind::Charge, mass) <= mass * Kind::Charge.share());
    }
    assert_eq!(meets(Kind::Trample, 600.0), 600.0);
}

/// The charges of a 50, a 300 and a 600 kg body after a 4 m run: 450, 1230 and 1490 J. None destroys
/// a whole robot through its torso, the two heavy ones break an arm and throw the robot off its feet
/// (over 3 m/s). The longest charge of the 600 kg body, 7.8 m, still destroys the torso.
#[test]
fn a_big_charge_breaks_parts_and_throws_but_rarely_destroys() {
    let hits =
        [(50.0, 17.2), (300.0, 16.0), (600.0, 14.1)].map(|(mass, top)| charge(mass, top, 4.0));
    for ((energy, _), wanted) in hits.iter().zip([449.0, 1229.0, 1488.0]) {
        assert!(
            (energy - wanted).abs() < 0.02 * wanted,
            "{energy} J, not {wanted}"
        );
        let mut robot = Condition::default();
        robot.hurt(Part::Torso, *energy, true);
        assert!(!robot.ruined(), "{energy} J destroyed a whole robot");
    }
    for (energy, thrown) in &hits[1..] {
        let mut robot = Condition::default();
        robot.hurt(Part::ArmL, *energy, true);
        assert!(
            robot.of(Part::ArmL) <= 0.0 && *thrown > 3.0,
            "{energy} J, {thrown} m/s"
        );
    }
    let mut robot = Condition::default();
    robot.hurt(Part::Torso, charge(600.0, 14.1, 7.8).0, true);
    assert!(robot.ruined());
}

/// A 600 kg body that tramples destroys the leg it lands on and not the robot: 3.6 kJ blunt on a
/// leg, where it carried 60 kJ at its top speed before.
#[test]
fn a_trample_breaks_a_leg() {
    let speed = contact(Kind::Trample, (14.1, 0.0), (2.2, G));
    let energy = joules(Kind::Trample, 600.0, speed);
    assert!((3000.0..4500.0).contains(&energy), "{energy}");
    let mut robot = Condition::default();
    robot.hurt(Part::LegL, energy, false);
    assert!(robot.of(Part::LegL) <= 0.0 && !robot.ruined());
}
