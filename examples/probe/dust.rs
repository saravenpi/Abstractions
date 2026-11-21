//! The dust and the spray the jets of the ship raise, seen from outside: the ship hovers 5, 15
//! and 40 m over sand, over grass and over the sea near the base, then takes off from grass.
//! Selected with `PROBE_STAGES=dust`; run it on `SMOKE_SEED=57`.

use std::f64::consts::TAU;

use abstractions::world::Surface;

use super::*;

/// Heights in metres of the hull over the ground the ship hovers at, in order.
const HEIGHTS: [f64; 3] = [5.0, 15.0, 40.0];

/// Frames at the first height, while the ground streams in and the dust builds up.
const FIRST: u32 = 90;

/// Frames at each height after it, while the dust of the height before clears: 5 s at the
/// pace of flight, past the longest life of a puff.
const DWELL: u32 = 50;

/// Returns a point of the ground of `surface` at least 0.5 m above the sea within 2 km of `from`:
/// the first along one of 16 directions.
fn find_ground(world: &World, from: DVec3, surface: Surface) -> Option<DVec3> {
    let globe = &world.resource::<Planet>().globe;
    let up = from.normalize();
    let east = up.any_orthonormal_vector();
    let north = up.cross(east);
    (0..16).find_map(|k| {
        let turn = f64::from(k) * TAU / 16.0;
        let dir = east * turn.cos() + north * turn.sin();
        (3..200)
            .map(|i| (from + dir * f64::from(i) * 10.0).normalize() * from.length())
            .find(|p| globe.surface(p.normalize()) == surface && globe.height(p.normalize()) > 0.5)
    })
}

/// Returns a point of open sea near the base: 150 m out past the first shore.
fn find_sea(world: &World, from: DVec3) -> Option<DVec3> {
    let shore = find_shore(world, from)? * from.length();
    let out = (shore - from).normalize_or(DVec3::X);
    Some((shore + out * 150.0).normalize() * from.length())
}

/// Returns the camera for a hover `height` metres up, in hull axes: off the left of the nose,
/// farther and higher the higher the ship, looking at the ground under it.
fn camera(height: f64) -> (Vec3, Vec3) {
    let back = 1.0 + height as f32 / 20.0;
    (
        Vec3::new(-14.0, 2.0, -10.0) * back,
        Vec3::new(0.0, -0.8 * height as f32, 0.0),
    )
}

/// Returns which height of [`HEIGHTS`] the hover holds at `frame`, and whether the shot of it is
/// due on this frame.
fn schedule(frame: u32) -> (usize, bool) {
    let index = match frame <= FIRST {
        true => 0,
        false => ((frame - FIRST - 1) / DWELL + 1) as usize,
    };
    let due = FIRST + index as u32 * DWELL;
    (index, frame == due)
}

/// Hovers the ship over the ground found at `spot` at each of [`HEIGHTS`] and shoots each as
/// `<name>-<height>m`. Ends after the last.
fn hover_over(
    world: &mut World,
    notes: &mut Notes,
    (frame, spot, name): (u32, DVec3, &str),
) -> Act {
    let (index, due) = schedule(frame);
    let Some(height) = HEIGHTS.get(index).copied() else {
        return inside(world, notes);
    };
    if frame == 1 {
        pace(world, FLIGHT);
    }
    let (dir, over_sea) = above_ground(world, spot, height);
    let toward = dir.any_orthonormal_vector();
    hover(world, dir, over_sea, (toward, 0.0));
    outside(world, camera(height));
    if due {
        shot(world, &format!("{name}-{height:.0}m"));
    }
    Act::Wait
}

/// Finds a spot on the first frame of a stage with `find` and keeps it in the notes.
fn spot(
    world: &World,
    notes: &mut Notes,
    frame: u32,
    find: impl Fn(&World, DVec3) -> Option<DVec3>,
) -> Option<DVec3> {
    if frame == 1 {
        let pad = world.get_resource::<Base>().map(|base| base.pad)?;
        notes.spots = find(world, pad).into_iter().collect();
    }
    notes.spots.first().copied()
}

/// The ship hovers over a beach or a dune: a thick cloud of pale sand at 5 m, a little at 15 m,
/// none at 40 m.
pub fn dust_sand(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(at) = spot(world, notes, frame, |w, from| {
        find_ground(w, from, Surface::Sand)
    }) else {
        return Act::Fail("no sand within 2 km of the pad".into());
    };
    hover_over(world, notes, (frame, at, "dust-sand"))
}

/// The ship hovers over grass: brown earth dust, thinner than over sand.
pub fn dust_grass(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(at) = spot(world, notes, frame, |w, from| {
        find_ground(w, from, Surface::Grass)
    }) else {
        return Act::Fail("no grass within 2 km of the pad".into());
    };
    hover_over(world, notes, (frame, at, "dust-grass"))
}

/// The ship hovers over the sea: white spray that falls back into it, no dust.
pub fn dust_sea(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(at) = spot(world, notes, frame, find_sea) else {
        return Act::Fail("no sea within 2 km of the pad".into());
    };
    hover_over(world, notes, (frame, at, "dust-sea"))
}

/// The ship hovers just over grass, then climbs on its lift nozzles: a shot a second into the
/// climb, with the dust blown out, and one 12 m up.
pub fn dust_takeoff(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(at) = spot(world, notes, frame, |w, from| {
        find_ground(w, from, Surface::Grass)
    }) else {
        return Act::Fail("no grass within 2 km of the pad".into());
    };
    let rise = bound(world, |s| s.controls.keys.ship.rise);
    let height = world.resource::<ShipTelemetry>().altitude_ground;
    match frame {
        _ if notes.shot_at.is_some() => {
            key(world, rise, false);
            return inside(world, notes);
        }
        1 => pace(world, FLIGHT),
        f if f < FIRST => {
            let (dir, over_sea) = above_ground(world, at, 3.0);
            hover(world, dir, over_sea, (dir.any_orthonormal_vector(), 0.0));
            outside(world, camera(6.0));
        }
        FIRST => key(world, rise, true),
        f if f == FIRST + 10 => shot(world, "dust-takeoff-1s"),
        f if f > FIRST + 10 && (height > 12.0 || f > FIRST + 200) => {
            last(world, notes, (f, "dust-takeoff-12m"))
        }
        _ => {}
    }
    Act::Wait
}
