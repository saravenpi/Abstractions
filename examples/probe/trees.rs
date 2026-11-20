//! Standing under the nearest tree in leaf or bare.

use super::*;

/// Returns the planet position and the height of the nearest grown big plant to the player that `fits`.
pub fn tree(
    world: &mut World,
    fits: impl Fn(&abstractions::world::canopy::Crown) -> bool,
) -> Option<(DVec3, f32, f32)> {
    let from = player_at(world);
    let mut best: Option<(f64, DVec3, f32, f32)> = None;
    let mut q = world.query::<(
        &abstractions::world::Sited,
        &abstractions::world::canopy::Canopy,
    )>();
    for (sited, canopy) in q.iter(world) {
        for c in canopy
            .0
            .iter()
            .filter(|c| c.foot > 0.0 && c.height > 2.5 && fits(c))
        {
            let at = sited.pos + sited.rot * c.at.as_dvec3();
            let d = at.distance(from);
            if best.is_none_or(|b| d < b.0) {
                best = Some((d, at, c.height, c.reach));
            }
        }
    }
    best.map(|(d, at, h, r)| {
        info!("SMOKE tree {d:.0} m away, {h:.1} m tall, crown {r:.1} m");
        (at, h, r)
    })
}

/// Stands under the nearest tree that `fits`, looking up into its crown, and shoots `name`.
pub fn under_tree(
    world: &mut World,
    frame: u32,
    fits: fn(&abstractions::world::canopy::Crown) -> bool,
    name: &str,
) -> Act {
    match frame {
        1 => {
            world.resource_mut::<Lower>().0 = 0.0;
            let Some((foot, height, reach)) = tree(world, fits) else {
                return Act::Fail("no such tree near the wake point".into());
            };
            let up = foot.normalize();
            let east = up.cross(DVec3::Y).normalize_or(DVec3::X);
            let feet = ground(world, foot + east * f64::from(reach.max(2.0) + 1.5), 0.0);
            stand(world, feet, foot + up * f64::from(height * 0.5), 0.25);
            Act::Wait
        }
        200 => {
            hygiene(world, name);
            Act::Shoot(name.into())
        }
        _ => Act::Wait,
    }
}

/// Under the nearest tree in leaf.
pub fn leafy_tree(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    under_tree(
        world,
        frame,
        |c| c.shed.is_some_and(|s| s.leaf.is_some()),
        "23-leafy-tree",
    )
}

/// Under the nearest bare tree.
pub fn bare_tree(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    under_tree(
        world,
        frame,
        |c| c.shed.is_none_or(|s| s.leaf.is_none()),
        "24-bare-tree",
    )
}
