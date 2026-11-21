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

/// Holds the camera at `eye`, in hull axes, and draws the whole pilot in the seat.
fn outside(world: &mut World, eye: (Vec3, Vec3)) {
    world.resource_mut::<Eye>().0 = Some(eye);
    view(world, ViewMode::First, ViewMode::Third);
}

/// Frees the camera, gives the ship its first person view back and ends the stage.
fn inside(world: &mut World, notes: &mut Notes) -> Act {
    world.resource_mut::<Eye>().0 = None;
    notes.shot_at = None;
    view(world, ViewMode::First, ViewMode::First);
    Act::Next
}

/// Takes the last shot of a stage from the held camera. The stage ends on the next frame, once
/// the shot is taken.
fn last(world: &mut World, notes: &mut Notes, (frame, name): (u32, &str)) {
    shot(world, name);
    notes.shot_at = Some(frame);
}

/// Holds the vertical thrusters from the pad and watches from outside: a shot a second after
/// the nozzles light, with the dust, and one 8 m up.
pub fn lift(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let up = bound(world, |s| s.controls.keys.ship.rise);
    let height = world.resource::<ShipTelemetry>().altitude_ground;
    match frame {
        _ if notes.shot_at.is_some() => {
            key(world, up, false);
            return inside(world, notes);
        }
        1 => {
            pace(world, FLIGHT);
            outside(world, LIFT);
        }
        20 => key(world, up, true),
        30 => shot(world, "34-lift-dust"),
        f if f > 30 && (height > 8.0 || f > 400) => last(world, notes, (f, "34-lift-8m")),
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
pub fn burn(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(hull) = ship(world) else {
        return Act::Fail("no ship".into());
    };
    match frame {
        1 => {
            let ahead = hull.rot * DVec3::NEG_Z;
            hover(world, hull.pos.normalize(), 300.0, (ahead, 0.0));
            outside(world, BURN);
        }
        2..=44 => throttle(world, 1.0),
        _ => throttle(world, 0.0),
    }
    match frame {
        30 => shot(world, "35-burn-air"),
        31 => outside(world, WAKE),
        44 => shot(world, "35-burn-air-side"),
        45 => outside(world, BURN),
        58 => shot(world, "35-burn-ember"),
        60 => return inside(world, notes),
        _ => {}
    }
    Act::Wait
}

/// The main engines above the air: full throttle seen from the side. Then the ship is put past
/// the lock sphere of the planet, where the warp drive may spool, and the spool is shot.
pub fn vacuum(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(hull) = ship(world) else {
        return Act::Fail("no ship".into());
    };
    let body = world.resource::<Location>().body;
    let air = world.resource::<Universe>().planet(body).atmosphere_height;
    let radius = world.resource::<Planet>().globe.radius();
    let (up, ahead) = (hull.pos.normalize(), hull.rot * DVec3::NEG_Z);
    let spooling = matches!(hull.warp, Warp::Spooling { left } if left > SPOOL_TIME * 0.3);
    match frame {
        _ if notes.shot_at.is_some() => {
            with_ship(world, |ship| ship.warp = Warp::Off);
            return inside(world, notes);
        }
        1 => {
            hover(world, up, air * 1.6, (ahead, 0.0));
            outside(world, WAKE);
        }
        2..=39 => throttle(world, 1.0),
        40 => shot(world, "36-burn-vacuum"),
        41 => {
            hover(world, up, radius * 2.5, (ahead, 0.0));
            throttle(world, 0.0);
            with_ship(world, |ship| {
                ship.warp = Warp::Spooling { left: SPOOL_TIME }
            });
        }
        f if f > 42 && !spooling => {
            info!("SMOKE warp spool shot with the drive {:?}", hull.warp);
            last(world, notes, (f, "36-warp-spool"));
        }
        _ => {}
    }
    Act::Wait
}
