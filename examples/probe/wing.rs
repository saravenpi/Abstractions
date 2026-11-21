//! Looking at what the wings leave in flight: the vortex trails of the wing tips in a banked
//! turn by day, the streaks of the navigation lamps at night, and the headlight in the air.

use super::*;

/// The camera behind and above the ship, looking along its back.
const CHASE: (Vec3, Vec3) = (Vec3::new(0.0, 5.0, 18.0), Vec3::new(0.0, 0.0, 0.0));

/// The camera off the left wing, a little ahead, looking back along the trails.
const ABEAM: (Vec3, Vec3) = (Vec3::new(-14.0, 3.0, -6.0), Vec3::new(0.0, 0.0, 2.0));

/// The camera ahead of the nose on the left, looking at the headlight.
const AHEAD: (Vec3, Vec3) = (Vec3::new(-6.0, 2.0, -14.0), Vec3::new(0.0, 0.0, -2.0));

/// Speed in metres per second the ship is given for the turn.
const SPEED: f64 = 150.0;

/// Puts the clock on by half a day.
fn half_a_day(world: &mut World) {
    let half = world.resource::<Planet>().day_seconds * 0.5;
    world.resource_mut::<abstractions::space::Clock>().seconds += half;
}

/// Switches the headlight of the ship.
fn headlight(world: &mut World, on: bool) {
    with_ship(world, |ship| ship.headlight = on);
}

/// A banked turn at 150 m/s, 400 m over the pad, seen from outside: the trails by day from
/// behind and from the side, then half a day later the same at night with the streaks of the
/// lamps, then the headlight on and off. The clock is put on by another half day at the end.
pub fn trails(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(hull) = ship(world) else {
        return Act::Fail("no ship".into());
    };
    let held = [
        bound(world, |s| s.controls.keys.ship.bank_right),
        bound(world, |s| s.controls.keys.ship.pitch_up),
    ];
    if notes.shot_at.is_some() {
        for code in held {
            key(world, code, false);
        }
        half_a_day(world);
        throttle(world, 0.0);
        return inside(world, notes);
    }
    throttle(world, 1.0);
    match frame {
        1 => {
            let ahead = hull.rot * DVec3::NEG_Z;
            hover(world, hull.pos.normalize(), 400.0, (ahead, 0.0));
            with_ship(world, |ship| ship.vel = ship.rot * DVec3::NEG_Z * SPEED);
            pace(world, FLIGHT);
            outside(world, CHASE);
            for code in held {
                key(world, code, true);
            }
        }
        45 => shot(world, "38-trails-day-rear"),
        46 => outside(world, ABEAM),
        60 => shot(world, "38-trails-day-side"),
        61 => half_a_day(world),
        100 => shot(world, "38-trails-night-side"),
        101 => outside(world, CHASE),
        115 => shot(world, "38-trails-night-rear"),
        116 => {
            headlight(world, true);
            outside(world, AHEAD);
        }
        130 => shot(world, "38-headlight-flight-on"),
        131 => headlight(world, false),
        140 => last(world, notes, (frame, "38-headlight-flight-off")),
        _ => {}
    }
    Act::Wait
}
