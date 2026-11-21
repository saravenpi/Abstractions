//! A fight: a spear thrown at an animal that stands its ground, then swings of the held tool on
//! it while it winds up and strikes back, and the robot after the fight: shots `e40` to `e48`.
//!
//! The robot is not protected: the animal's blows land on it. The fight ends once the robot has
//! landed a blow and taken one, or when its head or torso falls under 30 percent, so the body is
//! not destroyed mid-stage.

use abstractions::avatar::strike::{Act as Hands, Rest, Spear, Strike};
use abstractions::condition::{Condition, Part};
use abstractions::craft::Pack;
use abstractions::craft::build::Intent;
use abstractions::craft::wield::KEYS;
use abstractions::fauna::bleed::Wound;
use abstractions::fauna::senses::stance::plant_eater;
use abstractions::fauna::{Beast, Broken, Kind};
use abstractions::life::Species;

use super::*;

/// The name of the stage in the log.
const STAGE: &str = "fight";

/// The seconds the blow of an animal takes, `STRIKE` of its attacks.
const STRIKE: f32 = 0.18;

/// Returns the share of the torso of the robot the hardest blow of `s` would take, from the
/// attacks its body has.
fn worst_blow(s: &Species) -> f32 {
    let harm = |kind: Kind| {
        let speed = match kind.dashes() {
            true => s.history.speed,
            false => kind.reach(&s.body) / STRIKE,
        };
        let through = if kind.cuts() { 1.0 } else { 0.6 };
        0.5 * s.history.mass * kind.share() * speed * speed * through / 2400.0
    };
    let able = Kind::ORDER
        .into_iter()
        .filter(|kind| kind.can(s, Broken::default()));
    able.map(harm).fold(0.0, f32::max)
}

/// Returns how well a species suits the fight: 0 for one that is not a walker of some size, 1
/// for one that flees, 2 for one that stands its ground against a robot, 3 when its hardest blow
/// also leaves the torso over 40 percent.
pub fn fighter(s: &Species) -> u32 {
    if !walker(s) || s.body.height < 0.4 {
        return 0;
    }
    let heavy = match plant_eater(s.role.guild) {
        true => s.history.mass > 160.0,
        false => s.history.mass > 40.0,
    };
    let worst = worst_blow(s);
    1 + u32::from(heavy && worst > 0.0) + u32::from(heavy && worst > 0.0 && worst < 0.6)
}

/// Returns the length and the hip height of species index `species`, in metres.
fn size_of(world: &World, species: u16) -> (f64, f64) {
    let life = world.resource::<Terrain>().globe().life();
    let found = life.and_then(|life| life.species.get(usize::from(species)));
    found.map_or((1.0, 0.5), |s| {
        (f64::from(s.body.length), f64::from(s.body.height))
    })
}

/// Returns the metres from the root of an animal of `species` to where the robot stands in
/// front of it, `extra` metres past its nose.
fn gap_of(world: &World, species: u16, extra: f64) -> f64 {
    0.5 * size_of(world, species).0 + extra
}

/// Stands the robot in front of the animal `entity`, `extra` metres past its nose, looking at
/// its body. Returns false once the animal is gone.
pub fn face_animal(world: &mut World, entity: Entity, extra: f64) -> bool {
    let Some(a) = animal(world, entity) else {
        return false;
    };
    let gap = gap_of(world, a.species, extra);
    let feet = ground(world, a.at + a.facing * gap, 0.0);
    let aim = a.at + a.at.normalize() * size_of(world, a.species).1;
    stand(world, feet, aim, pitch_to(feet, aim));
    true
}

/// Stands the robot `metres` to the side of the animal `a`, looking at its body.
fn beside(world: &mut World, a: &Animal, metres: f64) {
    let up = a.at.normalize();
    let side = a.facing.cross(up).normalize_or(a.facing);
    let feet = ground(world, a.at + side * metres, 0.0);
    let aim = a.at + up * size_of(world, a.species).1;
    stand(world, feet, aim, pitch_to(feet, aim));
}

