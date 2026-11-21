//! The screens after the hunt: the pack with what was taken, the workshop with it, the field
//! guide with the species met, and the pause menu: shots `e70` to `e80`.

use abstractions::craft::ui::{Bench, Tab as BenchTab, Workshop};
use abstractions::fauna::guide::open::Shelf;
use abstractions::fauna::guide::{Guide, Seen};
use abstractions::life::Guild;
use abstractions::menus::Menu;

use super::*;

/// The pack opened on its key with the stones, the tissue and the tools in it, a second entry
/// selected with the arrows, then the workshop on Tab: shots `e70` to `e72`.
pub fn play_pack(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    const STAGE: &str = "pack";
    if frame == 1 {
        notes.play.start(frame);
        view(world, ViewMode::Third, ViewMode::First);
    }
    let pack_key = bound(world, |s| s.controls.keys.pack);
    let open = *world.resource::<State<Workshop>>().get() == Workshop::Open;
    let making = world.resource::<Bench>().tab == BenchTab::Make;
    match frame {
        10 | 11 => tap(world, pack_key, frame, 10),
        30 if making => key(world, KeyCode::Tab, true),
        31 => key(world, KeyCode::Tab, false),
        60 if !open => return gave_up(world, STAGE, "the pack key did not open the pack"),
        60 => {
            say(
                STAGE,
                &format!("e70 the pack after the hunt: {}", pack_words(world)),
            );
            shot(world, "e70-pack-loot");
        }
        61..80 => [61, 67, 73]
            .into_iter()
            .for_each(|at| tap(world, KeyCode::ArrowDown, frame, at)),
        100 => shot(world, "e71-pack-entry"),
        101 | 102 => tap(world, KeyCode::Tab, frame, 101),
        140 => {
            say(STAGE, &format!("e72 {}", offer_words(world)));
            shot(world, "e72-workshop-loot");
        }
        141 | 142 => tap(world, pack_key, frame, 141),
        165 => return Act::Next,
        _ => {}
    }
    Act::Wait
}

/// Stands the robot 12 m from the nearest drawn animal of a species the guide has not seen yet.
/// Returns false when every drawn species was seen.
fn meet(world: &mut World) -> bool {
    let globe = world.resource::<Terrain>().globe().clone();
    let Some(life) = globe.life() else {
        return false;
    };
    let me = robot(world);
    let drawn = animals(world);
    let seen = world.resource::<Seen>().clone();
    let unseen = drawn
        .iter()
        .filter(|a| !seen.has(life.species[usize::from(a.2)].id))
        .min_by(|a, b| a.1.distance(me).total_cmp(&b.1.distance(me)));
    let Some((_, at, species)) = unseen.copied() else {
        return false;
    };
    let up = at.normalize();
    let away = (me - at)
        .reject_from(up)
        .normalize_or(up.any_orthonormal_vector());
    let feet = clear(world, at, away, 12.0);
    stand(world, feet, at, pitch_to(feet, at));
    let name = &life.species[usize::from(species)].name;
    say(
        "guide",
        &format!("standing 12 m from a {name}, not seen yet"),
    );
    true
}

/// Returns the row of the guide of the first animal that was seen, and how many species were.
fn seen_animal(world: &World) -> (usize, String) {
    let shelf = world.resource::<Shelf>();
    let globe = world.resource::<Terrain>().globe();
    let animal = |index: u16| {
        let species = globe
            .life()
            .and_then(|life| life.species.get(usize::from(index)));
        species.is_some_and(|s| s.role.guild != Guild::Producer)
    };
    let entries = &shelf.book.entries;
    let row = entries.iter().position(|e| e.seen && animal(e.index));
    let animals = entries.iter().filter(|e| e.seen && animal(e.index)).count();
    (
        row.unwrap_or(0),
        format!("{}, {animals} of them animals", shelf.book.tally()),
    )
}

/// The field guide: the robot goes near each drawn species it has not seen, then opens the
/// guide on its key and moves down to the first animal it knows: shots `e75` and `e76`.
pub fn play_guide(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    const STAGE: &str = "guide";
    if frame == 1 {
        notes.play.start(frame);
        view(world, ViewMode::First, ViewMode::First);
    }
    let play = &mut notes.play;
    let held = play.held(frame);
    let guide_key = bound(world, |s| s.controls.keys.guide);
    if play.step == 0 {
        let due = held % 45 == 2;
        if held > 500 || (due && !meet(world)) {
            play.next(frame);
        }
        return Act::Wait;
    }
    let open = *world.resource::<State<Guide>>().get() == Guide::Open;
    match held {
        1 | 2 => press(world, guide_key, held),
        50 if !open => return gave_up(world, STAGE, "the guide key did not open the guide"),
        50 => {
            say(STAGE, &format!("e75 the guide: {}", seen_animal(world).1));
            shot(world, "e75-guide");
        }
        52..172 => {
            let below = world.resource::<Shelf>().window.cursor < seen_animal(world).0;
            match (held - 52) % 3 {
                0 if below => key(world, KeyCode::ArrowDown, true),
                1 => key(world, KeyCode::ArrowDown, false),
                _ => {}
            }
        }
        182 => {
            let shelf = world.resource::<Shelf>();
            let entry = shelf.book.entries.get(shelf.window.cursor);
            let name = entry.map_or("?".to_string(), |e| e.name.clone());
            say(STAGE, &format!("e76 the card of {name}"));
            shot(world, "e76-guide-animal");
        }
        183 | 184 => tap(world, guide_key, held, 183),
        200 => return Act::Next,
        _ => {}
    }
    Act::Wait
}

/// The pause menu over the game, as the last shot: the menu stays open.
pub fn play_pause(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        tidy(world);
    }
    let paused = *world.resource::<State<Menu>>().get() != Menu::None;
    match frame {
        10 if !paused => key(world, KeyCode::Escape, true),
        11 => key(world, KeyCode::Escape, false),
        _ => {}
    }
    match frame {
        50 => {
            say("pause", "e80 the pause menu");
            Act::Shoot("e80-pause".into())
        }
        _ => Act::Wait,
    }
}
