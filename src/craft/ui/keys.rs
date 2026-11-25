//! The keys of the open screen: Tab between the panes, the arrows through the pack, the intents
//! and the cards, the number keys for the hotbar, and the actions of each pane. Each action is a
//! [`Bind`]: the button that shows it on screen carries the same label and key.

use bevy::prelude::*;

use super::bench::{Pane, step_intent};
use super::catalog_ui::filtered_recipes;
use super::click::Press;
use super::deeds::Deed;
use crate::craft::build::catalog::Category;
use crate::craft::pack::BAR_SLOTS;
use crate::craft::wield::pressed;
use crate::ui::Icon;

/// One action of a pane: what it does, how its button shows it, and its key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bind {
    /// What it does.
    pub press: Press,
    /// The icon of its button.
    pub icon: Option<Icon>,
    /// The word of its button.
    pub label: &'static str,
    /// The name of its key, as the button shows it.
    pub key: &'static str,
    /// Its key.
    pub code: KeyCode,
}

/// The actions of the pack pane: Hold, Sort, Drop.
pub const PACK_BINDS: [Bind; 3] = [
    Bind {
        press: Press::Hold,
        icon: None,
        label: "Hold",
        key: "Enter",
        code: KeyCode::Enter,
    },
    Bind {
        press: Press::SortPack,
        icon: Some(Icon::Shuffle),
        label: "Sort",
        key: "S",
        code: KeyCode::KeyS,
    },
    Bind {
        press: Press::Drop,
        icon: Some(Icon::Trash),
        label: "Drop",
        key: "X",
        code: KeyCode::KeyX,
    },
];

/// The actions of the workshop pane, on the focused card. Make comes first: it is the primary one.
pub const MAKE_BINDS: [Bind; 4] = [
    Bind {
        press: Press::Make,
        icon: Some(Icon::Plus),
        label: "Make",
        key: "Enter",
        code: KeyCode::Enter,
    },
    Bind {
        press: Press::Head,
        icon: Some(Icon::Shuffle),
        label: "Other head",
        key: "H",
        code: KeyCode::KeyH,
    },
    Bind {
        press: Press::Shaft,
        icon: Some(Icon::Shuffle),
        label: "Other shaft",
        key: "J",
        code: KeyCode::KeyJ,
    },
    Bind {
        press: Press::Treat,
        icon: None,
        label: "Treat",
        key: "T",
        code: KeyCode::KeyT,
    },
];

/// The moves of the pack pane, as the legend shows them: what each key does, then the key.
pub const PACK_LEGEND: [(&str, &str); 4] = [
    ("Workshop", "Tab"),
    ("Choose", "Up Down"),
    ("Slot", "1-6"),
    ("Half", "Shift X"),
];

/// The moves of the workshop pane, as the legend shows them.
pub const MAKE_LEGEND: [(&str, &str); 4] = [
    ("Pack", "Tab"),
    ("Intent", "Left Right"),
    ("Tool", "Up Down"),
    ("Slot", "1-6"),
];

/// Returns the actions of `pane`.
pub fn binds(pane: Pane) -> &'static [Bind] {
    match pane {
        Pane::Pack => &PACK_BINDS,
        Pane::Workshop => &MAKE_BINDS,
    }
}

/// Returns the moves of `pane`, as its legend shows them.
pub fn moves(pane: Pane) -> &'static [(&'static str, &'static str)] {
    match pane {
        Pane::Pack => &PACK_LEGEND,
        Pane::Workshop => &MAKE_LEGEND,
    }
}

/// Returns the sum of the moves of the keys pressed this frame among `moves`.
fn moved(keys: &ButtonInput<KeyCode>, moves: &[(KeyCode, isize)]) -> isize {
    moves
        .iter()
        .filter(|(k, _)| keys.just_pressed(*k))
        .map(|(_, by)| by)
        .sum()
}

/// Switches panes on Tab, moves through slots, categories, recipes, or intents with arrow keys.
pub(super) fn step(keys: Res<ButtonInput<KeyCode>>, mut deed: Deed) {
    if keys.just_pressed(KeyCode::Tab) {
        deed.bench.pane = deed.bench.pane.next();
        return;
    }
    let down = moved(&keys, &[(KeyCode::ArrowDown, 1), (KeyCode::ArrowUp, -1)]);
    let side = moved(&keys, &[(KeyCode::ArrowRight, 1), (KeyCode::ArrowLeft, -1)]);
    let pane = deed.bench.pane;
    match pane {
        Pane::Pack if down != 0 || side != 0 => {
            if let Some(cur) = deed.bench.selected_slot.filter(|s| *s >= BAR_SLOTS) {
                let off = cur - BAR_SLOTS;
                let row = (off / 6) as isize + down;
                let col = (off % 6) as isize + side;
                let next = (row.rem_euclid(4) * 6 + col.rem_euclid(6)) as usize;
                deed.bench.selected_slot = Some(BAR_SLOTS + next);
            } else if down != 0 {
                let last = deed.lines().len().saturating_sub(1);
                deed.bench.cursor = deed.bench.cursor.saturating_add_signed(down).min(last);
            }
        }
        Pane::Workshop if deed.bench.category == Category::Tools && (down != 0 || side != 0) => {
            let pick = (deed.bench.pick as isize + down).rem_euclid(3) as usize;
            let intent = step_intent(deed.bench.intent, side);
            deed.ask(intent, pick);
        }
        Pane::Workshop if down != 0 || side != 0 => {
            if side != 0 {
                let at = Category::ALL
                    .iter()
                    .position(|c| *c == deed.bench.category)
                    .unwrap_or(0) as isize;
                let cat =
                    Category::ALL[(at + side).rem_euclid(Category::ALL.len() as isize) as usize];
                deed.pick_category(cat);
            }
            if down != 0 {
                let len =
                    filtered_recipes(deed.bench.category, &deed.pack, deed.bench.bullet_mineral)
                        .len();
                if len > 0 {
                    deed.bench.recipe_idx =
                        (deed.bench.recipe_idx as isize + down).rem_euclid(len as isize) as usize;
                }
            }
        }
        _ => {}
    }
}

/// Runs the action whose key was pressed this frame in the pane the keys drive, and the number
/// keys of the hotbar.
pub(super) fn act(keys: Res<ButtonInput<KeyCode>>, mut deed: Deed) {
    if keys.get_just_pressed().next().is_none() {
        return;
    }
    if let Some(slot) = pressed(&keys) {
        deed.press(Press::Slot(slot));
    }
    let pane = deed.bench.pane;
    if pane == Pane::Workshop && deed.bench.category != Category::Tools {
        if keys.just_pressed(KeyCode::Enter) {
            deed.press(Press::CraftBatch(1));
        } else if keys.just_pressed(KeyCode::KeyC) {
            deed.press(Press::CraftBatch(5));
        } else if keys.just_pressed(KeyCode::KeyM) {
            deed.press(Press::CraftBatch(0));
        } else if keys.just_pressed(KeyCode::KeyO) {
            deed.press(Press::CycleMineral);
        }
        return;
    }
    for bind in binds(pane).iter().filter(|b| keys.just_pressed(b.code)) {
        deed.press(bind.press);
    }
}
