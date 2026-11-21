//! The home base and the launch pad.

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
