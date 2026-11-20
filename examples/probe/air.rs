//! The ship in the air: fog, chase, hand over to the autopilot and the clouds.

use super::*;

/// 500 m above the ground: the counts the brief asks for.
pub fn low(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(sea) = notes.shore else {
        return Act::Fail("no shore".into());
    };
    match frame {
        1 => {
            let base = world.get_resource::<Base>().map_or(sea * 2.0, |b| b.centre);
            seaward(world, sea, base, 500.0);
            Act::Wait
        }
        40 => {
            hygiene(world, "500 m");
            Act::Shoot("23-fog-500m".into())
        }
        _ => Act::Wait,
    }
}

/// The fog from 300 m above the shore.
pub fn fog_300(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(sea) = notes.shore else {
        return Act::Fail("no shore".into());
    };
    match frame {
        1 => {
            let base = world.get_resource::<Base>().map_or(sea * 2.0, |b| b.centre);
            seaward(world, sea, base, 300.0);
            Act::Wait
        }
        40 => Act::Shoot("21-fog-300m".into()),
        _ => Act::Wait,
    }
}

/// Third person view of the ship, at `name`, then first person again.
pub fn chase(world: &mut World, frame: u32, name: &str) -> Act {
    match frame {
        1 => {
            view(world, ViewMode::First, ViewMode::Third);
            Act::Wait
        }
        40 => {
            shot(world, name);
            Act::Wait
        }
        42 => {
            view(world, ViewMode::First, ViewMode::First);
            Act::Next
        }
        _ => Act::Wait,
    }
}

/// Third person, the ship landed on its pad.
pub fn tp_pad(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    chase(world, frame, "18-tp-ship-pad")
}

/// Third person, the ship at 300 m.
pub fn tp_flight(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    chase(world, frame, "22-tp-ship-300m")
}

/// Third person, the ship in orbit.
pub fn tp_orbit(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    chase(world, frame, "28-tp-ship-orbit")
}

/// 1000 m up, looking down past the ring where the near sea hands over to the far sea.
pub fn handover(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(sea) = notes.shore else {
        return Act::Fail("no shore".into());
    };
    match frame {
        1 => {
            let globe = world.resource::<Planet>().globe.clone();
            let out = compass(sea, 32)
                .into_iter()
                .map(|d| (sea + d * 1500.0).normalize())
                .max_by(|a, b| globe.height(*b).total_cmp(&globe.height(*a)))
                .unwrap_or(sea.normalize());
            hover(
                world,
                sea.normalize(),
                1000.0,
                (out - sea.normalize(), 0.65),
            );
            Act::Wait
        }
        40 => Act::Shoot("24-handover-1000m".into()),
        _ => Act::Wait,
    }
}

/// Inside the cloud band.
pub fn clouds(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let sea = notes.shore.unwrap_or(DVec3::Y);
    let body = world.resource::<Location>().body;
    let spec = world.resource::<Universe>().planet(body);
    match frame {
        1 => {
            let mid = (spec.cloud_base + spec.cloud_top) * 0.5;
            hover(
                world,
                sea,
                mid,
                (sea.normalize().any_orthonormal_vector(), 0.1),
            );
            Act::Wait
        }
        20 => Act::Shoot("25-clouds".into()),
        _ => Act::Wait,
    }
}
