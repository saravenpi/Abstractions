//! The feel of the ship in 0.11.0: the plume in thin air, a pitch in forward flight that
//! fires the attitude jets and not the lift nozzles, a landing on the lift nozzles, and a run
//! at full speed low over the ground with the streaming logged.

use super::*;

/// The camera behind and above the ship, looking along its back.
const CHASE: (Vec3, Vec3) = (Vec3::new(0.0, 4.0, 16.0), Vec3::new(0.0, 0.0, 0.0));

/// The camera off the left flank, level with the belly, holding the whole ship and its plume.
const SIDE: (Vec3, Vec3) = (Vec3::new(-14.0, -0.5, 2.0), Vec3::new(0.0, -0.3, 2.0));

/// The camera beside the nose, looking back along the belly at the lift nozzles.
const BELLY: (Vec3, Vec3) = (Vec3::new(-6.5, 0.2, -5.0), Vec3::new(0.0, -0.4, 0.4));

/// Puts the ship `height` metres over the ground under it, level, moving at `speed` along its
/// nose, at the flight pace, and holds the camera at `eye`.
fn set_off(world: &mut World, (height, speed): (f64, f64), eye: (Vec3, Vec3)) {
    let Some(hull) = ship(world) else {
        return;
    };
    let up = hull.pos.normalize();
    let floor = f64::from(world.resource::<Planet>().globe.height(up)).max(0.0);
    hover(world, up, floor + height, (hull.rot * DVec3::NEG_Z, 0.0));
    with_ship(world, |ship| ship.vel = ship.rot * DVec3::NEG_Z * speed);
    pace(world, FLIGHT);
    outside(world, eye);
}

/// The main engines at full throttle where the air is a tenth of the air at sea level, seen
/// from the side: the plume is wider than in thick air but its bright core stays in the nozzle.
pub fn thin(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let body = world.resource::<Location>().body;
    let top = world.resource::<Universe>().planet(body).atmosphere_height;
    match frame {
        1 => set_off(world, (top / 12.0 * 10f64.ln(), 0.0), SIDE),
        2..=30 => throttle(world, 1.0),
        _ => {}
    }
    if frame == 30 {
        last(world, notes, (frame, "39-burn-thin"));
    }
    if notes.shot_at.is_some_and(|at| at < frame) {
        throttle(world, 0.0);
        return inside(world, notes);
    }
    Act::Wait
}

/// A full pitch up at 200 m/s, 300 m over the ground, seen from the chase camera and from the
/// side: the tail and nose attitude jets fire, the lift nozzles under the belly stay dark.
pub fn pitch(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let up = bound(world, |s| s.controls.keys.ship.pitch_up);
    match frame {
        1 => set_off(world, (300.0, 200.0), CHASE),
        5 => key(world, up, true),
        12 => shot(world, "39-pitch-chase"),
        13 => outside(world, SIDE),
        16 => last(world, notes, (frame, "39-pitch-side")),
        _ => {}
    }
    throttle(world, 1.0);
    if notes.shot_at.is_some_and(|at| at < frame) {
        key(world, up, false);
        throttle(world, 0.0);
        return inside(world, notes);
    }
    Act::Wait
}

/// A landing: the ship let go 25 m over the ground with the sink key held, seen from beside
/// the nose, once on the way down and once just over the ground. The lift nozzles hold it up.
pub fn alight(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let down = bound(world, |s| s.controls.keys.ship.sink);
    let height = world.resource::<ShipTelemetry>().altitude_ground;
    match frame {
        1 => set_off(world, (25.0, 0.0), BELLY),
        3 => key(world, down, true),
        20 => shot(world, "39-land-sink"),
        f if f > 20 && notes.shot_at.is_none() && (height < 2.5 || f > 400) => {
            last(world, notes, (f, "39-land-flare"))
        }
        _ => {}
    }
    if notes.shot_at.is_some_and(|at| at < frame) {
        key(world, down, false);
        return inside(world, notes);
    }
    Act::Wait
}

/// Twelve seconds at full lever 300 m over the ground from 230 m/s, seen from the chase camera
/// and from the side, with the speed and the height logged: the ground ahead must be built
/// before it shows.
pub fn dash(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let seen = *world.resource::<ShipTelemetry>();
    match frame {
        1 => set_off(world, (300.0, 230.0), CHASE),
        60 => shot(world, "39-dash-chase"),
        61 => outside(world, SIDE),
        120 => last(world, notes, (frame, "39-dash-side")),
        _ => {}
    }
    throttle(world, 1.0);
    if frame % 20 == 0 {
        info!(
            "SMOKE dash: {:.0} m/s at {:.0} m over the ground",
            seen.surface_speed, seen.altitude_ground
        );
    }
    if notes.shot_at.is_some_and(|at| at < frame) {
        throttle(world, 0.0);
        return inside(world, notes);
    }
    Act::Wait
}
