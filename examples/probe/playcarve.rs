//! A kill and what is taken from it: an animal hunted with the held tool until it dies, its
//! carcass on the ground, parts cut from it with the interact key, and a spear pulled out of an
//! animal: shots `e55` to `e59`.

use abstractions::avatar::strike::{Act as Hands, Rest};
use abstractions::condition::{Condition, Part};
use abstractions::craft::Pack;
use abstractions::craft::matter::Grade;
use abstractions::fauna::{Beast, Dead, Herds, slay};
use abstractions::life::Species;
use abstractions::space::Clock;

use super::*;

/// The name of the stage in the log.
const STAGE: &str = "carve";

/// Returns how well a species suits a hunt that must end in a kill: a walker of some size,
/// better the lighter it is, since its health grows with its mass.
fn prey(s: &Species) -> u32 {
    let fit = walker(s) && s.body.height >= 0.25;
    let light = u32::from(s.history.mass < 150.0) + u32::from(s.history.mass < 40.0);
    (1 + light) * u32::from(fit)
}

/// Returns the first drawn carcass.
fn carcass(world: &mut World) -> Option<Entity> {
    world
        .query_filtered::<Entity, (With<Beast>, With<Dead>)>()
        .iter(world)
        .next()
}

/// Returns the animal a thrown spear is stuck in, alive or dead.
fn speared(world: &mut World) -> Option<Entity> {
    let thrown = spears(world);
    thrown
        .iter()
        .find_map(|(rest, of)| of.filter(|_| *rest == Rest::Stuck))
}

/// Kills the animal `entity` through the function a killing blow calls, for a hunt that ran out
/// of frames: the ledger, the carcass and the fall are those of a real kill.
fn finish(world: &mut World, entity: Entity) {
    let done = world.run_system_once(
        move |mut commands: Commands,
              mut beasts: Query<&mut Beast>,
              mut herds: ResMut<Herds>,
              (clock, terrain): (Res<Clock>, Res<Terrain>)| {
            if let Ok(mut beast) = beasts.get_mut(entity) {
                let dead = (entity, &mut *beast);
                let ledger = (&mut *herds, clock.seconds);
                slay(&mut commands, dead, ledger, (&*terrain, Grade::Poor));
            }
        },
    );
    if done.is_err() {
        say(STAGE, "the kill through fauna::slay could not run");
    }
}

/// The first steps: a carcass that is already there, or an animal to hunt: the one that carries
/// the spear, else one of a light species reached at its herd.
fn find_prey(world: &mut World, play: &mut Play, frame: u32) -> Option<Act> {
    let held = play.held(frame);
    let step = play.step;
    match step {
        0 if held == 2 => {
            view(world, ViewMode::Third, ViewMode::First);
            if let Some(dead) = carcass(world) {
                say(STAGE, "a carcass lies here already, from the fight");
                play.beast = Some(dead);
                play.goto(3, frame);
                return None;
            }
            if let Some(foe) = speared(world).or(play.beast.filter(|e| animal(world, *e).is_some()))
            {
                play.beast = Some(foe);
                play.species = animal(world, foe).map_or(0, |a| a.species);
                play.name = species_name(world, play.species);
                say(STAGE, &format!("hunting the {} of the fight", play.name));
                play.goto(2, frame);
                return None;
            }
            let Some(herd) = pick_herd(world, play.tries, prey) else {
                return Some(gave_up(
                    world,
                    STAGE,
                    "no herd of walkers within 3 km of the robot",
                ));
            };
            play.species = herd.species;
            play.name = species_name(world, herd.species);
            say(
                STAGE,
                &format!("hunting {}", go_to_herd(world, &herd, 30.0)),
            );
            play.next(frame);
        }
        1 if held >= 40 && !living(world, play.species).is_empty() => {
            play.beast = living(world, play.species).first().map(|found| found.0);
            play.next(frame);
        }
        1 if held > 300 => {
            play.tries += 1;
            if play.tries > 4 {
                return Some(gave_up(
                    world,
                    STAGE,
                    "five herds had no animal drawn near them",
                ));
            }
            play.goto(0, frame);
        }
        _ => {}
    }
    None
}

