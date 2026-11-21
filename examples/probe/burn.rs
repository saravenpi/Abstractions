//! Looking at the exhaust from outside the ship: the lift nozzles and their dust at take-off,
//! the main engines in thick air, the nozzles after a shutdown, the plume in vacuum and the
//! spool of the warp drive.

use abstractions::ship::{SPOOL_TIME, Warp};

use super::*;

/// The camera of the take-off: beside the nose, looking back along the belly.
const LIFT: (Vec3, Vec3) = (Vec3::new(-6.5, 0.2, -5.0), Vec3::new(0.0, -0.4, 0.4));

/// The camera of a burn: behind the right wing, looking at the nozzles and down the plume.
const BURN: (Vec3, Vec3) = (Vec3::new(5.5, 1.6, 6.5), Vec3::new(0.0, 0.2, 5.5));

/// The camera of a burn seen from the side, far enough to hold the whole plume.
const WAKE: (Vec3, Vec3) = (Vec3::new(-11.0, 1.0, 6.0), Vec3::new(0.0, 0.2, 5.0));

/// Holds the vertical thrusters from the pad and watches from outside: a shot as the gear
/// leaves the ground, with the dust, and one 8 m up.
pub fn lift(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let up = bound(world, |s| s.controls.keys.ship.rise);
    let height = world.resource::<ShipTelemetry>().altitude_ground;
    match frame {
        1 => {
            pace(world, FLIGHT);
            world.resource_mut::<Eye>().0 = Some(LIFT);
            key(world, up, true);
        }
        12 => shot(world, "34-lift-dust"),
        _ if height > 8.0 || frame > 400 => {
            key(world, up, false);
            world.resource_mut::<Eye>().0 = None;
            return Act::Shoot("34-lift-8m".into());
        }
        _ => {}
    }
    Act::Wait
}

/// Sets the throttle of the ship.
fn throttle(world: &mut World, to: f64) {
    with_ship(world, |ship| ship.throttle = to);
}

/// The main engines 300 m over the pad: full throttle seen from behind and from the side, then
/// the nozzles a moment after the shutdown.
pub fn burn(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let Some(hull) = ship(world) else {
        return Act::Fail("no ship".into());
    };
    match frame {
        1 => {
            let ahead = hull.rot * DVec3::NEG_Z;
            hover(world, hull.pos.normalize(), 300.0, (ahead, 0.0));
            world.resource_mut::<Eye>().0 = Some(BURN);
        }
        2..=44 => throttle(world, 1.0),
        _ => throttle(world, 0.0),
    }
    match frame {
        30 => shot(world, "35-burn-air"),
        31 => world.resource_mut::<Eye>().0 = Some(WAKE),
        44 => shot(world, "35-burn-air-side"),
        45 => world.resource_mut::<Eye>().0 = Some(BURN),
        58 => shot(world, "35-burn-ember"),
        60 => {
            world.resource_mut::<Eye>().0 = None;
            return Act::Next;
        }
        _ => {}
    }
    Act::Wait
}

/// The main engines above the air: full throttle seen from the side, then the warp drive
/// spooling.
pub fn vacuum(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let Some(hull) = ship(world) else {
        return Act::Fail("no ship".into());
    };
    let body = world.resource::<Location>().body;
    let air = world.resource::<Universe>().planet(body).atmosphere_height;
    match frame {
        1 => {
            let ahead = hull.rot * DVec3::NEG_Z;
            hover(world, hull.pos.normalize(), air * 1.6, (ahead, 0.0));
            world.resource_mut::<Eye>().0 = Some(WAKE);
        }
        2..=40 => throttle(world, 1.0),
        41 => {
            throttle(world, 0.0);
            with_ship(world, |ship| {
                ship.warp = Warp::Spooling { left: SPOOL_TIME }
            });
        }
        _ => {}
    }
    match frame {
        40 => shot(world, "36-burn-vacuum"),
        f if f > 41 && !matches!(hull.warp, Warp::Spooling { left } if left > SPOOL_TIME * 0.3) => {
            world.resource_mut::<Eye>().0 = None;
            with_ship(world, |ship| ship.warp = Warp::Off);
            return Act::Shoot("36-warp-spool".into());
        }
        _ => {}
    }
    Act::Wait
}
