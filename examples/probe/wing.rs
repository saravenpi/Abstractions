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

/// Sine of the height of the star under which it is night: 10 degrees under the horizon.
const NIGHT: f64 = -0.17;

/// Moves the clock to the next moment within a day at which the star is 10 degrees or more under
/// the horizon at the ship, and keeps the time it left in `notes`. Returns false, with the
/// clock untouched and a line in the log, when the star never sets there: a planet that turns
/// one face to its star has no night at its day side, and a picture taken there is no night
/// picture.
pub fn nightfall(world: &mut World, notes: &mut Notes) -> bool {
    let Some(hull) = ship(world) else {
        return false;
    };
    let body = world.resource::<Planet>().body;
    let spec = world.resource::<Universe>().planet(body);
    let (here, now) = (
        hull.pos.normalize(),
        world.resource::<abstractions::space::Clock>().seconds,
    );
    let step = spec.day_seconds / 240.0;
    let low = |k: &i32| {
        here.dot(abstractions::universe::sun(
            &spec,
            now + step * f64::from(*k),
        ))
    };
    let Some(found) = (0..240).find(|k| low(k) < NIGHT) else {
        eprintln!(
            "PROBE night: skipped, the star cannot set here, use a seed whose home turns (SMOKE_SEED=18)"
        );
        return false;
    };
    eprintln!(
        "PROBE night: clock moved {:.0} s, star height {:.2}",
        step * f64::from(found),
        low(&found)
    );
    notes.day = Some(now);
    world.resource_mut::<abstractions::space::Clock>().seconds = now + step * f64::from(found);
    true
}

/// Puts the clock back to the time a night stage left, when it left one.
pub fn daybreak(world: &mut World, notes: &mut Notes) {
    if let Some(day) = notes.day.take() {
        world.resource_mut::<abstractions::space::Clock>().seconds = day;
    }
}

/// Switches the headlight of the ship.
fn headlight(world: &mut World, on: bool) {
    with_ship(world, |ship| ship.headlight = on);
}

/// A banked turn at 150 m/s, 400 m over the pad, seen from outside: the trails by day from
/// behind and from the side, then at the next night the same with the streaks of the
/// lamps, then the headlight on and off. Where the star never sets the night shots are left out.
/// The clock goes back to where it was at the end.
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
        daybreak(world, notes);
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
        61 if !nightfall(world, notes) => notes.shot_at = Some(frame),
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