/// One frame of the hunt: the robot is kept in front of the animal and swings whenever its hands
/// are ready. After 1500 frames, or with the head or the torso of the robot under 30 percent,
/// the animal is killed through the function a killing blow calls, and the log says so.
fn hunt(world: &mut World, play: &mut Play, frame: u32) {
    let held = play.held(frame);
    if let Some(slot) = slot_for(world, false) {
        hold_slot(world, slot, held);
    }
    if frame == play.marks[0] + 1 {
        mouse(world, MouseButton::Left, false);
    }
    let found = play.beast.and_then(|e| Some((e, animal(world, e)?)));
    let Some((entity, a)) = found.or_else(|| living(world, play.species).first().copied()) else {
        say(
            STAGE,
            &format!("the {} is gone, looking for another herd", play.name),
        );
        play.tries += 1;
        play.goto(0, frame);
        return;
    };
    play.beast = Some(entity);
    if a.dead {
        let left = a.health * 100.0;
        say(
            STAGE,
            &format!(
                "{} killed after {} swings (health {left:.0} %)",
                play.name, play.marks[1]
            ),
        );
        play.next(frame);
        return;
    }
    let condition = *world.resource::<Condition>();
    let weak = condition.of(Part::Head) < 0.3 || condition.of(Part::Torso) < 0.3;
    if held > 1500 || weak || (held > 90 && strike_of(world).is_none()) {
        let left = a.health * 100.0;
        let swings = play.marks[1];
        say(
            STAGE,
            &format!(
                "the hunt did not end by blows ({swings} swings, {} at {left:.0} % health, robot weak: \
                 {weak}): killed through fauna::slay, the function a killing blow calls",
                play.name
            ),
        );
        finish(world, entity);
        return;
    }
    let ready = strike_of(world).is_some_and(|strike| strike.act == Hands::Ready);
    if held > 14 && ready && frame > play.marks[0] + 20 && frame > play.marks[2] + 15 {
        let want = a.at + a.facing * (0.5 * f64::from(a_length(world, a.species)) + 0.55);
        let off = (robot(world) - want).reject_from(a.at.normalize()).length();
        if off > 0.9 {
            face_animal(world, entity, 0.55);
            play.marks[2] = frame;
        } else {
            mouse(world, MouseButton::Left, true);
            (play.marks[0], play.marks[1]) = (frame, play.marks[1] + 1);
        }
    }
}

/// Returns the length in metres of species index `species`.
fn a_length(world: &World, species: u16) -> f32 {
    let life = world.resource::<Terrain>().globe().life();
    let found = life.and_then(|life| life.species.get(usize::from(species)));
    found.map_or(1.0, |s| s.body.length)
}

/// Returns the planet position of the body of the carcass `entity`: its first part, or its root.
fn body_at(world: &mut World, entity: Entity) -> Option<DVec3> {
    let drawn = animals(world);
    let part = drawn.iter().find(|a| a.0 == entity).map(|a| a.1);
    part.or(animal(world, entity).map(|a| a.at))
}

