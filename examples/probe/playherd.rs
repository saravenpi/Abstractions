//! A herd from close: the robot crouched downwind of it, seen from behind, then standing up and
//! walking at it while the herd reacts: shots `e30` and `e31`.

use abstractions::fauna::Nearby;
use abstractions::fauna::senses::stance::plant_eater;
use abstractions::life::Species;
use abstractions::life::herd::Herd;

use super::*;

/// The name of the stage in the log.
const STAGE: &str = "herd";

/// Returns the herd within 3 km of the robot that `score` ranks highest, the nearest among
/// equals, leaving out the `skip` best: a stage that could not use a herd asks for the next.
/// A species that scores 0 is never taken.
pub fn pick_herd(world: &mut World, skip: u32, score: impl Fn(&Species) -> u32) -> Option<Herd> {
    let herds = herds_around(world, 3000.0);
    let globe = world.resource::<Terrain>().globe().clone();
    let life = globe.life()?;
    let rank = |herd: &Herd| score(&life.species[usize::from(herd.species)]);
    let mut ranked: Vec<(u32, usize)> = herds
        .iter()
        .enumerate()
        .map(|(at, herd)| (rank(herd), at))
        .filter(|(points, _)| *points > 0)
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let (_, at) = *ranked.get(skip as usize)?;
    Some(herds[at])
}

/// Stands the robot `metres` from the centre of `herd`, on ground level with it, looking at it,
/// and returns the herd in words.
pub fn go_to_herd(world: &mut World, herd: &Herd, metres: f64) -> String {
    let globe = world.resource::<Terrain>().globe().clone();
    let me = robot(world);
    let at = ground_at(world, herd.centre * globe.radius());
    let feet = level_spot(world, at, me, metres);
    stand(world, feet, at, -0.05);
    let words = globe.life().map(|life| {
        let s = &life.species[usize::from(herd.species)];
        format!(
            "{} ({:?}, {:.0} kg, {:.1} m long, hip {:.2} m), herd of {} {:?}",
            s.name,
            s.role.guild,
            s.history.mass,
            s.body.length,
            s.body.height,
            herd.count,
            herd.state
        )
    });
    format!(
        "{}, {:.0} m from where the robot was; now standing {metres:.0} m from its centre",
        words.unwrap_or_default(),
        at.distance(me)
    )
}

/// Returns the name of species index `species` on the planet underfoot.
pub fn species_name(world: &World, species: u16) -> String {
    let life = world.resource::<Terrain>().globe().life();
    let found = life.and_then(|life| life.species.get(usize::from(species)));
    found.map_or("?".to_string(), |s| s.name.clone())
}

/// Returns how well a species suits a shot of a grazing herd: a walker the compass marks, better
/// when it is large, best when it eats plants.
fn grazer(s: &Species) -> u32 {
    let marked = walker(s) && s.body.length >= 0.4;
    let points =
        1 + u32::from(big_walker(s)) + u32::from(big_walker(s) && plant_eater(s.role.guild));
    points * u32::from(marked)
}

/// Returns the living animals of the herd of the stage in words: how many, how many in view,
/// how far the nearest and the farthest are, and what each does about the robot.
fn herd_words(world: &mut World, species: u16) -> (String, f64) {
    let me = robot(world);
    let members = living(world, species);
    let far = |a: &(Entity, Animal)| a.1.at.distance(me);
    let nearest = members.first().map_or(f64::MAX, far);
    let farthest = members.last().map_or(0.0, far);
    let stances: Vec<String> = members
        .iter()
        .map(|(_, a)| format!("{:?}", a.stance))
        .collect();
    let crouch = world
        .query_filtered::<&Motion, With<Player>>()
        .iter(world)
        .next()
        .map_or(0.0, |motion| motion.crouch);
    let marks: Vec<String> = world
        .resource::<Nearby>()
        .herds
        .iter()
        .map(|seen| format!("{} {:.0} m", seen.name, seen.metres))
        .collect();
    let words = format!(
        "{} alive and drawn, from {nearest:.0} m to {farthest:.0} m, {} animals in view, stances \
         [{}], robot crouch {crouch:.2}, herd marks of the compass [{}]",
        members.len(),
        in_view(world),
        stances.join(" "),
        marks.join(", ")
    );
    (words, nearest)
}

