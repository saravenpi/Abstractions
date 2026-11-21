//! The heat of an atmospheric entry and the climb out of the air: a ship falling from a low
//! orbit into the air of the planet, seen from a chase camera and from the seat, and a climb from
//! above the pad to orbit under the autopilot, seen from the chase camera. The interface is
//! hidden; every shot logs the height, the speed and the glow.

use abstractions::render::{CloudCrossing, Heat};
use abstractions::ship::SEAT;
use abstractions::speed::{sound_speed, temperature_at};
use abstractions::universe::{TOP_SCALES, heights};

use super::*;

/// The chase camera of both flights, in hull axes: behind the left wing and above it, looking
/// at the nose.
const CHASE: (Vec3, Vec3) = (Vec3::new(-14.0, 5.0, 16.0), Vec3::new(0.0, 0.0, -3.0));

/// Frames between two shots while the ship glows, three seconds of the game, and between two
/// shots while it does not.
const EVERY: (u32, u32) = (30, 100);

/// Frames a flight may run before it ends.
const LONGEST: u32 = 4000;

/// Path angle in radians below the horizon of the fall from orbit: four degrees.
const SLOPE: f64 = 0.07;

/// Puts the ship 500 m under the top of the air over the pad, at the speed of a circular orbit
/// there, [`SLOPE`] below the horizon toward the east, nose first, with the assist off and the
/// engines idle.
fn from_orbit(world: &mut World) {
    let body = world.resource::<Location>().body;
    let spec = world.resource::<Universe>().planet(body);
    let up = world
        .get_resource::<Base>()
        .map_or(DVec3::Y, |b| b.pad)
        .normalize();
    let east = DVec3::Y.cross(up).normalize_or(up.any_orthonormal_vector());
    let height = spec.atmosphere_height - 500.0;
    hover(world, up, height, (east, SLOPE));
    let radius = world.resource::<Planet>().globe.radius() + height;
    let speed = (spec.mu() / radius).sqrt();
    with_ship(world, |ship| {
        ship.vel = ship.forward() * speed;
        (ship.assist, ship.throttle) = (false, 0.0);
    });
    eprintln!("PROBE entry from {height:.0} m at {speed:.0} m/s");
}

/// Logs the flight and takes the shot `name-index` every [`EVERY`] frames of `t`, more often
/// while the ship glows.
fn record(world: &mut World, t: u32, name: &str) {
    let heat = world.resource::<Heat>().glow.heat;
    let every = if heat > 0.01 { EVERY.0 } else { EVERY.1 };
    if !t.is_multiple_of(every) {
        return;
    }
    let telemetry = *world.resource::<ShipTelemetry>();
    eprintln!(
        "PROBE {name} {t}: {:.0} m, {:.0} m/s, {:.0} Pa, glow {heat:.2}",
        telemetry.altitude_sea, telemetry.surface_speed, telemetry.dynamic_pressure
    );
    shot(world, &format!("{name}-{t:04}"));
}

/// Takes the shot `name` once, the first time the camera is well inside a real cloud: the view
/// should white out as thickly as the cloud field is there, with no lines on it.
fn cloudy(world: &mut World, notes: &mut Notes, name: &str) {
    let depth = world
        .get_resource::<CloudCrossing>()
        .map_or(0.0, |cloud| cloud.depth);
    let mark = DVec3::splat(-1.0);
    if depth > 0.3 && !notes.spots.contains(&mark) {
        notes.spots.push(mark);
        eprintln!("PROBE {name} in cloud {depth:.2}");
        shot(world, name);
    }
}

/// Gives the camera back to the game and ends a flight stage.
fn landed(world: &mut World, notes: &mut Notes) -> Act {
    pace(world, WALK);
    inside(world, notes)
}

/// Falls from orbit with the camera at `eye` in hull axes, seen from outside when `out`, and
/// shoots the flight down through the cloud layer to 1.5 km.
fn fall(
    world: &mut World,
    notes: &mut Notes,
    frame: u32,
    (eye, out, name): ((Vec3, Vec3), bool, &str),
) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    if t == 1 {
        noon(world);
        world.resource_mut::<Eye>().0 = Some(eye);
        let seen = [ViewMode::First, ViewMode::Third][usize::from(out)];
        view(world, ViewMode::First, seen);
        from_orbit(world);
        notes.spots.clear();
        pace(world, FLIGHT);
        return Act::Wait;
    }
    record(world, t, name);
    cloudy(world, notes, &format!("{name}-cloud"));
    let height = world.resource::<ShipTelemetry>().altitude_sea;
    match t > LONGEST || height < 1500.0 {
        true => landed(world, notes),
        false => Act::Wait,
    }
}

/// The fall from orbit from the chase camera.
pub fn entry(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    fall(world, notes, frame, (CHASE, true, "E-chase"))
}

/// The fall from orbit from the seat, looking ahead through the canopy.
pub fn entry_seat(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let eye = SEAT.as_vec3();
    let ahead = (eye, eye + Quat::from_rotation_x(-0.1) * Vec3::NEG_Z);
    fall(world, notes, frame, (ahead, false, "E-seat"))
}

/// Returns the Mach number of the ship over the planet `body` now.
fn mach(world: &World, body: BodyId) -> f64 {
    let spec = world.resource::<Universe>().planet(body);
    let telemetry = world.resource::<ShipTelemetry>();
    let [top, _, weather] = heights(&spec.record, spec.gravity, spec.radius);
    let scale = (top / TOP_SCALES, weather);
    let cold = temperature_at(
        spec.record.temperature,
        spec.record.mix.kappa(),
        scale,
        telemetry.altitude_sea,
    );
    telemetry.surface_speed / f64::from(sound_speed(cold, &spec.record.mix)).max(1.0)
}

/// Climbs from 40 m over the pad to orbit under the autopilot, from the chase camera: a shot as
/// the ship passes Mach 1, one at 9 km where contrails form, one at the edge of space, and one
/// every ten seconds besides.
pub fn exit(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    let body = world.resource::<Location>().body;
    if t == 1 {
        noon(world);
        outside(world, CHASE);
        let pad = world.get_resource::<Base>().map_or(DVec3::Y, |b| b.pad);
        let height = ground(world, pad, 40.0).length() - world.resource::<Planet>().globe.radius();
        hover(
            world,
            pad.normalize(),
            height,
            (pad.normalize().any_orthonormal_vector(), 0.0),
        );
        pace(world, FLIGHT);
        notes.spots.clear();
        return Act::Wait;
    }
    if t == 2 {
        world.trigger(Engage(Target::Body(body)));
    }
    let top = world.resource::<Universe>().planet(body).atmosphere_height;
    let height = world.resource::<ShipTelemetry>().altitude_sea;
    let marks = [
        mach(world, body) >= 1.0,
        height >= 9000.0,
        height >= 0.9 * top,
    ];
    for (index, (passed, name)) in marks
        .into_iter()
        .zip(["X-mach1", "X-contrail", "X-edge"])
        .enumerate()
    {
        let mark = DVec3::splat(index as f64);
        if passed && !notes.spots.contains(&mark) {
            notes.spots.push(mark);
            eprintln!(
                "PROBE exit {name} at {height:.0} m, Mach {:.2}",
                mach(world, body)
            );
            shot(world, name);
        }
    }
    cloudy(world, notes, "X-cloud");
    if t % EVERY.1 == 0 {
        record(world, t, "X-climb");
    }
    match t > LONGEST || height > 1.2 * top {
        true => landed(world, notes),
        false => Act::Wait,
    }
}
