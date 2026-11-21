//! Blows that meet the world: a swing at the ground that stops on it, and the held tool at the wall
//! of the base, raised out of it in third and first person, then swung into it and stopped there.
//! Shots `w01` to `w04`. Run it on seed 57.

use abstractions::avatar::strike::{Act as Hands, Layer, Strike};
use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};

use super::*;

/// The name of the stage in the log.
const STAGE: &str = "wall";

/// Logs the hands and how far the held tool is raised out of the world, as `what`.
fn read(world: &mut World, what: &str) {
    let hands = strike_of(world).map(|s: Strike| (s.act, s.result, s.landed));
    let raise = world.query::<&Layer>().iter(world).next().map(|l| l.raise);
    say(STAGE, &format!("{what}: hands {hands:?}, raise {raise:?}"));
}

/// Returns true once a blow under way has stopped on something.
fn stopped(world: &mut World) -> bool {
    strike_of(world).is_some_and(|s| matches!(s.act, Hands::Stun(..)) && s.landed)
}

/// Returns the planet position of the wall of the base 1.2 m up, met from 30 m out, and its normal.
fn wall(world: &mut World) -> Option<(DVec3, DVec3)> {
    let anchor = *world.resource::<Anchor>();
    let centre = world.get_resource::<Base>()?.centre;
    let up = centre.normalize();
    let side = up.cross(DVec3::Y).normalize_or(DVec3::X);
    let out = ground(world, centre + side * 30.0, 1.2);
    let (from, to) = (
        anchor.to_render(out),
        anchor.to_render(ground(world, centre, 1.2)),
    );
    let way = Dir3::new(to - from).ok()?;
    let found = world.run_system_once(move |space: SpatialQuery| {
        let hit = space.cast_ray(from, way, 40.0, true, &SpatialQueryFilter::default())?;
        Some((from + way * hit.distance, hit.normal))
    });
    let (at, normal) = found.ok().flatten()?;
    let rotation = anchor.rotation.inverse();
    Some((anchor.to_frame(at), rotation * normal.as_dvec3()))
}

/// The swing at the ground: looking down steeply, a click, and the shot once it stopped.
fn at_ground(world: &mut World, play: &mut Play, held: u32) -> bool {
    match held {
        1 => {
            let anchor = *world.resource::<Anchor>();
            let me = world
                .query_filtered::<&Transform, With<Player>>()
                .iter(world)
                .next()
                .map_or(Vec3::ZERO, |t| t.translation);
            let feet = ground(world, anchor.to_frame(me), 0.0);
            stand(world, feet, ground(world, feet + DVec3::X * 3.0, 0.0), -1.0);
        }
        20 => mouse(world, MouseButton::Left, true),
        22 => mouse(world, MouseButton::Left, false),
        _ => {}
    }
    if held > 22 && stopped(world) && play.once(0) {
        read(world, "w01 swing at the ground");
        shot(world, "w01-swing-ground");
    }
    held >= 90
}

/// The wall: the robot 0.6 m from the base wall, the tool raised out of it in third then first
/// person, a click, and the shot of the blow stopped at the wall.
fn at_wall(world: &mut World, play: &mut Play, held: u32) -> bool {
    if held == 1 {
        let Some((at, normal)) = wall(world) else {
            say(STAGE, "no wall of the base found: the wall is skipped");
            return true;
        };
        let feet = ground(world, at + normal * 0.6, 0.0);
        stand(world, feet, at, 0.0);
    }
    match held {
        40 => {
            read(world, "w02 raised at the wall");
            shot(world, "w02-wall-raised-tp");
            view(world, ViewMode::First, ViewMode::First);
        }
        60 => {
            read(world, "w03 raised at the wall, first person");
            shot(world, "w03-wall-raised-fp");
            view(world, ViewMode::Third, ViewMode::First);
        }
        80 => mouse(world, MouseButton::Left, true),
        82 => mouse(world, MouseButton::Left, false),
        _ => {}
    }
    if held > 82 && stopped(world) && play.once(1) {
        read(world, "w04 the blow at the wall");
        shot(world, "w04-wall-stop");
    }
    held >= 150
}

/// A swing at the ground and the tool at the wall of the base: shots `w01` to `w04`.
pub fn play_wall(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        notes.play.start(frame);
        noon(world);
        view(world, ViewMode::Third, ViewMode::First);
    }
    if frame > 2000 {
        return gave_up(world, STAGE, "the stage did not end in 2000 frames");
    }
    let play = &mut notes.play;
    let held = play.held(frame);
    let done = match play.step {
        0 => {
            if let (1 | 2, Some(slot)) = (held, slot_for(world, false)) {
                hold_slot(world, slot, held);
            }
            held >= 20
        }
        1 => at_ground(world, play, held),
        2 => at_wall(world, play, held),
        _ => {
            tidy(world);
            return Act::Next;
        }
    };
    if done {
        play.next(frame);
    }
    Act::Wait
}