/// Returns the hotbar slot of the first tool that is thrown, or of the first that is swung.
pub fn slot_for(world: &World, thrown: bool) -> Option<usize> {
    let pack = world.resource::<Pack>();
    let fits = |id: u32| {
        pack.item(id)
            .is_some_and(|item| (item.intent() == Intent::Throw) == thrown)
    };
    pack.bar
        .slots
        .iter()
        .position(|slot| slot.is_some_and(fits))
}

/// Holds hotbar slot `slot` by its number key over the first two frames of a step. The key of
/// the slot already held is not pressed: it would stow the tool.
pub fn hold_slot(world: &mut World, slot: usize, held: u32) {
    let now = world.resource::<Pack>().bar.held;
    match held {
        1 if now != slot => key(world, KEYS[slot], true),
        2 => key(world, KEYS[slot], false),
        _ => {}
    }
}

/// Returns every thrown spear: where it is, and the animal it is stuck in.
pub fn spears(world: &mut World) -> Vec<(Rest, Option<Entity>)> {
    let found: Vec<(Rest, Option<Entity>)> = world
        .query::<(&Spear, Option<&ChildOf>)>()
        .iter(world)
        .map(|(spear, of)| (spear.rest, of.map(ChildOf::parent)))
        .collect();
    let stuck = |of: Option<Entity>| of.filter(|e| world.get::<Beast>(*e).is_some());
    found
        .into_iter()
        .map(|(rest, of)| (rest, stuck(of)))
        .collect()
}

