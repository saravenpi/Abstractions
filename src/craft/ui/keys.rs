//! The keys of the open screen: the search, moving through the list and the proposals, and the
//! actions of each tab.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use super::Workshop;
use super::bench::{Bench, Tab, step_intent};
use super::here::Here;
use super::lines::What;
use super::offer::{offer, title};
use super::paint::arranged;
use crate::computer::sift::type_into;
use crate::craft::matter::Treatment;
use crate::craft::pack::Pack;
use crate::craft::wield::pressed;

/// Returns the sum of the moves of the keys pressed this frame among `moves`.
fn moved(keys: &ButtonInput<KeyCode>, moves: &[(KeyCode, isize)]) -> isize {
    moves
        .iter()
        .filter(|(k, _)| keys.just_pressed(*k))
        .map(|(_, by)| by)
        .sum()
}

/// Starts a search on the slash key, edits it with the keys pressed this frame, ends it on Escape
/// or Enter, and changes the order on F2. Keys pressed before the screen opened are dropped.
pub(super) fn type_filter(
    mut typed: MessageReader<KeyboardInput>,
    (keys, opened): (Res<ButtonInput<KeyCode>>, Res<State<Workshop>>),
    mut bench: ResMut<Bench>,
) {
    if opened.is_changed() || bench.tab != Tab::Pack {
        typed.clear();
        return;
    }
    if !bench.typing && keys.just_pressed(KeyCode::Slash) {
        bench.typing = true;
        typed.clear();
        return;
    }
    if keys.just_pressed(KeyCode::F2) {
        bench.sort = bench.sort.next();
    }
    if !bench.typing {
        typed.clear();
        return;
    }
    let mut filter = bench.filter.clone();
    for key in typed.read().filter(|key| key.state == ButtonState::Pressed) {
        match &key.logical_key {
            Key::Escape | Key::Enter => bench.typing = false,
            Key::Backspace => _ = filter.pop(),
            Key::Character(text) => _ = type_into(&mut filter, text),
            _ => {}
        }
    }
    if filter != bench.filter {
        (bench.filter, bench.window) = (filter, Default::default());
    }
}

/// Switches tabs on Tab, moves through the list on the pack tab, and through the proposals and
/// the intents on the workshop tab.
pub(super) fn step(
    keys: Res<ButtonInput<KeyCode>>,
    (pack, here): (Res<Pack>, Here),
    mut bench: ResMut<Bench>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        bench.tab = bench.tab.next();
        bench.typing = false;
        bench.note.clear();
        return;
    }
    let ups = [(KeyCode::ArrowDown, 1), (KeyCode::ArrowUp, -1)];
    match bench.tab {
        Tab::Pack => {
            let page = bench.rows.max(1) as isize;
            let pages = [(KeyCode::PageDown, page), (KeyCode::PageUp, -page)];
            let by = moved(&keys, &ups) + moved(&keys, &pages);
            if by != 0 && !bench.typing {
                let (len, rows) = (arranged(&bench, &pack, &here).len(), bench.rows);
                bench.window.step(len, rows, by);
            }
        }
        Tab::Make => {
            let pick = moved(&keys, &ups);
            let side = moved(&keys, &[(KeyCode::ArrowRight, 1), (KeyCode::ArrowLeft, -1)]);
            if pick != 0 || side != 0 {
                bench.pick = (bench.pick as isize + pick).rem_euclid(3) as usize;
                bench.intent = step_intent(bench.intent, side);
                bench.swap = [0, 0];
            }
        }
    }
}

/// Holds, slots or drops the selected entry of the pack.
fn pack_keys(keys: &ButtonInput<KeyCode>, pack: &mut Pack, here: &Here, bench: &mut Bench) {
    let all = arranged(bench, pack, here);
    let Some(line) = all.get(bench.window.cursor) else {
        return;
    };
    match line.what {
        What::Item(id) if keys.just_pressed(KeyCode::Enter) => {
            let slot = pack.bar.held;
            pack.bar.put(slot, id);
        }
        What::Item(id) if keys.just_pressed(KeyCode::KeyX) => {
            pack.bar.forget(id);
            pack.items.retain(|item| item.id != id);
        }
        What::Item(id) => {
            if let Some(slot) = pressed(keys) {
                pack.bar.put(slot, id);
            }
        }
        What::Stack(at) if keys.just_pressed(KeyCode::KeyX) => _ = pack.stacks.remove(at),
        What::Stack(_) => {}
    }
}

/// Swaps a piece, treats the head material or makes the chosen proposal.
fn make_keys(keys: &ButtonInput<KeyCode>, pack: &mut Pack, here: &Here, bench: &mut Bench) {
    bench.swap[0] += usize::from(keys.just_pressed(KeyCode::KeyJ));
    bench.swap[1] += usize::from(keys.just_pressed(KeyCode::KeyH));
    let (treat, make) = (
        keys.just_pressed(KeyCode::KeyT),
        keys.just_pressed(KeyCode::Enter),
    );
    if !treat && !make {
        return;
    }
    let stock = pack.stock();
    let chosen = offer(bench, &stock, &here.place())
        .ok()
        .and_then(|o| o.chosen);
    let Some(chosen) = chosen else {
        bench.note = "nothing to make from what is carried".to_string();
        return;
    };
    bench.note = match make {
        true => match pack.make(&chosen, &stock) {
            Some(_) => format!("made a {}", title(&chosen, &stock)),
            None => "not enough material".to_string(),
        },
        false => treat_head(pack, here, &stock[usize::from(chosen.design.of[1])].stuff),
    };
    bench.swap = [0, 0];
}

/// Treats the head material `stuff` with the first treatment the planet allows that it has not
/// had, and returns what happened in words.
fn treat_head(pack: &mut Pack, here: &Here, stuff: &crate::craft::matter::Stuff) -> String {
    let Some(planet) = here.planet() else {
        return "no planet underfoot".to_string();
    };
    let can = Treatment::ALL
        .into_iter()
        .filter(|t| t.possible(&planet.record));
    match can.into_iter().find(|t| pack.treat(stuff, *t)) {
        Some(t) => format!("head material {}", super::lines::treated(t)),
        None => "no treatment left for this material here".to_string(),
    }
}

/// Runs the actions of the open tab, unless a search is being typed.
pub(super) fn act(
    keys: Res<ButtonInput<KeyCode>>,
    (mut pack, here): (ResMut<Pack>, Here),
    mut bench: ResMut<Bench>,
) {
    if bench.typing || keys.get_just_pressed().next().is_none() {
        return;
    }
    match bench.tab {
        Tab::Pack => pack_keys(&keys, &mut pack, &here, &mut bench),
        Tab::Make => make_keys(&keys, &mut pack, &here, &mut bench),
    }
}