/// Puts the robot 20 m downwind of the nearest animal of the herd, crouched, looking at it,
/// seen from behind. Returns false while no animal of the herd is drawn yet.
fn stalk(world: &mut World, play: &mut Play) -> bool {
    let Some((entity, near)) = living(world, play.species).first().copied() else {
        return false;
    };
    let me = robot(world);
    let wind = wind_way(world);
    let up = near.at.normalize();
    let back = (me - near.at).reject_from(up).normalize_or(near.facing);
    let feet = ground(world, near.at + wind.unwrap_or(back) * 20.0, 0.0);
    view(world, ViewMode::Third, ViewMode::First);
    stand(world, feet, near.at, pitch_to(feet, near.at));
    let crouch = bound(world, |s| s.controls.keys.crouch);
    key(world, crouch, true);
    play.beast = Some(entity);
    say(
        STAGE,
        &format!(
            "robot put 20 m downwind of the nearest animal (wind known: {}), crouch held",
            wind.is_some()
        ),
    );
    true
}

/// The first steps: a herd chosen and reached, then the robot put downwind of it once its
/// animals are drawn.
fn approach(world: &mut World, play: &mut Play, frame: u32) -> Option<Act> {
    let held = play.held(frame);
    let step = play.step;
    match step {
        0 if held == 2 => {
            noon(world);
            let Some(herd) = pick_herd(world, play.tries, grazer) else {
                return Some(gave_up(
                    world,
                    STAGE,
                    "no herd of walkers within 3 km of the robot",
                ));
            };
            play.species = herd.species;
            play.name = species_name(world, herd.species);
            say(STAGE, &format!("using {}", go_to_herd(world, &herd, 60.0)));
            play.next(frame);
        }
        1 if held >= 40 && stalk(world, play) => play.next(frame),
        1 if held > 300 => {
            play.tries += 1;
            say(
                STAGE,
                &format!("no {} was drawn near its herd in 300 frames", play.name),
            );
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

/// The last steps: the shot of the crouched robot and the calm herd, then the robot stands,
/// walks at the herd, and the shot of the herd reacting.
fn watch(world: &mut World, play: &mut Play, frame: u32) -> Option<Act> {
    let held = play.held(frame);
    let keys = world.resource::<Persistent<Settings>>().controls.keys;
    let step = play.step;
    match step {
        2 if held == 90 => {
            let (words, nearest) = herd_words(world, play.species);
            if nearest > 35.0 && play.once(0) {
                say(
                    STAGE,
                    &format!("the herd moved off ({nearest:.0} m), moving the robot again"),
                );
                play.goto(1, frame);
                return None;
            }
            say(
                STAGE,
                &format!("e30 crouched downwind of {}: {words}", play.name),
            );
            shot(world, "e30-herd-crouched");
            play.next(frame);
        }
        3 => {
            match held {
                1 => key(world, keys.crouch, false),
                30 => key(world, keys.forward, true),
                80 => key(world, keys.forward, false),
                _ => {}
            }
            if held == 95 {
                let (words, _) = herd_words(world, play.species);
                say(
                    STAGE,
                    &format!("e31 standing and walking at {}: {words}", play.name),
                );
                shot(world, "e31-herd-alarmed");
                tidy(world);
                return Some(Act::Next);
            }
        }
        _ => {}
    }
    None
}

/// A grazing herd from 20 m, the robot crouched downwind and seen from behind, then standing
/// and walking at it: shots `e30` and `e31`.
pub fn play_herd(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        notes.play.start(frame);
    }
    if frame > 2500 {
        return gave_up(world, STAGE, "no herd was shown in 2500 frames");
    }
    let play = &mut notes.play;
    let step = play.step;
    let ended = match step {
        0 | 1 => approach(world, play, frame),
        _ => watch(world, play, frame),
    };
    ended.unwrap_or(Act::Wait)
}
