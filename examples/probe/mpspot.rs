//! Stages of one joined game that look at what the server gave back, and at the lamps of the
//! ships of the others.

use abstractions::net::Remote;
use abstractions::ship::Engine;

use super::*;

/// Frames a game waits after the world is played before it reports where it stands.
const SETTLE: u32 = 90;

/// Logs where the joined player stands and what the ship is, then shoots the view. It is the
/// stage of the checks that leave and join again, restart the server or edit a stored record:
/// the line `MP spot` is compared with the one logged before leaving.
pub fn mp_spot(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if !joined() {
        return Act::Next;
    }
    match frame {
        SETTLE => {
            mp_report(world, "spot");
            spot(world, "after joining");
            Act::Wait
        }
        f if f == SETTLE + 60 => Act::Shoot(format!("41-mp-spot-{}", role())),
        _ => Act::Wait,
    }
}

/// Returns how many lamps hang on the hulls of the ships of the other players.
fn lamps(world: &mut World) -> usize {
    let mut hulls = world.query_filtered::<Entity, (With<Remote>, With<Engine>)>();
    let hulls: Vec<Entity> = hulls.iter(world).collect();
    let mut lights = world.query_filtered::<Entity, Or<(With<SpotLight>, With<PointLight>)>>();
    let lights: Vec<Entity> = lights.iter(world).collect();
    let carried = |light: &Entity| {
        let mut at = *light;
        loop {
            if hulls.contains(&at) {
                return true;
            }
            match world.get::<ChildOf>(at) {
                Some(parent) => at = parent.parent(),
                None => return false,
            }
        }
    };
    lights.iter().filter(|light| carried(light)).count()
}

/// Placeholder for the headlight of the ship record: the first player switches its headlight
/// on in the act of its phase and the second logs how many lamps it sees on that ship. It
/// reports and shoots whatever it sees, so the lead enables it by naming `mp-lights` in
/// `SMOKE_ONLY` once the branch that adds the headlight bit is merged and the act that switches
/// it on is written in `mpact.rs`.
pub fn mp_lights(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if !joined() {
        return Act::Next;
    }
    if frame.is_multiple_of(30) {
        let seen = lamps(world);
        info!("MP lights {frame}: {seen} lamps on the ships of the others");
    }
    match frame {
        f if f > 240 => Act::Shoot(format!("42-mp-lights-{}", role())),
        _ => Act::Wait,
    }
}
