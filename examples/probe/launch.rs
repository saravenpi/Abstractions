//! Boarding the ship and taking off from the pad and from the shore.

use super::*;

/// Returns the ship.
pub fn ship(world: &mut World) -> Option<Ship> {
    world.query::<&Ship>().iter(world).next().cloned()
}

/// Changes the ship in place.
pub fn with_ship(world: &mut World, change: impl FnOnce(&mut Ship)) {
    if let Some(mut ship) = world.query::<&mut Ship>().iter_mut(world).next() {
        change(&mut ship);
    }
}

/// Back at the base, beside the hatch of the ship, then into its seat.
pub fn board(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let Some(hull) = ship(world) else {
        return Act::Fail("no ship".into());
    };
    let key_board = bound(world, |s| s.controls.keys.ship.board);
    match frame {
        1 => {
            let beside = hull.pos + hull.rot * DVec3::new(-2.6, -1.2, -1.8);
            stand(world, beside, hull.pos, 0.0);
            Act::Wait
        }
        60 => {
            shot(world, "16-hatch");
            Act::Wait
        }
        62 => {
            key(world, key_board, true);
            Act::Wait
        }
        63 => {
            key(world, key_board, false);
            Act::Wait
        }
        90 => match world.resource::<Location>().travel {
            Travel::InShip => Act::Shoot("17-seated".into()),
            Travel::OnFoot => Act::Fail("the board key did not seat the player".into()),
        },
        _ => Act::Wait,
    }
}

/// Holds the vertical thrusters until the ship climbs past 15 m.
pub fn takeoff(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let up = bound(world, |s| s.controls.keys.ship.rise);
    if frame == 1 {
        pace(world, FLIGHT);
        key(world, up, true);
    }
    let height = world.resource::<ShipTelemetry>().altitude_ground;
    if height > 15.0 {
        key(world, up, false);
        return Act::Shoot("19-takeoff".into());
    }
    if frame > 400 {
        key(world, up, false);
        return Act::Fail(format!("the ship reached only {height:.1} m"));
    }
    Act::Wait
}

/// Puts the ship at rest `height` metres above the sea at the planet direction `dir`, the nose
/// along `toward` and pitched down by `pitch` radians.
pub fn hover(world: &mut World, dir: DVec3, height: f64, (toward, pitch): (DVec3, f64)) {
    let planet = world.resource::<Planet>();
    let (radius, body) = (planet.globe.radius(), planet.body);
    let up = dir.normalize();
    let flat = (toward - up * toward.dot(up)).normalize_or(up.any_orthonormal_vector());
    let level = DQuat::from_mat3(&bevy::math::DMat3::from_cols(flat.cross(up), up, -flat));
    with_ship(world, |ship| {
        ship.frame = Frame::Surface(body);
        ship.pos = up * (radius + height);
        ship.rot = level * DQuat::from_rotation_x(-pitch);
        (ship.vel, ship.spin, ship.landed) = (DVec3::ZERO, DVec3::ZERO, false);
    });
}

/// Returns a shore point within 2 km of `from`: the first sea along one of 16 directions.
pub fn find_shore(world: &World, from: DVec3) -> Option<DVec3> {
    let globe = &world.resource::<Planet>().globe;
    let up = from.normalize();
    let east = up.any_orthonormal_vector();
    let north = up.cross(east);
    (0..16).find_map(|k| {
        let a = f64::from(k) * std::f64::consts::TAU / 16.0;
        let dir = east * a.cos() + north * a.sin();
        (1..200)
            .map(|i| (from + dir * f64::from(i) * 10.0).normalize())
            .find(|p| globe.height(*p) < -2.0)
    })
}

/// Puts the ship `height` metres up, 150 m inland of the shore point `sea` seen from `from`,
/// looking out to sea a little below the horizon.
pub fn seaward(world: &mut World, sea: DVec3, from: DVec3, height: f64) {
    let back = (from - sea).normalize();
    let over = (sea + back * 150.0).normalize();
    hover(world, over, height, (sea - over, 0.22));
}

/// The fog from 100 m above the shore.
pub fn shore(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(hull) = ship(world) else {
        return Act::Fail("no ship".into());
    };
    match frame {
        1 => {
            let Some(sea) = find_shore(world, hull.pos) else {
                return Act::Fail("no shore within 2 km".into());
            };
            notes.shore = Some(sea);
            seaward(world, sea, hull.pos, 100.0);
            Act::Wait
        }
        30 => Act::Shoot("20-fog-100m".into()),
        _ => Act::Wait,
    }
}
