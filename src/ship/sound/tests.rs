//! What a ship sounds like is read from its real state, the nearest ships are the ones heard,
//! and a ship faster than sound booms once, as its shock wave passes.

use super::engines;
use super::near::{admit, booms, closing};
use crate::audio::{Ch, HULLS, HullSound};
use crate::ship::cockpit::{Cockpit, Lamp};
use crate::ship::exhaust::Exhaust;
use crate::speed::sound_at;
use crate::universe::Universe;

/// Returns a ship in air, `distance` metres from the listener, at `speed` metres per second.
fn ship(distance: f32, speed: f32) -> HullSound {
    HullSound::default()
        .with(Ch::Present, 1.0)
        .with(Ch::Air, 1.0)
        .with(Ch::Distance, distance)
        .with(Ch::Speed, speed)
}

/// The thrust, the lift, the drive and the gear of a ship reach its sound, and its engines run
/// while it flies or someone sits in it, not while it rests empty.
#[test]
fn the_state_of_a_ship_is_its_sound() {
    let exhaust = Exhaust {
        main: 0.8,
        lift: 0.4,
        warp: 0.5,
        air: 0.3,
        ..Exhaust::default()
    };
    let mut cockpit = Cockpit::default();
    cockpit.asked.gear = 0.25;
    cockpit.read.speed = Some(120.0);
    let flying = engines(&exhaust, &cockpit);
    let read =
        [Ch::Thrust, Ch::Lift, Ch::Warp, Ch::Air, Ch::Gear, Ch::Speed].map(|ch| flying.at(ch));
    assert_eq!(read, [0.8, 0.4, 0.5, 0.3, 0.25, 120.0]);
    assert_eq!((flying.at(Ch::Present), flying.at(Ch::Power)), (1.0, 1.0));
    cockpit.read.lamps[Lamp::Landed as usize] = true;
    assert_eq!(engines(&exhaust, &cockpit).at(Ch::Power), 0.0);
}

/// Sound is slower in the cold air high over the home planet than at its sea level.
#[test]
fn sound_is_slower_in_cold_air() {
    let universe = Universe::new(1);
    let home = universe.planet(universe.home());
    let (low, high) = (sound_at(&home, 0.0), sound_at(&home, 8000.0));
    assert!(low > 250.0 && low < 450.0, "{low}");
    assert!(high < low);
}

/// The nearest ships take the slots, nearest first, and a ship that is not there takes none.
#[test]
fn the_nearest_ships_are_heard() {
    let mut slots = [HullSound::default(); 2];
    for far in [900.0, 50.0, 400.0] {
        admit(&mut slots, ship(far, 0.0));
    }
    admit(&mut slots, ship(10.0, 0.0).with(Ch::Present, 0.0));
    assert_eq!(slots.map(|slot| slot.at(Ch::Distance)), [50.0, 400.0]);
}

/// A ship that closes in at Mach 2 and then passes booms once, when its closing speed falls
/// through the speed of sound. A slow ship, and a ship with no air around it, never boom.
#[test]
fn a_fast_ship_booms_as_it_passes() {
    let dt = 0.1;
    assert!((closing(1000.0, &ship(932.0, 700.0), dt) - 2.0).abs() < 0.01);
    assert_eq!(
        closing(1000.0, &ship(932.0, 700.0).with(Ch::Air, 0.0), dt),
        0.0
    );
    assert_eq!(closing(1000.0, &ship(100.0, 700.0), dt), 0.0);
    let mut last = [(1000.0, 0.0); HULLS];
    let mut heard = 0;
    for distance in [932.0, 864.0, 800.0, 760.0, 740.0, 735.0, 745.0] {
        let mut hulls = [HullSound::default(); HULLS];
        hulls[1] = ship(distance, 700.0);
        heard += booms(&hulls, &mut last, dt).count();
    }
    assert_eq!(heard, 1);
    let mut slow = [(1000.0, 0.0); HULLS];
    let hulls = [ship(990.0, 100.0); HULLS];
    assert_eq!(booms(&hulls, &mut slow, dt).count(), 0);
}
