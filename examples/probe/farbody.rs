//! Stages that judge the planets and moons seen from afar, the interface hidden: the home planet
//! from 2,000, 20,000 and 200,000 km, a neighbour, a moon and a ringed giant from a few radii and
//! from far, and a moon in the sky of the ground by night and by day.
//!
//! Selected with `PROBE_STAGES=far-bodies`. Each stage boards the ship by itself. The home planet,
//! the neighbour and the moon from afar are in the `farshots` module.

use abstractions::screens::HideHud;

use super::*;

/// Frames a view is held before its shot: time for the finer surface to bake and show.
const SETTLE: u32 = 600;

/// Holds the ship over the day side of the home planet, `height` metres up, nose straight down,
/// and shoots `name`.
pub fn over_home(
    world: &mut World,
    notes: &mut Notes,
    frame: u32,
    (height, name): (f64, &str),
) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    if t == 1 {
        world.resource_mut::<HideHud>().0 = true;
        let light = star(world);
        let side = light.any_orthonormal_vector();
        let up = (light + side * 0.8).normalize();
        hover(world, up, height, (side, 1.5));
        eprintln!("PROBE far-bodies {name}: {height} m over the home planet");
    }
    match t {
        SETTLE => {
            hygiene(world, name);
            Act::Shoot(name.into())
        }
        _ => Act::Wait,
    }
}

/// Returns the body of the home system `which` names: the nearest planet, the first moon of any
/// planet, or a gas giant with rings.
fn pick(world: &World, which: &str) -> Option<BodyId> {
    let universe = world.resource::<Universe>();
    let home = universe.home();
    let planets = universe.system(home.star).planets;
    match which {
        "moon" => planets
            .iter()
            .find_map(|p| universe.moons(p.id).first().map(|m| m.id)),
        "giant" => planets
            .iter()
            .find(|p| {
                p.record.class == abstractions::universe::PlanetClass::GasGiant && p.rings.is_some()
            })
            .map(|p| p.id),
        _ => neighbour(world, home),
    }
}

/// Holds the ship `radii` radii from the centre of the body `which` names, on its lit side a
/// little off the line to the star, looking at it, and shoots `name`.
pub fn visit(
    world: &mut World,
    notes: &mut Notes,
    frame: u32,
    (which, radii, name): (&str, f64, &str),
) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    let Some(id) = pick(world, which) else {
        eprintln!("PROBE far-bodies {name}: the home system has no {which}");
        return Act::Next;
    };
    if t == 1 {
        world.resource_mut::<HideHud>().0 = true;
        let time = world.resource::<abstractions::space::Clock>().seconds;
        let spec = world.resource::<Universe>().planet(id);
        let centre = spec.position(time);
        let sunward = -centre.normalize();
        let side = sunward.cross(DVec3::Y).normalize_or(DVec3::X);
        let from =
            centre + (sunward + side * 0.6 + DVec3::Y * 0.3).normalize() * spec.radius * radii;
        aim(world, from, centre);
        eprintln!(
            "PROBE far-bodies {name}: {id:?} of {} m from {radii} radii",
            spec.radius
        );
    }
    match t {
        SETTLE => {
            hygiene(world, name);
            Act::Shoot(name.into())
        }
        _ => Act::Wait,
    }
}

/// Holds the ship 30 m over the ground under the robot of the planet `home`, looking at its moon
/// `moon` wherever it stands, and logs how high it stands.
fn look_up(world: &mut World, (home, moon): (BodyId, BodyId), name: &str) {
    let time = world.resource::<abstractions::space::Clock>().seconds;
    let universe = world.resource::<Universe>();
    let toward = universe.planet_position(moon, time) - universe.planet_position(home, time);
    let spin = world
        .resource::<abstractions::render::Vantage>()
        .observer
        .spin;
    let seen = (spin.inverse() * toward).normalize();
    let up = robot(world).normalize();
    let rise = seen.dot(up).clamp(-1.0, 1.0).asin();
    hover(world, up, 30.0, (seen, -rise));
    eprintln!(
        "PROBE far-bodies {name}: the moon stands {:.1} degrees up",
        rise.to_degrees()
    );
}

/// Holds the ship 30 m over the ground of the home planet at night or by day, looking at the
/// first moon of the home planet, and shoots `name`.
fn moon_sky(world: &mut World, notes: &mut Notes, frame: u32, (night, name): (bool, &str)) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    let home = world.resource::<Universe>().home();
    let Some(moon) = world
        .resource::<Universe>()
        .moons(home)
        .first()
        .map(|m| m.id)
    else {
        eprintln!("PROBE far-bodies {name}: the home planet has no moon");
        return Act::Next;
    };
    match t {
        1 => {
            world.resource_mut::<HideHud>().0 = true;
            clock_at(world, if night { -0.4 } else { 0.5 }, !night);
        }
        30 => look_up(world, (home, moon), name),
        n if n == 30 + SETTLE => {
            hygiene(world, name);
            return Act::Shoot(name.into());
        }
        _ => {}
    }
    Act::Wait
}

/// A gas giant with rings from six radii.
pub fn far_giant(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    visit(world, notes, frame, ("giant", 6.0, "F8-giant-rings"))
}

/// A moon of the home planet in the sky of the ground at night.
pub fn far_sky_night(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    moon_sky(world, notes, frame, (true, "F9-moon-sky-night"))
}

/// A moon of the home planet in the sky of the ground by day.
pub fn far_sky_day(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    moon_sky(world, notes, frame, (false, "F10-moon-sky-day"))
}