/// The carcass: seen from 3 m behind the robot, then through its eyes from 1.8 m, then three
/// presses of the interact key that each cut the part the robot looks at.
fn carve_steps(world: &mut World, play: &mut Play, frame: u32) -> Option<Act> {
    let held = play.held(frame);
    let cut = bound(world, |s| s.controls.keys.ship.board);
    let Some(at) = play.beast.and_then(|e| body_at(world, e)) else {
        return Some(gave_up(world, STAGE, "the carcass is gone"));
    };
    let up = at.normalize();
    let back = (robot(world) - at)
        .reject_from(up)
        .normalize_or(up.any_orthonormal_vector());
    let near = |world: &mut World, metres: f64| {
        let feet = ground(world, at + back * metres, 0.0);
        stand(world, feet, at, pitch_to(feet, at));
    };
    match held {
        1 => mouse(world, MouseButton::Left, false),
        60 => near(world, 3.0),
        100 => {
            let herds = world.resource::<Herds>();
            let facts = format!(
                "{} kills in the ledger, carved {:?}",
                herds.slain.len(),
                herds.carved
            );
            say(
                STAGE,
                &format!("e55 the carcass of {} on the ground: {facts}", play.name),
            );
            shot(world, "e55-carcass");
        }
        101 => {
            view(world, ViewMode::First, ViewMode::First);
            near(world, 1.8);
            play.mark = world.resource::<Pack>().mass();
        }
        130 => {
            say(
                STAGE,
                "e56 looking at the carcass from 1.8 m: the game has no prompt for the cut key",
            );
            shot(world, "e56-carcass-close");
        }
        131..165 => (131..165)
            .step_by(10)
            .for_each(|at| tap(world, cut, held, at)),
        165 => {
            let gained = world.resource::<Pack>().mass() - play.mark;
            let carved = format!("{:?}", world.resource::<Herds>().carved);
            say(
                STAGE,
                &format!(
                    "e57 four presses of the cut key put {gained:.2} kg in the pack, carved {carved}: {}",
                    pack_words(world)
                ),
            );
            shot(world, "e57-parts-taken");
            play.next(frame);
        }
        _ => {}
    }
    None
}

/// The spear: the hands emptied, the robot beside the animal the spear is stuck in, the shot,
/// then the interact key that pulls it out.
fn spear_steps(world: &mut World, play: &mut Play, frame: u32) -> Option<Act> {
    let held = play.held(frame);
    let pull = bound(world, |s| s.controls.keys.ship.board);
    let Some(holder) = speared(world)
        .filter(|_| held < 41)
        .or(play.beast.filter(|_| held >= 41))
    else {
        say(STAGE, "no spear is stuck in an animal: nothing to pull");
        tidy(world);
        return Some(Act::Next);
    };
    let free = world
        .resource::<Pack>()
        .bar
        .slots
        .iter()
        .position(Option::is_none);
    if let Some(slot) = free {
        hold_slot(world, slot, held);
    }
    match held {
        1 => play.beast = Some(holder),
        12 => {
            view(world, ViewMode::Third, ViewMode::First);
            if let Some(a) = animal(world, holder) {
                let feet = ground(world, a.at + a.facing * 1.6, 0.0);
                stand(world, feet, a.at, pitch_to(feet, a.at));
            }
        }
        40 => {
            say(
                STAGE,
                &format!(
                    "e58 a spear stuck in {}, hands empty: {}",
                    play.name,
                    pack_words(world)
                ),
            );
            shot(world, "e58-spear-stuck");
        }
        41 | 42 => tap(world, pull, held, 41),
        65 => {
            let left = spears(world);
            say(
                STAGE,
                &format!(
                    "e59 after the pull key: spears left {left:?}; {}",
                    pack_words(world)
                ),
            );
            shot(world, "e59-spear-pulled");
            tidy(world);
            return Some(Act::Next);
        }
        _ => {}
    }
    None
}

/// A hunt that ends in a kill, the carcass, parts cut from it and the spear pulled: shots `e55`
/// to `e59`.
pub fn play_carve(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        let last = notes.play.beast;
        notes.play.start(frame);
        notes.play.beast = last;
    }
    if frame > 5000 {
        return gave_up(world, STAGE, "no kill was carved in 5000 frames");
    }
    let play = &mut notes.play;
    let step = play.step;
    let ended = match step {
        0 | 1 => find_prey(world, play, frame),
        2 => {
            hunt(world, play, frame);
            None
        }
        3 => carve_steps(world, play, frame),
        _ => spear_steps(world, play, frame),
    };
    ended.unwrap_or(Act::Wait)
}
