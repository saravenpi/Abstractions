//! Stones picked from a rock, the pack that holds them, the workshop that proposes tools from
//! them, a tool made and the hotbar that holds it: shots `e20` to `e24`.

use abstractions::craft::build::{Intent, Place, mass, solve};
use abstractions::craft::matter::Stuff;
use abstractions::craft::ui::{Bench, Workshop};
use abstractions::craft::{Held, Pack};
use abstractions::world::Chunks;
use abstractions::world::rocks::RockId;
use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};

use super::*;

/// The name of the stage in the log.
const STAGE: &str = "loot";

/// The key that breaks a piece off the rock under the crosshair: `PICK_KEY` of the pack.
const PICK: KeyCode = KeyCode::KeyH;

/// The key of the dev starter tools, `avatar::strike::starter::KEY`.
pub const STARTER: KeyCode = KeyCode::F8;

/// A rock near the robot: its identity, the planet position of its middle, and the planet
/// position on the ground where the robot stands to break it.
pub struct Found {
    /// The identity of the rock.
    pub id: RockId,
    /// Planet position of its middle.
    pub mid: DVec3,
    /// Planet position on the ground where the robot stands to break it.
    pub feet: DVec3,
}

/// What the scan for a rock reads: the physics world, the loaded chunks and where things are.
pub type Scan<'w, 's> = (
    SpatialQuery<'w, 's>,
    Res<'w, Chunks>,
    Query<'w, 's, &'static GlobalTransform>,
    Res<'w, Anchor>,
    Query<'w, 's, &'static GlobalTransform, With<Player>>,
);

/// Finds the rock nearest the robot, from 2.5 m to 30 m off, by casting rays down on a grid
/// around it and asking the loaded chunks which hits are rocks, as the pick key does for the
/// one ray of the crosshair.
pub fn scan((space, chunks, placed, anchor, players): Scan) -> Option<Found> {
    let me = players.iter().next()?.translation();
    let up = abstractions::space::up(&anchor, me);
    let east = up.any_orthonormal_vector();
    let north = up.cross(east);
    let down = Dir3::new(-up).ok()?;
    let filter = SpatialQueryFilter::default();
    let mut best: Option<(f32, Vec3, Vec3, RockId)> = None;
    for (x, z) in (-40..=40).flat_map(|x| (-40..=40).map(move |z| (x, z))) {
        let over = me + (east * x as f32 + north * z as f32) * 0.75 + up * 25.0;
        let Some(hit) = space.cast_ray(over, down, 60.0, true, &filter) else {
            continue;
        };
        let point = over - up * hit.distance;
        let Ok(props) = placed.get(hit.entity) else {
            continue;
        };
        let local = props.affine().inverse().transform_point3(point);
        let Some(rock) = chunks.rock_under(hit.entity, local) else {
            continue;
        };
        let away = (point - me).reject_from(up).length();
        if away >= 2.5 && best.is_none_or(|(least, ..)| away < least) {
            best = Some((away, point, props.transform_point(rock.mid), rock.id));
        }
    }
    let (_, point, mid, id) = best?;
    let back = (me - point).reject_from(up).normalize_or_zero();
    Some(Found {
        id,
        mid: anchor.to_frame(mid),
        feet: anchor.to_frame(point + back * 1.1),
    })
}

/// Returns the kilograms of stone in the pack.
fn stone(world: &World) -> f32 {
    let stacks = world.resource::<Pack>().stacks.iter();
    stacks
        .filter(|stack| matches!(stack.stuff, Stuff::Mineral(_)))
        .map(|stack| stack.mass)
        .sum()
}

/// Looks for a rock around the robot and stands the robot at it, looking at its middle. When no
/// rock is near, the robot moves to another spot around the base and looks again next time.
/// Returns false once it has looked in twelve places.
fn seek(world: &mut World, play: &mut Play, frame: u32) -> bool {
    let found = world.run_system_once(scan).ok().flatten();
    if let Some(rock) = found {
        let feet = ground(world, rock.feet, 0.0);
        stand(world, feet, rock.mid, pitch_to(feet, rock.mid));
        let off = feet.distance(rock.mid);
        say(
            STAGE,
            &format!(
                "rock {} found, standing {off:.1} m from its middle",
                rock.id.text()
            ),
        );
        play.mark = stone(world);
        play.next(frame);
        return true;
    }
    play.tries += 1;
    let Some(base) = world.get_resource::<Base>().copied() else {
        return false;
    };
    let ways = compass(base.wake, 8);
    let out = 45.0 * f64::from(1 + play.tries / 8);
    let feet = ground(
        world,
        base.wake + ways[(play.tries % 8) as usize] * out,
        0.0,
    );
    stand(world, feet, base.centre, 0.0);
    say(
        STAGE,
        &format!("no rock within 30 m, looking {out:.0} m from the wake point"),
    );
    play.goto(0, frame);
    play.tries <= 12
}

/// Presses the pick key four times on the rock the robot looks at, then says what the pack
/// gained. Returns false when it gained nothing and the search must start again.
fn pick(world: &mut World, play: &mut Play, frame: u32) -> bool {
    let held = play.held(frame);
    for at in [20, 32, 44, 56] {
        tap(world, PICK, held, at);
    }
    if held < 72 {
        return true;
    }
    let gained = stone(world) - play.mark;
    if gained <= 0.0 {
        play.tries += 1;
        say(
            STAGE,
            "four presses of H took no stone from that rock, looking for another",
        );
        play.goto(0, frame);
        return play.tries <= 12;
    }
    say(
        STAGE,
        &format!(
            "e20 four presses of H broke {gained:.2} kg off the rock: {}",
            pack_words(world)
        ),
    );
    shot(world, "e20-rock-picked");
    play.next(frame);
    true
}

