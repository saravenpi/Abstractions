//! One patch of grass seen from the horizon and from 2 m to 30 km up, straight down and tilted, to compare its colour.

use super::*;

/// Looks along the ground from the wake spot, level, and shoots `40-ground-horizon`.
pub fn ground_horizon(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    match frame {
        1 => {
            let feet = player_at(world);
            let east = feet.normalize().cross(DVec3::Y).normalize_or(DVec3::X);
            stand(world, feet, feet + east * 100.0, 0.02);
            Act::Wait
        }
        90 => {
            hygiene(world, "ground horizon");
            Act::Shoot("40-ground-horizon".into())
        }
        _ => Act::Wait,
    }
}

/// Holds the ship `height` metres over the grass spot and shoots it straight down as `name-down`, then tilted as
/// `name-tilt`.
fn over_grass(world: &mut World, frame: u32, (height, name): (f64, &str)) -> Act {
    let Some(spot) = grass_spot(world) else {
        return Act::Fail("no turf near the wake point".into());
    };
    let (up, radius) = (spot.normalize(), world.resource::<Planet>().globe.radius());
    let at = spot.length() - radius + height;
    let east = up.cross(DVec3::Y).normalize_or(DVec3::X);
    match frame {
        1 => {
            hover(world, up, at, (east, 1.55));
            Act::Wait
        }
        110 => {
            shot(world, &format!("{name}-down"));
            hover(world, up, at, (east, 0.5));
            Act::Wait
        }
        170 => Act::Shoot(format!("{name}-tilt")),
        _ => Act::Wait,
    }
}

/// Two metres over the grass.
pub fn ground_2m(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    over_grass(world, frame, (2.0, "41-ground-2m"))
}

/// Thirty metres over the grass.
pub fn ground_30m(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    over_grass(world, frame, (30.0, "42-ground-30m"))
}

/// Three hundred metres over the grass.
pub fn ground_300m(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    over_grass(world, frame, (300.0, "43-ground-300m"))
}

/// Three kilometres over the grass.
pub fn ground_3km(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    over_grass(world, frame, (3.0e3, "44-ground-3km"))
}

/// Thirty kilometres over the grass.
pub fn ground_30km(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    over_grass(world, frame, (3.0e4, "45-ground-30km"))
}