/// The first steps: a tool in the pack, a herd that stands its ground reached, and one of its
/// animals drawn.
fn find_foe(world: &mut World, play: &mut Play, frame: u32) -> Option<Act> {
    let held = play.held(frame);
    let step = play.step;
    match step {
        0 if held == 2 => {
            noon(world);
            view(world, ViewMode::Third, ViewMode::First);
            if world.resource::<Pack>().items.is_empty() {
                return Some(gave_up(
                    world,
                    STAGE,
                    "the pack holds no tool to fight with",
                ));
            }
            let Some(herd) = pick_herd(world, play.tries, fighter) else {
                return Some(gave_up(
                    world,
                    STAGE,
                    "no herd of walkers within 3 km of the robot",
                ));
            };
            play.species = herd.species;
            play.name = species_name(world, herd.species);
            say(STAGE, &format!("using {}", go_to_herd(world, &herd, 40.0)));
            play.next(frame);
        }
        1 if held >= 40 && !living(world, play.species).is_empty() => {
            play.beast = living(world, play.species).first().map(|found| found.0);
            let globe = world.resource::<Terrain>().globe().clone();
            let s = globe
                .life()
                .map(|life| &life.species[usize::from(play.species)]);
            let facts = s.map(|s| (fighter(s), worst_blow(s) * 100.0));
            say(
                STAGE,
                &format!(
                    "{} drawn; (fight score, hardest blow as % of the torso) {facts:?}",
                    play.name
                ),
            );
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

/// The throw: the thrown tool held, the robot 7 m to the side of the animal, the primary button
/// held for a second and let go, and what became of the spear.
fn throw_steps(world: &mut World, play: &mut Play, frame: u32) {
    let held = play.held(frame);
    if play.step == 3 {
        land_spear(world, play, frame);
        return;
    }
    if held == 1 && (slot_for(world, true).is_none() || play.beast.is_none()) {
        say(
            STAGE,
            "no tool to throw in the hotbar, or no animal: no spear is thrown",
        );
        play.goto(4, frame);
        return;
    }
    if held <= 2
        && let Some(slot) = slot_for(world, true)
    {
        hold_slot(world, slot, held);
    }
    match held {
        12 => {
            if let Some(a) = play.beast.and_then(|foe| animal(world, foe)) {
                beside(world, &a, 7.0);
            }
        }
        30 => mouse(world, MouseButton::Left, true),
        62 => mouse(world, MouseButton::Left, false),
        66 => {
            say(
                STAGE,
                &format!(
                    "e40 a spear drawn for a second and thrown at {} from 7 m",
                    play.name
                ),
            );
            shot(world, "e40-spear-thrown");
            play.next(frame);
        }
        _ => {}
    }
}

/// What became of the thrown spear, 2.5 s after it left: the robot is put beside the animal it
/// is stuck in, and the shot is taken.
fn land_spear(world: &mut World, play: &mut Play, frame: u32) {
    let held = play.held(frame);
    let thrown = spears(world);
    let stuck = thrown
        .iter()
        .find_map(|(rest, of)| of.filter(|_| *rest == Rest::Stuck));
    if held == 60
        && let Some(a) = stuck.and_then(|entity| animal(world, entity))
    {
        beside(world, &a, 4.0);
    }
    if held < 75 {
        return;
    }
    play.beast = stuck.or(play.beast);
    let wounds = world.query::<&Wound>().iter(world).count();
    let health = stuck
        .and_then(|e| animal(world, e))
        .map(|a| a.health * 100.0);
    say(
        STAGE,
        &format!(
            "e41 spears {thrown:?}; stuck in an animal: {}; animals with an open wound: {wounds}; its \
             health {health:?} %",
            stuck.is_some()
        ),
    );
    shot(world, "e41-spear-landed");
    play.next(frame);
}

/// Takes the shots of the duel, each once: the swing, the blow that landed, the blood, the
/// wind-up and the strike of the animal, and the robot struck.
fn duel_shots(world: &mut World, play: &mut Play, frame: u32, (a, strike): (&Animal, &Strike)) {
    if matches!(strike.act, Hands::Swing(t) if t > 0.03) && play.once(1) {
        shot(world, "e42-swing");
    }
    if strike.landed && play.once(2) {
        let facts = format!("{:?}, its health {:.0} %", strike.result, a.health * 100.0);
        say(
            STAGE,
            &format!("e43 a swing landed on {}: {facts}", play.name),
        );
        shot(world, "e43-blow-landed");
        play.marks[3] = frame;
    }
    let bleeding = play.beast.is_some_and(|e| world.get::<Wound>(e).is_some());
    if bleeding && play.done & 4 != 0 && frame > play.marks[3] + 8 && play.once(3) {
        say(STAGE, "e44 the animal has an open wound and bleeds");
        shot(world, "e44-blood");
    }
    if let Some(attack) = a.attack {
        let winding = !attack.striking() && attack.t > 0.4 * attack.wind && attack.t < attack.wind;
        if winding && play.once(4) {
            say(
                STAGE,
                &format!(
                    "e45 {} winds up a {:?} for {:.2} s",
                    play.name, attack.kind, attack.wind
                ),
            );
            shot(world, "e45-animal-windup");
        }
        if attack.striking() && play.once(5) {
            shot(world, "e46-animal-strike");
        }
    }
    if play.marks[2] > 0 && frame == play.marks[2] + 6 && play.once(7) {
        shot(world, "e47-robot-struck");
    }
}

/// Notes a blow the robot took since the last frame: the frame it landed on and what it left.
fn feel(world: &World, play: &mut Play, frame: u32) -> Condition {
    let condition = *world.resource::<Condition>();
    if condition.integrity() < play.mark - 1e-4 {
        if play.once(6) {
            play.marks[2] = frame;
        }
        play.count += 1;
        say(
            STAGE,
            &format!("the robot was struck: {}", frame_words(world)),
        );
    }
    play.mark = condition.integrity();
    condition
}

/// One frame of the duel: the robot is kept in front of the animal and swings whenever its hands
/// are ready. Returns why the duel is over, once it is.
fn duel(world: &mut World, play: &mut Play, frame: u32) -> Option<String> {
    let Some((entity, a)) = play.beast.and_then(|e| Some((e, animal(world, e)?))) else {
        return Some("the animal is gone".to_string());
    };
    let Some(strike) = strike_of(world) else {
        return Some("the robot has no tool in its hands".to_string());
    };
    duel_shots(world, play, frame, (&a, &strike));
    let condition = feel(world, play, frame);
    if frame == play.marks[0] + 1 {
        mouse(world, MouseButton::Left, false);
    }
    if a.dead {
        return Some("the animal died".to_string());
    }
    if condition.of(Part::Head) < 0.3 || condition.of(Part::Torso) < 0.3 {
        return Some("the head or the torso of the robot is under 30 %".to_string());
    }
    let both = play.done & 4 != 0 && play.marks[2] > 0 && frame > play.marks[2] + 45;
    if both || play.held(frame) > 1400 {
        return Some(format!(
            "{} swings, {} blows taken",
            play.marks[1], play.count
        ));
    }
    let rested = frame > play.marks[0] + 20 && frame > play.tries + 15;
    if strike.act == Hands::Ready && rested {
        let want = a.at + a.facing * gap_of(world, a.species, 0.55);
        let off = (robot(world) - want).reject_from(a.at.normalize()).length();
        if off > 0.9 {
            face_animal(world, entity, 0.55);
            play.tries = frame;
        } else {
            mouse(world, MouseButton::Left, true);
            (play.marks[0], play.marks[1]) = (frame, play.marks[1] + 1);
        }
    }
    None
}

/// The melee: the swung tool held, the robot put in front of the animal, the duel, then the
/// robot 30 m away with what the fight left of it.
fn melee_steps(world: &mut World, play: &mut Play, frame: u32) -> Option<Act> {
    let held = play.held(frame);
    let step = play.step;
    match step {
        4 => {
            let Some(slot) = slot_for(world, false) else {
                say(STAGE, "no tool to swing in the hotbar");
                play.goto(6, frame);
                return None;
            };
            hold_slot(world, slot, held);
            if held == 14 {
                play.beast = play.beast.filter(|e| animal(world, *e).is_some());
                play.beast = play
                    .beast
                    .or(living(world, play.species).first().map(|f| f.0));
                play.mark = world.resource::<Condition>().integrity();
                (play.marks, play.tries) = ([0; 4], frame);
                play.next(frame);
            }
        }
        5 => {
            if let Some(why) = duel(world, play, frame) {
                say(
                    STAGE,
                    &format!("the duel with {} is over: {why}", play.name),
                );
                play.goto(6, frame);
            }
        }
        _ => return after(world, play, frame),
    }
    None
}

/// After the duel: the buttons let go, the robot 30 m from the animal, and the shot of the robot
/// with the damage it carries and the frame rows of the HUD.
fn after(world: &mut World, play: &mut Play, frame: u32) -> Option<Act> {
    let held = play.held(frame);
    if held == 1 {
        mouse(world, MouseButton::Left, false);
        if let Some(a) = play.beast.and_then(|e| animal(world, e)) {
            let me = robot(world);
            let away = (me - a.at)
                .reject_from(a.at.normalize())
                .normalize_or(a.facing);
            let feet = ground(world, a.at + away * 30.0, 0.0);
            stand(world, feet, a.at, 0.0);
        }
    }
    if held < 50 {
        return None;
    }
    say(
        STAGE,
        &format!("e48 after the fight: {}", frame_words(world)),
    );
    shot(world, "e48-after-fight");
    tidy(world);
    Some(Act::Next)
}

/// A spear thrown and a duel with an animal that stands its ground: shots `e40` to `e48`.
pub fn play_fight(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        notes.play.start(frame);
    }
    if frame > 4500 {
        return gave_up(world, STAGE, "the fight did not end in 4500 frames");
    }
    let play = &mut notes.play;
    let step = play.step;
    let ended = match step {
        0 | 1 => find_foe(world, play, frame),
        2 | 3 => {
            throw_steps(world, play, frame);
            None
        }
        _ => melee_steps(world, play, frame),
    };
    ended.unwrap_or(Act::Wait)
}
