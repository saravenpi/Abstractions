//! The opening of the wingsuit from where a glide starts: the top of a jump on flat ground, a
//! run off a cliff, a long fall and a jump off the hull of the parked ship. Each is shot from the
//! side at three moments after the jump key is pressed again, the spread, the middle of the lie
//! and the glide, with a `PROBE glide` line of the airspeed, the sink and the pitch.

use abstractions::craft::Held;
use abstractions::gear::Gear;

use super::*;

/// Frames after the press, at 30 frames a second, of the three moments: 0.13 s, 0.33 s and 1 s.
const MOMENTS: [(u32, &str); 3] = [(4, "spread"), (10, "mid"), (30, "gliding")];

/// Where the camera stands, metres to the right, up and behind the robot, and where it looks.
const SIDE: (Vec3, Vec3) = (Vec3::new(6.0, 1.0, 1.0), Vec3::new(0.0, 0.0, -1.0));

/// Logs the gear of the robot at `moment` of the opening of `case`.
fn log(world: &mut World, case: &str, moment: &str) {
    let found = world
        .query_filtered::<(&Gear, &Motion), With<Player>>()
        .iter(world)
        .next()
        .map(|(gear, motion)| (gear.state, motion.wings));
    let Some((state, wings)) = found else {
        return;
    };
    let held = world
        .query::<&Held>()
        .iter(world)
        .next()
        .map(|held| held.0.is_some());
    eprintln!(
        "PROBE glide {case} {moment}: mode {:?} wings {wings} spread {:.2} prone {:.2} airspeed {:.1} m/s \
         sink {:.1} m/s pitch {:.1} deg clearance {:.1} m tool in hand {held:?}",
        state.mode,
        state.spread(),
        state.prone,
        state.airspeed,
        -state.velocity.y,
        state.pitch.to_degrees(),
        clearance(world),
    );
}

/// Follows the robot from its right side, looking along `heading`.
pub fn stand_beside(world: &mut World, heading: DVec3) {
    *world.resource_mut::<Tripod>() = Tripod::Follow {
        robot: true,
        heading: heading.normalize_or(DVec3::X),
        eye: SIDE.0,
        aim: SIDE.1,
    };
}

/// Runs frame `frame` of the opening of `case` whose jump key is pressed again at frame
/// `press`: the press, then the three shots with their lines, then the next stage.
pub fn opening(world: &mut World, frame: u32, (case, press): (&str, u32)) -> Act {
    let jump = bound(world, |s| s.controls.keys.jump);
    if frame == press {
        key(world, jump, true);
    }
    if frame == press + 2 {
        key(world, jump, false);
    }
    for (after, moment) in MOMENTS {
        if frame == press + after {
            log(world, case, moment);
            shot(world, &format!("52-open-{case}-{moment}"));
        }
    }
    if frame >= press + 32 {
        *world.resource_mut::<Tripod>() = Tripod::Free;
        return Act::Next;
    }
    Act::Wait
}

/// Returns the home base and a direction along the ground from it, or a failure.
pub fn open_site(world: &World) -> Result<(DVec3, DVec3), Act> {
    let base = world
        .get_resource::<Base>()
        .copied()
        .ok_or_else(|| Act::Fail("no home base".into()))?;
    let side = base
        .centre
        .normalize()
        .cross(DVec3::Y)
        .normalize_or(DVec3::X);
    Ok((base.centre, side))
}

/// A jump on flat ground at the wake point of the base, the jump key pressed again at the top:
/// too low to glide, the robot spreads, tips forward and lands.
pub fn open_jump(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let Some(base) = world.get_resource::<Base>().copied() else {
        return Act::Fail("no home base".into());
    };
    let jump = bound(world, |s| s.controls.keys.jump);
    match frame {
        1 => {
            view(world, ViewMode::Third, ViewMode::First);
            stand(world, ground(world, base.wake, 0.0), base.pad, -0.1);
            stand_beside(world, base.pad - base.wake);
        }
        20 => key(world, jump, true),
        22 => key(world, jump, false),
        _ => {}
    }
    opening(world, frame, ("jump", 33))
}

/// A run off a cliff: the robot 160 m over the ground running at 8.5 m/s, the jump key pressed
/// at once, as it would be one step past the edge.
pub fn open_cliff(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let (centre, side) = match open_site(world) {
        Ok(found) => found,
        Err(failed) => return failed,
    };
    if frame == 1 {
        view(world, ViewMode::Third, ViewMode::First);
        stand(
            world,
            ground(world, centre - side * 160.0, 160.0),
            ground(world, centre, 160.0),
            -0.2,
        );
        stand_beside(world, side);
    }
    if frame == 3 {
        for mut motion in world
            .query_filtered::<&mut Motion, With<Player>>()
            .iter_mut(world)
        {
            motion.velocity = Quat::from_rotation_y(motion.yaw) * Vec3::new(0.0, 0.0, -8.5);
        }
    }
    opening(world, frame, ("cliff", 4))
}

/// A fall of two and a half seconds from 300 m over the ground, then the press.
pub fn open_fall(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let (centre, side) = match open_site(world) {
        Ok(found) => found,
        Err(failed) => return failed,
    };
    if frame == 1 {
        view(world, ViewMode::Third, ViewMode::First);
        stand(
            world,
            ground(world, centre + side * 200.0, 300.0),
            ground(world, centre, 300.0),
            -0.3,
        );
        stand_beside(world, -side);
    }
    opening(world, frame, ("fall", 76))
}

/// A jump off the top of the hull of the parked ship, the jump key pressed again at the top.
pub fn open_ship(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let Some(hull) = ship(world) else {
        return Act::Fail("no ship".into());
    };
    let jump = bound(world, |s| s.controls.keys.jump);
    let ahead = hull.rot * DVec3::NEG_Z;
    match frame {
        1 => {
            view(world, ViewMode::Third, ViewMode::First);
            let top = hull.pos + hull.pos.normalize() * 2.5;
            stand(world, top, top + ahead * 10.0, -0.1);
            stand_beside(world, ahead);
        }
        20 => key(world, jump, true),
        22 => key(world, jump, false),
        _ => {}
    }
    opening(world, frame, ("ship", 33))
}
