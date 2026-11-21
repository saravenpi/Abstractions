//! Stages that judge the sky and the light: the same view at several hours of the day.

use abstractions::render::{CloudShell, Starfield};
use abstractions::world::spin;

use super::*;

/// Simulated seconds per frame while the eye adapts to a new hour.
const ADAPT: f64 = 1.0;

/// Frames the eye is given to adapt after the clock moved.
const SETTLE: u32 = 90;

/// Returns the sine of the height of the star over the robot at `time` seconds.
fn star_height(world: &World, here: DVec3, time: f64) -> f64 {
    let body = world.resource::<Planet>().body;
    let spec = world.resource::<Universe>().planet(body);
    let toward = -spec.position(time).normalize_or(DVec3::X);
    here.dot(spin(spec.day_seconds, time).inverse() * toward)
}

/// Moves the clock to the next moment in the day the star stands at the sine `height` over the
/// robot, rising or setting, and logs it.
pub fn clock_at(world: &mut World, height: f64, rising: bool) {
    let body = world.resource::<Planet>().body;
    let day = world.resource::<Universe>().planet(body).day_seconds;
    let here = robot(world).normalize();
    let now = world.resource::<abstractions::space::Clock>().seconds;
    let step = day / 720.0;
    let found = (0..720)
        .map(|k| now + step * f64::from(k))
        .find(|t| {
            let (a, b) = (
                star_height(world, here, *t),
                star_height(world, here, t + step),
            );
            (a - height) * (b - height) <= 0.0 && (b > a) == rising
        })
        .unwrap_or(now);
    eprintln!("PROBE clock moved {:.0} s to height {height}", found - now);
    world.resource_mut::<abstractions::space::Clock>().seconds = found;
}

/// Turns the robot to look at the horizon, `pitch` radians above it, toward the star at its
/// azimuth when `sunward`, away from it otherwise.
pub fn gaze(world: &mut World, pitch: f32, sunward: bool) {
    let here = robot(world);
    let feet = ground(world, here, 0.0);
    let up = feet.normalize();
    let east = up.cross(DVec3::Y).normalize_or(DVec3::X);
    let side = if sunward { east } else { -east };
    stand(world, feet, feet + side * 100.0, pitch);
}

/// A stage that sets the clock to `height` and looks at the sky: wait for the eye, then shoot.
fn hour(
    world: &mut World,
    frame: u32,
    (height, rising, pitch, name): (f64, bool, f32, &str),
) -> Act {
    match frame {
        1 => {
            clock_at(world, height, rising);
            gaze(world, pitch, true);
            pace(world, ADAPT);
            Act::Wait
        }
        n if n == SETTLE => {
            pace(world, WALK);
            Act::Wait
        }
        n if n == SETTLE + 4 => Act::Shoot(name.into()),
        _ => Act::Wait,
    }
}

/// Noon, the horizon ahead.
pub fn look_noon(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    hour(world, frame, (0.8, true, 0.15, "L2-noon"))
}

/// The star low in the afternoon, the horizon ahead.
pub fn look_sunset(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    hour(world, frame, (0.05, false, 0.1, "L3-sunset"))
}

/// Dusk, the star just under the horizon.
pub fn look_dusk(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    hour(world, frame, (-0.12, false, 0.1, "L4-dusk"))
}

/// Deep night, looking at the horizon.
pub fn look_night(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    hour(world, frame, (-0.8, false, 0.1, "L5-night"))
}

/// Deep night, looking high at the sky for the band of the galaxy and the stars.
pub fn look_stars(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    hour(world, frame, (-0.8, false, 1.0, "L6-night-sky"))
}

/// Noon with the cloud shell hidden, then the starfield hidden, to find what draws the grain.
pub fn look_grain(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        gaze(world, 0.3, true);
    }
    let hide = |world: &mut World, clouds: bool, stars: bool| {
        let mut q = world.query_filtered::<&mut Visibility, With<CloudShell>>();
        q.iter_mut(world).for_each(|mut v| {
            *v = [Visibility::Inherited, Visibility::Hidden][usize::from(clouds)]
        });
        let mut q = world.query::<(&Starfield, &mut Visibility)>();
        q.iter_mut(world).for_each(|(_, mut v)| {
            *v = [Visibility::Inherited, Visibility::Hidden][usize::from(stars)]
        });
    };
    match frame {
        30 => {
            shot(world, "L1a-all");
            Act::Wait
        }
        31 => {
            hide(world, true, false);
            Act::Wait
        }
        60 => {
            shot(world, "L1b-no-clouds");
            Act::Wait
        }
        61 => {
            hide(world, false, true);
            Act::Wait
        }
        90 => {
            shot(world, "L1c-no-stars");
            Act::Wait
        }
        91 => {
            hide(world, true, true);
            Act::Wait
        }
        120 => Act::Shoot("L1d-neither".into()),
        _ => Act::Wait,
    }
}
