//! Trail tests: a wing tip trails only where the wing makes lift in air, a lamp leaves a streak
//! only in flight, and the load read from the path of a hull, which is how the ship of another
//! player is read, is the load of the same flight computed directly.

use bevy::prelude::*;

use super::{Air, Flown, lamp_heat, lamp_life, load, vortex_heat};

/// Pull of the planet of the tests, in metres per second squared.
const G: f32 = 9.81;

/// Returns the air of a ship at `speed` under `load` in thick, wet air at night.
fn humid(speed: f32, load: f32) -> Air {
    Air {
        speed,
        load,
        thick: 1.0,
        moist: 1.0,
        day: 0.0,
        ..Air::default()
    }
}

/// A wing tip leaves no trail in a vacuum, none at rest on the ground, none slow on the lift
/// nozzles and none on a ballistic arc; it leaves one in level flight in air, a stronger one in
/// a banked turn, and a stronger one still in wet air.
#[test]
fn a_wing_tip_trails_where_the_wing_lifts_in_air() {
    let turn = humid(120.0, 2.0);
    assert_eq!(vortex_heat(&Air { thick: 0.0, ..turn }), 0.0);
    assert_eq!(vortex_heat(&humid(0.0, 1.0)), 0.0);
    assert_eq!(vortex_heat(&humid(20.0, 1.0)), 0.0);
    assert_eq!(vortex_heat(&humid(120.0, 0.0)), 0.0);
    let level = vortex_heat(&humid(120.0, 1.0));
    let dry = vortex_heat(&Air { moist: 0.0, ..turn });
    assert!(level > 0.0 && vortex_heat(&turn) > level, "{level}");
    assert!(dry > 0.0 && vortex_heat(&turn) > dry, "{dry}");
}

/// A lamp leaves no streak at rest and one in flight, fainter by day, and the streak lasts a
/// fifth as long in a vacuum as in thick air.
#[test]
fn a_lamp_leaves_a_streak_in_flight() {
    assert_eq!(lamp_heat(&humid(0.0, 1.0)), 0.0);
    let night = lamp_heat(&humid(80.0, 1.0));
    let day = lamp_heat(&Air {
        day: 1.0,
        ..humid(80.0, 1.0)
    });
    assert!(
        night > 0.9 && day > 0.0 && day < night * 0.5,
        "{night} {day}"
    );
    assert!(lamp_life(0.0) > 0.0 && lamp_life(0.0) * 4.0 < lamp_life(1.0));
}

/// Level and steady the load is 1, in free fall 0, and in a level turn it is what the bank gives.
#[test]
fn the_load_is_what_the_hull_feels_across_its_path() {
    let ahead = Vec3::NEG_Z * 100.0;
    assert!((load(ahead, Vec3::ZERO, Vec3::Y, G) - 1.0).abs() < 1.0e-5);
    assert!(load(ahead, Vec3::NEG_Y * G, Vec3::Y, G) < 1.0e-5);
    let turning = load(ahead, Vec3::X * G, Vec3::Y, G);
    assert!((turning - 2.0_f32.sqrt()).abs() < 1.0e-4, "{turning}");
}

/// A hull that is only seen, frame after frame, flying a level circle at 120 m/s with a radius
/// of 800 m, reads the load of that turn within 3 percent, and its wing tips trail as strongly
/// as those of a ship whose speed and load are known directly, within 3 percent.
#[test]
fn a_ship_that_is_only_seen_trails_like_the_one_that_is_flown() {
    let (speed, radius, dt) = (120.0_f32, 800.0_f32, 1.0 / 60.0);
    let known = (1.0 + (speed * speed / (radius * G)).powi(2)).sqrt();
    let at = |t: f32| {
        let turn = speed / radius * t;
        Vec3::new(turn.cos(), 0.0, turn.sin()) * radius
    };
    let mut flown = Flown::default();
    let mut seen = (0.0, 1.0);
    for frame in 1..240 {
        let (before, now) = (at((frame - 1) as f32 * dt), at(frame as f32 * dt));
        seen = flown.advance(Some((now - before) / dt), (Vec3::Y, G), dt);
    }
    assert!(
        (seen.0 - speed).abs() < 0.5 && (seen.1 / known - 1.0).abs() < 0.03,
        "{seen:?}"
    );
    let (remote, local) = (humid(seen.0, seen.1), humid(speed, known));
    let share = vortex_heat(&remote) / vortex_heat(&local);
    assert!(
        (share - 1.0).abs() < 0.03 && vortex_heat(&local) > 0.0,
        "{share}"
    );
    assert_eq!(flown.advance(None, (Vec3::Y, G), dt), (0.0, 1.0));
}
