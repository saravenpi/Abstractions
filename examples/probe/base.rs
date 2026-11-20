//! The home base, the launch pad and the nearest village.

use super::*;

/// The whole base from 30 m away.
pub fn base_view(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let Some(base) = world.get_resource::<Base>().copied() else {
        return Act::Fail("no home base".into());
    };
    match frame {
        1 => {
            let from = base.centre + (base.east * 0.6 - base.north) * 30.0;
            let feet = ground(world, from, 0.0);
            stand(world, feet, base.centre, -0.1);
            Act::Wait
        }
        40 => Act::Shoot("04-base".into()),
        _ => Act::Wait,
    }
}

/// The pad and the parked ship, seen from the wake point.
pub fn pad_view(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let Some(base) = world.get_resource::<Base>().copied() else {
        return Act::Fail("no home base".into());
    };
    match frame {
        1 => {
            let feet = ground(world, base.wake, 0.0);
            stand(world, feet, base.pad, 0.0);
            Act::Wait
        }
        40 => Act::Shoot("05-pad-ship".into()),
        _ => Act::Wait,
    }
}

/// Returns the village sites whose plaza lies within 5 m of the planet position `origin`.
pub fn village_sites(
    terrain: &Terrain,
    origin: DVec3,
) -> Vec<abstractions::world::structures::Site> {
    let at = terrain.locate(origin);
    let cell = (Vec2::new(at.x, at.z) / CELL).floor().as_ivec2();
    let cells = (-2..=2).flat_map(|x| (-2..=2).map(move |y| cell + IVec2::new(x, y)));
    cells
        .filter_map(|c| structure_in_cell(terrain, c))
        .filter(|s| s.kind == Kind::Village)
        .filter(|s| {
            terrain
                .embed(s.origin.x, s.floor, s.origin.y)
                .distance(origin)
                < 5.0
        })
        .collect()
}

/// Returns the planet position of the inland village nearest to `near` on the face chart
/// `terrain`: one whose floor is over 5 m above the sea, so not a shore village on stilts.
pub fn nearest_village(terrain: &Terrain, near: DVec3) -> Option<DVec3> {
    let at = terrain.locate(near);
    let cell = (Vec2::new(at.x, at.z) / CELL).floor().as_ivec2();
    let cells = (-40..=40).flat_map(|x| (-40..=40).map(move |y| cell + IVec2::new(x, y)));
    let sites = cells.filter_map(|c| structure_in_cell(terrain, c));
    let villages = sites.filter(|s| s.kind == Kind::Village && s.floor > 5.0);
    let place =
        |s: abstractions::world::structures::Site| terrain.embed(s.origin.x, s.floor, s.origin.y);
    villages
        .map(place)
        .min_by(|a, b| a.distance(near).total_cmp(&b.distance(near)))
}

/// The nearest village from 35 m outside it.
pub fn village(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    match frame {
        1 => {
            let (Some(base), terrain) = (
                world.get_resource::<Base>().copied(),
                world.resource::<Terrain>().clone(),
            ) else {
                return Act::Fail("no home base".into());
            };
            let Some(origin) = nearest_village(&terrain, base.centre) else {
                return Act::Fail("no village within 40 cells".into());
            };
            info!(
                "SMOKE nearest village {:.0} m from the base",
                origin.distance(base.centre)
            );
            notes.village = Some(origin);
            let up = origin.normalize();
            let side = up.cross(DVec3::Y).normalize_or(DVec3::X);
            let feet = ground(world, origin + side * 35.0, 0.0);
            stand(world, feet, origin, -0.15);
            Act::Wait
        }
        150 => {
            hygiene(world, "village");
            Act::Shoot("06-village".into())
        }
        _ => Act::Wait,
    }
}

/// Returns the unit tangent directions around `at`, `count` of them.
pub fn compass(at: DVec3, count: u32) -> Vec<DVec3> {
    let up = at.normalize();
    let east = up.cross(DVec3::Y).normalize_or(DVec3::X);
    let north = up.cross(east);
    (0..count)
        .map(|k| f64::from(k) * std::f64::consts::TAU / f64::from(count))
        .map(|a| east * a.cos() + north * a.sin())
        .collect()
}
