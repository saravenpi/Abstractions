//! Stages that look at the coarse ground from the air, held still, to tell what the streamer has not built
//! yet from what the coarse mesh looks like once it has settled.

use super::*;

/// Frames after the ship is put in place at which the early and the settled shot are taken.
const SHOTS: (u32, u32) = (30, 420);

/// Holds the ship still `high` metres over the sea above the player, looking east and `pitch` radians down,
/// and shoots `name-early` after [`SHOTS`].0 frames and `name-settled` after [`SHOTS`].1. Needs the player
/// aboard: run the `board` stage first.
fn held(world: &mut World, frame: u32, (high, pitch, name): (f64, f64, &str)) -> Act {
    match frame {
        1 => {
            world.resource_mut::<abstractions::screens::HideHud>().0 = true;
            let at = player_at(world);
            let up = at.normalize();
            let east = up.cross(DVec3::Y).normalize_or(DVec3::X);
            hover(world, up, high, (east, pitch));
            eprintln!("PROBE coarse {name}: held {high} m over the sea, {pitch} rad down");
            Act::Wait
        }
        n if n == SHOTS.0 => {
            hygiene(world, &format!("{name} early"));
            shot(world, &format!("{name}-early"));
            Act::Wait
        }
        n if n == SHOTS.1 => {
            hygiene(world, &format!("{name} settled"));
            Act::Shoot(format!("{name}-settled"))
        }
        _ => Act::Wait,
    }
}

/// The coast from 400 m, looking steeply down: the view of the `trails` shots, held still.
pub fn coarse_400(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    held(world, frame, (400.0, 1.0, "C1-400m-down"))
}

/// The land from 400 m, looking at the horizon: the far ground the air fades.
pub fn coarse_400_level(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    held(world, frame, (400.0, 0.15, "C2-400m-level"))
}

/// The land from 3 km, looking half down.
pub fn coarse_3km(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    held(world, frame, (3000.0, 0.5, "C3-3km"))
}