/// Returns what the workshop proposes for the intent of its bench from what the pack holds, in
/// words: the masses of its three proposals, or why it proposes nothing.
pub fn offer_words(world: &World) -> String {
    let bench = world.resource::<Bench>();
    let globe = world.resource::<Terrain>().globe();
    let place = Place::on(globe.gravity(), globe.record());
    let stock = world.resource::<Pack>().stock();
    let said = match solve(bench.intent, &stock, &place) {
        Ok(three) => format!(
            "three proposals: lightest {:.2} kg, hardest hitting {:.2} kg, most durable {:.2} kg",
            mass(&three.lightest.build.pieces()),
            mass(&three.hardest.build.pieces()),
            mass(&three.durable.build.pieces())
        ),
        Err(why) => format!("no proposal: {why}"),
    };
    format!(
        "workshop pane {:?}, intent {:?}, pick {}, {said}; note \"{}\"",
        bench.pane, bench.intent, bench.pick, bench.note
    )
}

/// The pack opened on its key with the stones in it, then the workshop on Tab with its three
/// proposals, and the chosen one made on Enter.
fn bench_steps(world: &mut World, play: &mut Play, frame: u32) -> Option<Act> {
    let held = play.held(frame);
    let pack_key = bound(world, |s| s.controls.keys.pack);
    let open = *world.resource::<State<Workshop>>().get() == Workshop::Open;
    let step = play.step;
    match step {
        2 => {
            press(world, pack_key, held);
            if held == 50 && !open {
                return Some(gave_up(world, STAGE, "the pack key did not open the pack"));
            }
            if held == 50 {
                say(STAGE, &format!("e21 the pack: {}", pack_words(world)));
                shot(world, "e21-pack-stones");
                play.next(frame);
            }
        }
        3 => {
            press(world, KeyCode::Tab, held);
            if held == 40 {
                say(STAGE, &format!("e22 {}", offer_words(world)));
                shot(world, "e22-workshop-cut");
                play.next(frame);
            }
        }
        4 => {
            press(world, KeyCode::Enter, held);
            if held == 12 {
                say(
                    STAGE,
                    &format!("Enter in the workshop: {}", offer_words(world)),
                );
                play.next(frame);
            }
        }
        _ => {}
    }
    None
}

/// The workshop asked for a tool to throw and Enter pressed on it, the pack closed on its key,
/// then the robot seen from behind with the hotbar and the tool it holds.
fn hotbar_steps(world: &mut World, play: &mut Play, frame: u32) -> Option<Act> {
    let held = play.held(frame);
    let pack_key = bound(world, |s| s.controls.keys.pack);
    let step = play.step;
    match step {
        5 => {
            for at in [1, 9, 17] {
                tap(world, KeyCode::ArrowRight, held, at);
            }
            tap(world, KeyCode::Enter, held, 40);
            if held == 30 {
                say(STAGE, &format!("e23 {}", offer_words(world)));
                shot(world, "e23-workshop-throw");
            }
            if held == 55 {
                let intent = world.resource::<Bench>().intent;
                let asked = intent == Intent::Throw;
                say(
                    STAGE,
                    &format!("asked to throw: {asked}; {}", offer_words(world)),
                );
                play.next(frame);
            }
        }
        6 => {
            press(world, pack_key, held);
            let bare = world.resource::<Pack>().items.is_empty();
            if held == 20 && bare {
                say(
                    STAGE,
                    "the workshop made no tool from the stones: pressing the dev starter key F8",
                );
                key(world, STARTER, true);
            }
            if held == 21 {
                key(world, STARTER, false);
                view(world, ViewMode::Third, ViewMode::First);
                play.next(frame);
            }
        }
        7 if held == 75 => {
            let hand = world.resource::<Held>().0.map(|hold| hold.intent);
            say(
                STAGE,
                &format!(
                    "e24 the hotbar, tool in hand for {hand:?}: {}",
                    pack_words(world)
                ),
            );
            shot(world, "e24-hotbar-tool");
            return Some(Act::Next);
        }
        _ => {}
    }
    None
}

/// Stones picked from a rock with H, the pack, the workshop and its proposals, a tool made and
/// held: shots `e20` to `e24`.
pub fn play_loot(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        notes.play.start(frame);
        view(world, ViewMode::First, ViewMode::First);
    }
    if frame > 3000 {
        return gave_up(
            world,
            STAGE,
            "no stone was picked and no tool made in 3000 frames",
        );
    }
    let play = &mut notes.play;
    let step = play.step;
    let going = match step {
        0 if play.held(frame) < 45 => true,
        0 => seek(world, play, frame),
        1 => pick(world, play, frame),
        2..=4 => return bench_steps(world, play, frame).unwrap_or(Act::Wait),
        _ => return hotbar_steps(world, play, frame).unwrap_or(Act::Wait),
    };
    match going {
        true => Act::Wait,
        false => gave_up(
            world,
            STAGE,
            "no rock gave stone in twelve places around the base",
        ),
    }
}
