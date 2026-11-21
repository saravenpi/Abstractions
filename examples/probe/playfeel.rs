//! How a blow feels and reads: a swing at a small animal in third person and in first person, its
//! wind-up, contact and follow-through, the hit mark and the animal fought; a blow on a large
//! animal; a rock struck with H, its chips and the notice; and the wingsuit opened with a tool
//! drawn, the tool slung over the back: shots `f60` to `f72`. Run it on seed 57.

use abstractions::avatar::strike::{Act as Hands, Knap, Mark, Sight};
use abstractions::life::Species;

use super::*;

/// The name of the stage in the log.
const STAGE: &str = "feel";

/// The key that breaks a piece off the rock under the crosshair: `PICK_KEY` of the pack.
const KNOCK: KeyCode = KeyCode::KeyH;

/// Returns how well a species suits the small target: a walker under 40 cm at the hip.
fn small(s: &Species) -> u32 {
    u32::from(walker(s)) * (1 + 2 * u32::from(s.body.height < 0.4))
}

/// Returns how well a species suits the large target: a walker over 80 cm at the hip.
fn large(s: &Species) -> u32 {
    u32::from(walker(s)) * (1 + 2 * u32::from(big_walker(s)))
}

/// Logs what the crosshair and the last blow show.
fn read(world: &World, what: &str) {
    let sight = *world.resource::<Sight>();
    let mark = *world.resource::<Mark>();
    say(
        STAGE,
        &format!(
            "{what}: target {:?} in reach {}, aim {:?}; mark {:?} {:.2} s ago, fight {:.1} s",
            sight.target, sight.reach, sight.point, mark.outcome, mark.age, mark.fight
        ),
    );
}

/// Takes the shots of one swing, each once by the bits from `bit`: the wind-up, the contact, the
/// follow-through and the hit mark, numbered from `first` with `tag`.
fn swing_shots(world: &mut World, play: &mut Play, (bit, first, tag): (u32, u32, &str)) {
    let Some(strike) = strike_of(world) else {
        return;
    };
    let names = ["windup", "contact", "through"];
    let phase = match strike.act {
        Hands::Windup(_) => Some(0),
        Hands::Swing(t) if t > 0.02 || strike.landed => Some(1),
        Hands::Recover(_) | Hands::Stun(..) => Some(2),
        _ => None,
    };
    if let Some(k) = phase
        && play.once(bit + k as u32)
    {
        read(world, &format!("f{} {tag} {}", first + k as u32, names[k]));
        shot(world, &format!("f{}-{tag}-{}", first + k as u32, names[k]));
    }
    if world.resource::<Mark>().shown().is_some() && play.once(bit + 3) {
        read(world, &format!("f{} {tag} hit mark", first + 3));
        shot(world, &format!("f{}-{tag}-hit-mark", first + 3));
    }
}

/// Finds a herd scored by `score` and stands the robot near it; returns false when there is none.
fn to_herd(world: &mut World, play: &mut Play, score: fn(&Species) -> u32) -> bool {
    let Some(herd) = pick_herd(world, 0, score) else {
        return false;
    };
    play.species = herd.species;
    play.name = species_name(world, herd.species);
    say(STAGE, &format!("using {}", go_to_herd(world, &herd, 30.0)));
    true
}

/// The steps of a swing at an animal of the herd: an animal drawn, the tool drawn, the robot put in
/// front of it, a click, the shots. Returns true once the step is over.
fn swing_at(world: &mut World, play: &mut Play, frame: u32, shots: (u32, u32, &str)) -> bool {
    let held = play.held(frame);
    if held < 40 {
        return false;
    }
    if held == 40 {
        play.beast = living(world, play.species).first().map(|found| found.0);
    }
    if let (40 | 41, Some(slot)) = (held, slot_for(world, false)) {
        hold_slot(world, slot, held - 39);
    }
    if held == 50
        && let Some(beast) = play.beast
    {
        face_animal(world, beast, 0.35);
    }
    match held {
        70 => mouse(world, MouseButton::Left, true),
        72 => mouse(world, MouseButton::Left, false),
        _ => {}
    }
    if held > 70 {
        swing_shots(world, play, shots);
    }
    held >= 160
}

