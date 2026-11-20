//! The cost of the senses: one animal's frame, four threats seen over the ground, its stance, stays
//! under 20 microseconds.

use bevy::math::{DVec3, Vec2, Vec3};

use super::feel::{Air, Senser, Threat, perceive};
use super::meter::Meter;
use super::stance::{Foe, Me, choose};
use super::tune::NOSE_HUNTER;
use crate::fauna::place::Spot;
use crate::fauna::tests_react::busy;
use crate::life::Guild;

/// Microseconds of processor time one animal's senses may take per frame.
const BUDGET: f64 = 20.0;

/// Returns an animal at the origin of a frame and four robots spread around it.
fn scene() -> (Senser, Vec<Threat>) {
    let me = Senser {
        spot: Spot::at(DVec3::Y, 1.0e6),
        at: Vec3::ZERO,
        eye: 1.2,
        keen: 1.5,
        nose: NOSE_HUNTER,
        herd: 1,
    };
    let threats = (0..4)
        .map(|i| Threat {
            pos: me
                .spot
                .planet(Vec3::new(12.0 * i as f32 + 8.0, 0.0, 5.0 - 3.0 * i as f32)),
            speed: 1.5,
            loud: 0.5,
            mass: 80.0,
            key: i,
            herd: 0,
            robot: true,
            crouch: 0.0,
        })
        .collect();
    (me, threats)
}

/// A hungry hunter of 200 kg.
const HUNTER: Me = Me {
    mass: 200.0,
    health: 1.0,
    guild: Guild::Hunter,
    hungry: true,
    cornered: false,
    guarding: false,
    grace: false,
};

/// One animal sensing four robots spread around it over rolling ground costs under the budget.
#[test]
fn senses_fit_the_budget() {
    let (me, threats) = scene();
    let ground = |p: Vec2| (0.05 * p.x).sin() * 2.0 + (0.07 * p.y).cos();
    let air = Air {
        wind: me.spot.rot * DVec3::new(2.0, 0.0, 1.0),
        ground: &ground,
        dt: 1.0 / 60.0,
    };
    let mut meter = Meter::default();
    let rounds = 20_000;
    let start = busy();
    let mut stances = 0;
    for _ in 0..rounds {
        if let Some((index, aware)) = perceive(&mut meter, &me, &threats, &air) {
            let foe = Foe {
                mass: threats[index].mass,
                robot: true,
            };
            stances += choose(aware, &HUNTER, &foe) as u32;
        }
    }
    let micros = (busy() - start) / f64::from(rounds) * 1.0e6;
    println!("senses: {micros:.2} us per animal per frame ({stances})");
    assert!(micros < BUDGET, "{micros} us");
}
