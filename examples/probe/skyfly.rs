//! The cloud field from the air: the ship held still 3 km and 15 km up and 400 km up in a low
//! orbit, nose on the horizon, under a forced broken cover of 0.4 with the cloud shadows on, from
//! the chase camera, a shot every second of the game over ten seconds. Puffs should keep their
//! size in kilometres all the way out, thin into the haze along the horizon with no streak, gap or
//! bright band at the limb, drift east together a few metres a second without a jump, and their
//! shadows on the ground at 3 km should sit under them and slide with them.

use abstractions::render::CloudCover;

use super::*;

/// The broad cover forced for the shots: a broken sky.
const COVER: f32 = 0.4;

/// Frames the view settles for before the first shot, at [`FLIGHT`] seconds a frame.
const SETTLE: u32 = 40;

/// Frames between two shots: one second of the game.
const EVERY: u32 = 10;

/// Shots of one stage.
const SHOTS: u32 = 11;

/// Holds the ship `height` metres over the pad, nose east pitched down by `pitch` radians.
fn hold(world: &mut World, height: f64, pitch: f64) {
    let up = world
        .get_resource::<Base>()
        .map_or(DVec3::Y, |b| b.pad)
        .normalize();
    let east = DVec3::Y.cross(up).normalize_or(up.any_orthonormal_vector());
    hover(world, up, height, (east, pitch));
}

/// Turns the clouds and their shadows on and forces the cover, or gives the cover back.
fn sky(world: &mut World, on: bool) {
    world.resource_mut::<CloudCover>().0 = on.then_some(COVER);
    if on {
        let mut settings = world.resource_mut::<Persistent<Settings>>();
        settings.graphics.clouds = true;
        settings.graphics.lighting.cloud_shade = true;
    }
}

/// Holds the ship at `height` metres and pitch `pitch`, shooting `name` every second of the game
/// once the view has settled.
fn sweep(
    world: &mut World,
    notes: &mut Notes,
    frame: u32,
    (height, pitch, name): (f64, f64, &str),
) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    if t == 1 {
        noon(world);
        world.resource_mut::<Eye>().0 = Some(CHASE);
        view(world, ViewMode::First, ViewMode::Third);
        sky(world, true);
        pace(world, FLIGHT);
    }
    hold(world, height, pitch);
    if t >= SETTLE && (t - SETTLE).is_multiple_of(EVERY) {
        let second = (t - SETTLE) / EVERY;
        let clock = world.resource::<abstractions::space::Clock>().seconds;
        eprintln!("PROBE {name} {second}: {height:.0} m at {clock:.1} s");
        shot(world, &format!("{name}-{second:02}"));
    }
    match t >= SETTLE + EVERY * (SHOTS - 1) {
        true => {
            sky(world, false);
            landed(world, notes)
        }
        false => Act::Wait,
    }
}

/// The clouds seen from 3 km up, under and among the band, with their shadows on the ground.
pub fn sky_3km(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    sweep(world, notes, frame, (3000.0, 0.12, "S-3km"))
}

/// The clouds seen from 15 km up, over the band, out to the horizon.
pub fn sky_15km(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    sweep(world, notes, frame, (15_000.0, 0.2, "S-15km"))
}

/// The clouds seen from a low orbit 400 km up, down to the limb of the planet.
pub fn sky_orbit(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    sweep(world, notes, frame, (400_000.0, 0.35, "S-orbit"))
}