/// The rock: found and stood at, struck with H, and the shots of the wind-up, the contact with its
/// chips and the notice on the prompt line. Returns true once the step is over.
fn knock(world: &mut World, play: &mut Play, frame: u32) -> bool {
    let held = play.held(frame);
    if held == 5 {
        let found = world.run_system_once(scan).ok().flatten();
        let Some(rock) = found else {
            say(STAGE, "no rock within 30 m: no stone is struck");
            return true;
        };
        let feet = ground(world, rock.feet, 0.0);
        stand(world, feet, rock.mid, pitch_to(feet, rock.mid));
    }
    match held {
        40 => key(world, KNOCK, true),
        41 => key(world, KNOCK, false),
        _ => {}
    }
    let knap = world.query::<&Knap>().iter(world).next().copied();
    if knap.is_some_and(|k| k.t > 0.12 && !k.landed) && play.once(8) {
        shot(world, "f68-knap-windup");
    }
    if knap.is_some_and(|k| k.landed) && play.once(9) {
        play.marks[0] = held;
        shot(world, "f69-knap-contact");
    }
    if play.marks[0] > 0 && held == play.marks[0] + 20 && play.once(10) {
        say(
            STAGE,
            &format!("f70 after the knock: {}", pack_words(world)),
        );
        shot(world, "f70-knap-notice");
    }
    held >= 140
}

/// The glider: the robot lifted high with its tool drawn, the wingsuit opened, the stow and the
/// glide with the tool slung. Returns true once the step is over.
fn glide(world: &mut World, play: &mut Play, frame: u32) -> bool {
    let held = play.held(frame);
    if held == 1 {
        let base = world.get_resource::<Base>().map_or(DVec3::Y, |b| b.centre);
        let side = base.normalize().cross(DVec3::Y).normalize_or(DVec3::X);
        let feet = ground(world, base - side * 160.0, 160.0);
        stand(world, feet, ground(world, base, 160.0), -0.2);
    }
    if held > 4 {
        spread(world);
    }
    let stowing = strike_of(world).is_some_and(|s| matches!(s.act, Hands::Stow(t) if t > 0.1));
    if stowing && play.once(11) {
        shot(world, "f71-glider-stow");
    }
    if held == 70 {
        let out = strike_of(world).map(|s| s.act);
        say(
            STAGE,
            &format!("f72 gliding, the hands: {out:?}; {}", pack_words(world)),
        );
        shot(world, "f72-glider-slung");
    }
    held >= 80
}

/// A swing at a small and a large animal in third and first person, a rock struck and the glider
/// opened with a tool drawn: shots `f60` to `f72`.
pub fn play_feel(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        notes.play.start(frame);
        noon(world);
        view(world, ViewMode::Third, ViewMode::First);
    }
    if frame > 4000 {
        return gave_up(world, STAGE, "the stage did not end in 4000 frames");
    }
    let play = &mut notes.play;
    let held = play.held(frame);
    let done = match play.step {
        0 => {
            if held == 1 && slot_for(world, false).is_none() {
                key(world, STARTER, true);
            }
            if held == 2 {
                key(world, STARTER, false);
            }
            if held == 10 && !to_herd(world, play, small) {
                return gave_up(world, STAGE, "no herd of walkers within 3 km of the robot");
            }
            held >= 10
        }
        1 => swing_at(world, play, frame, (0, 60, "tp")),
        2 => {
            if held == 1 {
                view(world, ViewMode::First, ViewMode::First);
            }
            swing_at(world, play, frame, (4, 64, "fp"))
        }
        3 => {
            if held == 1 {
                view(world, ViewMode::Third, ViewMode::First);
                if !to_herd(world, play, large) {
                    say(
                        STAGE,
                        "no herd of large walkers: the large target is skipped",
                    );
                    play.goto(4, frame);
                    return Act::Wait;
                }
            }
            swing_at(world, play, frame, (12, 56, "large"))
        }
        4 => knock(world, play, frame),
        5 => glide(world, play, frame),
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
