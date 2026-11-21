//! Standing in the turf and looking at the blades.

use super::*;

/// Logs the turf the blade material holds.
pub fn turf_report(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if frame < 70 {
        return Act::Wait;
    }
    let handle = world
        .get_resource::<abstractions::world::BladeAssets>()
        .map(|b| b.material.clone());
    let turf = handle.and_then(|h| {
        world
            .resource::<Assets<abstractions::world::BladeMaterial>>()
            .get(&h)
            .map(|m| m.extension.turf)
    });
    info!("SMOKE turf {turf:?}");
    Act::Next
}

/// Finds an open spot of thick turf on grass near the wake point, as a planet position.
pub fn grass_spot(world: &mut World) -> Option<DVec3> {
    let terrain = world.resource::<Terrain>().clone();
    let from = player_at(world);
    let at = terrain.locate(from);
    let mut best: Option<(f32, Vec2)> = None;
    for k in 0..240 {
        let angle = k as f32 * 2.399_963;
        let r = 6.0 + 1.4 * k as f32;
        let w = at.xz() + Vec2::new(angle.cos(), angle.sin()) * r;
        if terrain.surface(w.x, w.y) != abstractions::world::Surface::Grass {
            continue;
        }
        let thick = abstractions::world::turf_at(&terrain, w);
        if thick > best.map_or(0.0, |b| b.0) + 0.05 {
            best = Some((thick, w));
        }
        if thick > 0.9 {
            break;
        }
    }
    let (thick, w) = best?;
    info!("SMOKE grass spot turf {thick:.2} at chart {w}");
    Some(terrain.embed(w.x, terrain.mesh_height(w.x, w.y), w.y))
}

/// Stands at a spot of thick turf, the camera lowered by `low` metres and pitched by `pitch`, and shoots `name`.
pub fn grass_view(world: &mut World, frame: u32, (low, pitch): (f32, f32), name: &str) -> Act {
    match frame {
        1 => {
            world.resource_mut::<Lower>().0 = 0.0;
            let Some(spot) = grass_spot(world) else {
                return Act::Fail("no turf near the wake point".into());
            };
            let up = spot.normalize();
            let east = up.cross(DVec3::Y).normalize_or(DVec3::X);
            stand(world, spot, spot + east * 10.0, pitch);
            Act::Wait
        }
        150 => {
            world.resource_mut::<Lower>().0 = low;
            hygiene(world, name);
            Act::Wait
        }
        160 => Act::Shoot(name.into()),
        _ => Act::Wait,
    }
}

/// Standing in the turf, looking a little down.
pub fn grass_stand(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    grass_view(world, frame, (0.0, -0.3), "20-grass-stand")
}

/// The camera low in the turf, looking along it.
pub fn grass_low(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    grass_view(world, frame, (1.0, -0.05), "21-grass-low")
}

/// Standing in the turf, looking out to where the blades thin into the carpet of the ground.
pub fn grass_out(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    grass_view(world, frame, (0.0, -0.08), "22-grass-out")
}

/// Frames the walk of [`grass_walk`] stands still first, while the blades around it are built.
const FILL: u32 = 120;

/// Metres the walk of [`grass_walk`] goes each frame: 6 m/s in a game of 30 frames a second.
const PACE: f64 = 0.2;

/// Counts the entities that draw blades, and those of them in view.
fn blade_count(world: &mut World) -> (usize, usize) {
    let mut blades = world.query::<(&Name, &ViewVisibility)>();
    let drawn = blades
        .iter(world)
        .filter(|(name, _)| name.as_str() == "Blades");
    drawn.fold((0, 0), |(all, seen), (_, view)| {
        (all + 1, seen + usize::from(view.get()))
    })
}

/// Returns the point on the ground under planet position `p` of `terrain`.
fn ground_under(terrain: &Terrain, p: DVec3) -> DVec3 {
    let at = terrain.locate(p);
    terrain.embed(at.x, terrain.mesh_height(at.x, at.z), at.z)
}

/// Walks east over the turf at 6 m/s, looking ahead and a little down, after standing [`FILL`] frames, logs the
/// blade entities every second of the walk, and shoots after 60 m: the blades ahead must stand to 50 m and thin
/// out smoothly, with no bare square near the feet.
pub fn grass_walk(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        world.resource_mut::<Lower>().0 = 0.0;
        let Some(spot) = grass_spot(world) else {
            return Act::Fail("no turf near the wake point".into());
        };
        notes.spots = vec![spot];
    }
    let Some(&spot) = notes.spots.first() else {
        return Act::Fail("no start for the walk".into());
    };
    let terrain = world.resource::<Terrain>().clone();
    let east = spot.normalize().cross(DVec3::Y).normalize_or(DVec3::X);
    let walked = f64::from(frame.saturating_sub(FILL)) * PACE;
    if frame >= FILL && (frame - FILL) % 30 == 0 {
        let (all, seen) = blade_count(world);
        info!("SMOKE grass walk {walked:.0} m: {all} blade bands, {seen} in view");
    }
    if frame == FILL + 300 {
        hygiene(world, "23-grass-walk");
        return Act::Shoot("23-grass-walk".into());
    }
    let feet = ground_under(&terrain, spot + east * walked);
    stand(
        world,
        feet,
        ground_under(&terrain, spot + east * (walked + 10.0)),
        -0.12,
    );
    Act::Wait
}

/// The same spot from 60 m up, where no blade is drawn and only the ground carpet shows.
pub fn grass_up(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    match frame {
        1 => {
            world.resource_mut::<Lower>().0 = 0.0;
            let Some(spot) = grass_spot(world) else {
                return Act::Fail("no turf near the wake point".into());
            };
            let up = spot.normalize();
            let east = up.cross(DVec3::Y).normalize_or(DVec3::X);
            stand(world, spot + up * 60.0, spot + east * 80.0, -0.6);
            Act::Wait
        }
        12 => Act::Shoot("26-grass-up".into()),
        _ => Act::Wait,
    }
}
