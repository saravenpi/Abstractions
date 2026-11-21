//! Home through the orbital cruise: the ship dropped out of warp at the arrival shell over the far
//! side of the home planet, the home key pressed, the cruise round the planet, the glide into the
//! air and its end near the base, each shot from the seat with the interface shown. Every shot
//! logs the regime, the height, the speed and what the HUD reads of the target.

use abstractions::computer::{HOME_KEY, Homeward};
use abstractions::screens::HideHud;
use abstractions::ship::{Orbital, Regime};

use super::*;

/// Frames the stage may run before it ends: two minutes of cruise and four of glide at a tenth
/// of a second a frame, with room to spare.
const LONGEST: u32 = 5000;

/// Frames between two log lines.
const EVERY: u32 = 50;

/// Puts the seated ship at rest at the height of the arrival shell over the point of the home
/// planet opposite the base, as a warp home drops it there, and returns the base.
fn far_side(world: &mut World) -> Option<Base> {
    let base = world.get_resource::<Base>().copied()?;
    let spec = world.resource::<Universe>().planet(base.body);
    let height = (spec.radius + spec.atmosphere_height * 1.1) * 1.11 - spec.radius;
    let dir = -base.home().normalize();
    let east = DVec3::Y
        .cross(dir)
        .normalize_or(dir.any_orthonormal_vector());
    hover(world, dir, height, (east, 0.0));
    eprintln!("PROBE cruise from {height:.0} m over the far side");
    Some(base)
}

/// Logs the cruise at frame `t`.
fn log(world: &World, t: u32) {
    let orbital = world.resource::<Orbital>();
    let telemetry = world.resource::<ShipTelemetry>();
    let reading = orbital.reading();
    eprintln!(
        "PROBE cruise {t}: {:?}, {:.0} m, {:.0} m/s, to home {:?}",
        orbital.regime(),
        telemetry.altitude_sea,
        telemetry.surface_speed,
        reading.map(|read| (
            read.arc.round(),
            read.eta.round(),
            read.descent.map(f64::round)
        ))
    );
}

/// Takes the shot `name` once, the first frame `passed` holds, marking it in the notes by `index`.
fn once(world: &mut World, notes: &mut Notes, (index, passed, name): (u32, bool, &str)) {
    let mark = DVec3::splat(f64::from(index));
    if passed && !notes.spots.contains(&mark) {
        notes.spots.push(mark);
        log(world, index);
        shot(world, name);
    }
}

/// Returns the shots of the cruise this frame asks for: arrival, cruise under way, past half
/// way, the start of the glide, deep in the air, and the end of the glide near the base.
fn phases(world: &World, notes: &Notes) -> [(u32, bool, &'static str); 5] {
    let orbital = world.resource::<Orbital>();
    let regime = orbital.regime();
    let half = orbital
        .reading()
        .and_then(|read| read.descent)
        .is_some_and(|left| left < 10_000_000.0);
    let low = world.resource::<ShipTelemetry>().altitude_sea < 40_000.0;
    let glided = notes.spots.contains(&DVec3::splat(3.0));
    [
        (1, regime == Regime::Cruise, "C2-cruise"),
        (2, regime == Regime::Cruise && half, "C3-half"),
        (3, regime == Regime::Glide, "C4-glide"),
        (4, regime == Regime::Glide && low, "C5-low"),
        (5, glided && regime == Regime::Off, "C6-end"),
    ]
}

/// Warps home to the far side, presses the home key and shoots the cruise, the glide and its
/// end with the HUD shown. Run it with `PROBE_STAGES=cruise SMOKE_SEED=57`.
pub fn cruise_home(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    match t {
        1 => {
            world.resource_mut::<HideHud>().0 = false;
            notes.spots.clear();
            if far_side(world).is_none() {
                return Act::Fail("no base".into());
            }
            pace(world, FLIGHT);
        }
        20 => shot(world, "C1-arrived"),
        21 => key(world, HOME_KEY, true),
        22 => key(world, HOME_KEY, false),
        40 if world.resource::<Orbital>().regime() == Regime::Off => {
            let home = world.resource::<Universe>().home();
            eprintln!("PROBE cruise: the home key did not start it, asking directly");
            world.trigger(Homeward(home));
        }
        _ => {}
    }
    for phase in phases(world, notes) {
        once(world, notes, phase);
    }
    if t % EVERY == 0 {
        log(world, t);
    }
    let ended = notes.spots.contains(&DVec3::splat(5.0));
    match t > LONGEST || ended {
        true => {
            pace(world, WALK);
            inside(world, notes)
        }
        false => Act::Wait,
    }
}
