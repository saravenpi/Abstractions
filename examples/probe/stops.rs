//! The stops of the far view over the shore, from two metres to twenty thousand kilometres.

use super::*;

/// Holds the ship `height` metres over the sea at the shore for 90 frames, nose down at the
/// planet, and shoots `name`.
fn stop(world: &mut World, notes: &Notes, frame: u32, (height, name): (f64, &str)) -> Act {
    let sea = notes.shore.unwrap_or(DVec3::Y);
    match frame {
        1 => {
            hover(
                world,
                sea,
                height,
                (sea.normalize().any_orthonormal_vector(), 1.0),
            );
            Act::Wait
        }
        90 => {
            hygiene(world, name);
            Act::Shoot(name.into())
        }
        _ => Act::Wait,
    }
}

/// Two metres over the ground.
pub fn stop_2m(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    stop(world, notes, frame, (2.0, "30-stop-2m"))
}

/// Ten kilometres up.
pub fn stop_10km(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    stop(world, notes, frame, (1.0e4, "31-stop-10km"))
}

/// A hundred kilometres up.
pub fn stop_100km(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    stop(world, notes, frame, (1.0e5, "32-stop-100km"))
}

/// A thousand kilometres up, where the world draws the far ball.
pub fn stop_1000km(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    stop(world, notes, frame, (1.0e6, "33-stop-1000km"))
}

/// Twenty thousand kilometres up, past the hand-over, where the sky draws the planet.
pub fn stop_20000km(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    stop(world, notes, frame, (2.0e7, "34-stop-20000km"))
}
