//! A javelin drawn and thrown: drawn back with the throw key held, looking level, up and down, in
//! third and in first person; in flight with its trail; planted in soft ground with the prompt that
//! pulls it out; and thrown at a rock it glances off. Shots `j01` to `j09`. Run it on seed 57.

use abstractions::avatar::strike::{Rest, Spear};
use abstractions::player::Interact;

use super::*;

/// The name of the stage in the log.
const STAGE: &str = "throw";

/// The pitches the draw is shot at, with their names: level, up and down.
const PITCHES: [(f32, &str); 3] = [(0.0, "level"), (0.6, "up"), (-0.6, "down")];

/// Returns where the first javelin is and how it rests, if one was thrown.
fn javelin(world: &mut World) -> Option<(Rest, Vec3, Vec3)> {
    world
        .query::<(&Spear, &Transform)>()
        .iter(world)
        .next()
        .map(|(spear, place)| (spear.rest, place.translation, place.rotation * Vec3::Y))
}

/// Logs the javelin and the prompt as `what`.
fn read(world: &mut World, what: &str) {
    let thrown = javelin(world);
    let prompt = world.resource::<Interact>().chosen();
    let hands = strike_of(world).map(|s| (s.act, s.aim));
    say(
        STAGE,
        &format!("{what}: javelin {thrown:?}, prompt {prompt:?}, hands {hands:?}"),
    );
}

/// Stands the robot on the ground `ahead` metres from where it is along its view, looking along it
/// at `pitch`, and returns the planet position of its feet.
fn stand_at(world: &mut World, ahead: f64, pitch: f32) -> DVec3 {
    let anchor = *world.resource::<Anchor>();
    let me = world
        .query_filtered::<&Transform, With<Player>>()
        .iter(world)
        .next()
        .map_or(Vec3::ZERO, |t| t.translation);
    let (feet, north) = (
        anchor.to_frame(me),
        anchor.rotation.inverse() * DVec3::NEG_Z,
    );
    let feet = ground(world, feet + north * ahead, 0.0);
    stand(world, feet, ground(world, feet + north * 30.0, 1.6), pitch);
    feet
}

/// The draw in one view: the throw key held, and a shot at each pitch, numbered from `first`.
fn draws(world: &mut World, held: u32, (first, tag): (u32, &str)) {
    let throw = bound(world, |s| s.controls.keys.throw);
    if held == 2 {
        key(world, throw, true);
    }
    for (index, (pitch, name)) in PITCHES.into_iter().enumerate() {
        let at = 40 + index as u32 * 30;
        if held == at {
            stand_at(world, 0.0, pitch);
        }
        if held == at + 20 {
            read(
                world,
                &format!("j{:02} draw {tag} {name}", first + index as u32),
            );
            shot(
                world,
                &format!("j{:02}-draw-{tag}-{name}", first + index as u32),
            );
        }
    }
}

/// The flight: the throw let go looking a little up, the shot of the javelin in the air with its
/// trail, then the robot stood beside where it planted, with the prompt.
fn flight(world: &mut World, play: &mut Play, held: u32) -> bool {
    let throw = bound(world, |s| s.controls.keys.throw);
    if held == 1 {
        stand_at(world, 0.0, 0.15);
    }
    if held == 30 {
        key(world, throw, false);
    }
    let flying = javelin(world).is_some_and(|(rest, ..)| rest == Rest::Flying);
    if held > 34 && flying && play.once(0) {
        read(world, "j07 in flight");
        shot(world, "j07-flight-trail");
    }
    let rest = javelin(world).filter(|(rest, ..)| *rest != Rest::Flying);
    if let Some((rest, tip, _)) = rest.filter(|_| play.marks[0] == 0) {
        play.marks[0] = held;
        let anchor = *world.resource::<Anchor>();
        let at = anchor.to_frame(tip);
        let feet = ground(world, at + anchor.rotation.inverse() * DVec3::Z * 2.5, 0.0);
        stand(world, feet, at, pitch_to(feet, at));
        say(STAGE, &format!("landed {rest:?}"));
    }
    if play.marks[0] > 0 && held == play.marks[0] + 30 {
        read(world, "j08 planted");
        shot(world, "j08-planted-soil");
        return true;
    }
    held > 400
}

/// The rock: the planted javelin pulled out with the interact key, the robot 6 m from a rock, a
/// throw at it, and the shot of the javelin where it came to rest after glancing off.
fn rock(world: &mut World, play: &mut Play, held: u32) -> bool {
    let (throw, interact) = (
        bound(world, |s| s.controls.keys.throw),
        bound(world, |s| s.controls.keys.ship.board),
    );
    match held {
        2 => key(world, interact, true),
        3 => key(world, interact, false),
        8 => read(world, "after the pull"),
        _ => {}
    }
    if let (12 | 13, Some(slot)) = (held, slot_for(world, true)) {
        hold_slot(world, slot, held - 11);
    }
    if held == 15 {
        let found = world.run_system_once(scan).ok().flatten();
        let Some(rock) = found else {
            say(STAGE, "no rock within 30 m: no throw at rock");
            return true;
        };
        let back = (rock.feet - rock.mid).normalize_or(DVec3::X) * 6.0;
        let feet = ground(world, rock.feet + back, 0.0);
        stand(world, feet, rock.mid, pitch_to(feet, rock.mid));
    }
    match held {
        25 => key(world, throw, true),
        75 => key(world, throw, false),
        _ => {}
    }
    let resting = javelin(world).is_some_and(|(rest, ..)| rest != Rest::Flying);
    if held > 80 && resting && play.once(1) {
        play.marks[1] = held;
    }
    if play.marks[1] > 0 && held == play.marks[1] + 20 {
        read(world, "j09 at the rock");
        shot(world, "j09-rock-glance");
        return true;
    }
    held > 400
}

/// A javelin drawn at three pitches in third and first person, thrown into soft ground and at a
/// rock: shots `j01` to `j09`.
pub fn play_throw(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        notes.play.start(frame);
        noon(world);
        view(world, ViewMode::Third, ViewMode::First);
    }
    if frame > 3000 {
        return gave_up(world, STAGE, "the stage did not end in 3000 frames");
    }
    let play = &mut notes.play;
    let held = play.held(frame);
    let done = match play.step {
        0 => {
            if held == 1 && slot_for(world, true).is_none() {
                key(world, STARTER, true);
            }
            if held == 2 {
                key(world, STARTER, false);
            }
            if let (3 | 4, Some(slot)) = (held, slot_for(world, true)) {
                hold_slot(world, slot, held - 2);
            }
            if held == 10 && slot_for(world, true).is_none() {
                return gave_up(world, STAGE, "no javelin in the hotbar");
            }
            held >= 20
        }
        1 => {
            draws(world, held, (1, "tp"));
            held >= 140
        }
        2 => {
            if held == 1 {
                view(world, ViewMode::First, ViewMode::First);
            }
            draws(world, held, (4, "fp"));
            if held == 140 {
                view(world, ViewMode::Third, ViewMode::First);
            }
            held >= 140
        }
        3 => flight(world, play, held),
        4 => rock(world, play, held),
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
