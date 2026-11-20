//! Orbit, warp to the next planet, landing and stepping out.

use super::*;

/// Just above the cloud band.
pub fn above(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let sea = notes.shore.unwrap_or(DVec3::Y);
    let body = world.resource::<Location>().body;
    let spec = world.resource::<Universe>().planet(body);
    match frame {
        1 => {
            hover(
                world,
                sea,
                spec.cloud_top + 200.0,
                (sea.normalize().any_orthonormal_vector(), 0.25),
            );
            Act::Wait
        }
        20 => Act::Shoot("26-above-clouds".into()),
        _ => Act::Wait,
    }
}

/// Above the air, the nose down at the planet.
pub fn orbit(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let sea = notes.shore.unwrap_or(DVec3::Y);
    let body = world.resource::<Location>().body;
    let spec = world.resource::<Universe>().planet(body);
    match frame {
        1 => {
            hover(
                world,
                sea,
                spec.atmosphere_height * 1.6,
                (sea.normalize().any_orthonormal_vector(), 1.2),
            );
            Act::Wait
        }
        60 => {
            hygiene(world, "orbit");
            Act::Shoot("27-orbit".into())
        }
        _ => Act::Wait,
    }
}

/// Returns the planet of the system of `home` nearest to it now, itself excluded, or with
/// `SMOKE_TARGET` set to `giant` or `moon`, the gas giant of the system or its first moon.
pub fn neighbour(world: &World, home: BodyId) -> Option<BodyId> {
    let universe = world.resource::<Universe>();
    let wanted = std::env::var("SMOKE_TARGET").unwrap_or_default();
    if wanted == "giant" || wanted == "moon" {
        let system = universe.system(home.star);
        let giant = system.planets.iter().find(|p| {
            p.record.class == abstractions::universe::PlanetClass::GasGiant && p.moons > 0
        })?;
        return match wanted.as_str() {
            "giant" => Some(giant.id),
            _ => universe.moons(giant.id).first().map(|moon| moon.id),
        };
    }
    let time = world.resource::<abstractions::space::Clock>().seconds;
    let from = universe.planet_position(home, time);
    let gap = |id: &BodyId| universe.planet_position(*id, time).distance(from);
    let system = universe.system(home.star);
    let others = system.planets.iter().map(|p| p.id).filter(|id| *id != home);
    others.min_by(|a, b| gap(a).total_cmp(&gap(b)))
}

/// Lifts the ship past the lock sphere of its planet, engages the autopilot to the nearest other
/// planet and shoots the warp.
pub fn warp(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let phase = world.resource::<Autopilot>().phase();
    match frame {
        1 => {
            let sea = notes.shore.unwrap_or(DVec3::Y);
            let radius = world.resource::<Planet>().globe.radius();
            hover(
                world,
                sea,
                radius * 2.5,
                (sea.normalize().any_orthonormal_vector(), 0.0),
            );
            Act::Wait
        }
        10 => {
            let home = world.resource::<Location>().body;
            let Some(far) = neighbour(world, home) else {
                return Act::Fail("no other planet".into());
            };
            notes.far = Some(far);
            info!(
                "SMOKE ship frame before engage {:?}",
                ship(world).map(|s| s.frame)
            );
            world.trigger(Engage(Target::Body(far)));
            Act::Wait
        }
        f if f > 10 && phase == Phase::Cruising => Act::Shoot("29-warp".into()),
        f if f > 2000 => Act::Fail(format!("the autopilot stayed {phase:?}")),
        f if f % 200 == 0 => {
            info!("SMOKE warp frame {f}: autopilot {phase:?}");
            Act::Wait
        }
        _ => Act::Wait,
    }
}

/// Waits for the drop out of warp at the other planet.
pub fn arrive(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let phase = world.resource::<Autopilot>().phase();
    let streamed = world.resource::<Planet>().body;
    match (phase, frame) {
        (_, f) if f > 4000 => Act::Fail(format!("the autopilot stayed {phase:?}")),
        (Phase::Arrived, _) if Some(streamed) == notes.far => Act::Shoot("30-arrival".into()),
        _ => Act::Wait,
    }
}

/// Sets the ship 12 m over dry ground of the new planet and holds the thrusters down until it lands.
pub fn land(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let down = bound(world, |s| s.controls.keys.ship.sink);
    if frame == 1 {
        let globe = world.resource::<Planet>().globe.clone();
        let north = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
        let spawn = north.find_spawn();
        let dir = north.embed(spawn.x, 0.0, spawn.z).normalize();
        let height = f64::from(globe.height(dir).max(0.0)) + 14.0;
        hover(world, dir, height, (dir.any_orthonormal_vector(), 0.0));
        key(world, down, true);
        info!("SMOKE landing on {:?}", notes.far);
    }
    let landed = ship(world).is_some_and(|s| s.landed);
    match (landed, frame) {
        (true, _) => {
            key(world, down, false);
            Act::Shoot("31-landed".into())
        }
        (_, f) if f > 900 => {
            key(world, down, false);
            Act::Fail("the ship never landed".into())
        }
        _ => Act::Wait,
    }
}

/// Out of the seat onto the new planet.
pub fn step_out(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let board = bound(world, |s| s.controls.keys.ship.board);
    match frame {
        30 => {
            pace(world, WALK);
            key(world, board, true);
            Act::Wait
        }
        31 => {
            key(world, board, false);
            Act::Wait
        }
        120 => match world.resource::<Location>().travel {
            Travel::OnFoot => Act::Shoot("32-step-out".into()),
            Travel::InShip => Act::Fail("still seated".into()),
        },
        _ => Act::Wait,
    }
}
