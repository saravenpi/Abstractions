//! Third person tours on foot, in the glide and in flight.

use super::*;

/// Third person, standing at the wake point of the base.
pub fn tp_stand(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let Some(base) = world.get_resource::<Base>().copied() else {
        return Act::Fail("no home base".into());
    };
    match frame {
        1 => {
            view(world, ViewMode::Third, ViewMode::First);
            let feet = ground(world, base.wake, 0.0);
            stand(world, feet, base.pad, -0.1);
            Act::Wait
        }
        60 => Act::Shoot("11-tp-stand".into()),
        _ => Act::Wait,
    }
}

/// Third person, walking toward the pad.
pub fn tp_walk(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let forward = bound(world, |s| s.controls.keys.forward);
    match frame {
        1 => {
            key(world, forward, true);
            Act::Wait
        }
        40 => {
            shot(world, "12-tp-walk");
            Act::Wait
        }
        45 => {
            key(world, forward, false);
            Act::Next
        }
        _ => Act::Wait,
    }
}

/// Third person, jumping and taking off with boots and wingsuit.
pub fn tp_takeoff(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let jump = bound(world, |s| s.controls.keys.jump);
    if frame == 1 {
        key(world, jump, true);
    }
    if frame > 4 {
        spread(world);
    }
    match frame {
        45 => {
            info!("SMOKE take-off clearance {:.1} m", clearance(world));
            Act::Shoot("13-tp-takeoff".into())
        }
        _ => Act::Wait,
    }
}

/// Third person, gliding in the wingsuit, high enough above the ground.
pub fn tp_glide(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let jump = bound(world, |s| s.controls.keys.jump);
    match frame {
        1 => {
            key(world, jump, false);
            let high = clearance(world);
            info!("SMOKE glide starts {high:.1} m above the ground");
            if high < 40.0 {
                let base = world.get_resource::<Base>().map_or(DVec3::Y, |b| b.centre);
                let side = base.normalize().cross(DVec3::Y).normalize_or(DVec3::X);
                let feet = ground(world, base - side * 160.0, 160.0);
                stand(world, feet, ground(world, base, 160.0), -0.2);
            }
            Act::Wait
        }
        f if f <= 4 => Act::Wait,
        f if f < 70 => {
            spread(world);
            match f {
                60 => Act::Shoot("14-tp-glide".into()),
                _ => Act::Wait,
            }
        }
        _ => Act::Next,
    }
}

/// Third person, powered flight: wingsuit and boots, trails out.
pub fn tp_fly(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let jump = bound(world, |s| s.controls.keys.jump);
    spread(world);
    match frame {
        1 => {
            key(world, jump, true);
            Act::Wait
        }
        50 => {
            info!(
                "SMOKE powered flight {:.1} m above the ground",
                clearance(world)
            );
            shot(world, "15-tp-fly");
            Act::Wait
        }
        52 => {
            key(world, jump, false);
            view(world, ViewMode::First, ViewMode::First);
            Act::Next
        }
        _ => Act::Wait,
    }
}
