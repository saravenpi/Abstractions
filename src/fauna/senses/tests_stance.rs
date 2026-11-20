//! Tests of fight or flight: grazers flee a robot, big animals and those guarding young stand,
//! hungry hunters go for a smaller robot but not in the grace after a landing.

use super::stance::{Foe, Me, Stance, choose};
use super::tune::ROBOT_KG;
use crate::life::Guild;

/// Returns an unhurt, fed, free animal of `guild` and `mass` kilograms.
fn me(guild: Guild, mass: f32) -> Me {
    Me {
        mass,
        health: 1.0,
        guild,
        hungry: false,
        cornered: false,
        guarding: false,
        grace: false,
    }
}

/// The robot as a foe.
const ROBOT: Foe = Foe {
    mass: ROBOT_KG,
    robot: true,
};

/// Low awareness keeps an animal calm, then wary, before it decides.
#[test]
fn awareness_comes_first() {
    let grazer = me(Guild::Grazer, 100.0);
    assert_eq!(choose(0.1, &grazer, &ROBOT), Stance::Calm);
    assert_eq!(choose(0.5, &grazer, &ROBOT), Stance::Wary);
    assert_eq!(choose(0.9, &grazer, &ROBOT), Stance::Flee);
}

/// A grazer flees anything over half its size, even with young, unless cornered.
#[test]
fn grazers_flee_big_threats() {
    let guarding = Me {
        guarding: true,
        ..me(Guild::Grazer, 150.0)
    };
    assert_eq!(choose(1.0, &guarding, &ROBOT), Stance::Flee);
    let cornered = Me {
        cornered: true,
        ..me(Guild::Grazer, 30.0)
    };
    assert_eq!(choose(1.0, &cornered, &ROBOT), Stance::Threaten);
    assert_eq!(
        choose(1.0, &me(Guild::Grazer, 400.0), &ROBOT),
        Stance::Threaten
    );
}

/// A scavenger fights by the mass ratio, and a wounded one flees.
#[test]
fn the_mass_ratio_decides() {
    let big = me(Guild::Scavenger, 60.0);
    assert_eq!(choose(1.0, &big, &ROBOT), Stance::Threaten);
    let hurt = Me { health: 0.3, ..big };
    assert_eq!(choose(1.0, &hurt, &ROBOT), Stance::Flee);
    let guarding = Me {
        guarding: true,
        ..hurt
    };
    assert_eq!(choose(1.0, &guarding, &ROBOT), Stance::Threaten);
}

/// A hungry hunter bigger than the robot goes for it once wary, but not in the grace after a
/// landing, and not when it is smaller.
#[test]
fn hungry_hunters_attack_robots() {
    let hungry = Me {
        hungry: true,
        ..me(Guild::Hunter, 200.0)
    };
    assert_eq!(choose(0.4, &hungry, &ROBOT), Stance::Attack);
    let early = Me {
        grace: true,
        ..hungry
    };
    assert_ne!(choose(0.4, &early, &ROBOT), Stance::Attack);
    let small = Me {
        mass: 30.0,
        ..hungry
    };
    assert_eq!(choose(1.0, &small, &ROBOT), Stance::Flee);
    let fed = me(Guild::Hunter, 200.0);
    assert_eq!(choose(1.0, &fed, &ROBOT), Stance::Threaten);
}
