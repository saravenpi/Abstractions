//! Mending in the field: a plate is patched with something hard, a joint is bound with sinew or
//! resin. The robot spends what it carries, and no patch lifts a part past [`PATCH_CAP`].

use bevy::prelude::*;

use super::tune::{BIND_MASS, HARD, PATCH, PATCH_CAP, PLATE_MASS};
use crate::condition::{Condition, Part};
use crate::craft::matter::{Class, Stuff};
use crate::craft::pack::{Pack, Stack};
use crate::settings::{Persistent, Settings};

/// Returns true when `stack` binds a joint: sinew or resin.
fn binds(stack: &Stack) -> bool {
    matches!(
        stack.stuff,
        Stuff::Tissue {
            class: Class::Hide | Class::Resin,
            ..
        }
    )
}

/// Returns the heap of `pack` that mends a part, and the mass it takes: the strongest binding for a
/// joint, the softest hard material for a plate, so the best stays for tools.
pub fn supply(pack: &Pack, joint: bool) -> Option<(Stuff, f32)> {
    let (mass, mut heaps): (f32, Vec<&Stack>) = match joint {
        true => (BIND_MASS, pack.stacks.iter().filter(|s| binds(s)).collect()),
        false => (
            PLATE_MASS,
            pack.stacks
                .iter()
                .filter(|s| s.material.hardness >= HARD)
                .collect(),
        ),
    };
    heaps.retain(|heap| heap.mass >= mass);
    let key = |heap: &&Stack| match joint {
        true => -heap.material.strength,
        false => heap.material.hardness,
    };
    heaps
        .into_iter()
        .min_by(|a, b| key(a).total_cmp(&key(b)))
        .map(|heap| (heap.stuff, mass))
}

/// Mends the worst part the pack can supply and returns it: a patch costs [`PLATE_MASS`] or
/// [`BIND_MASS`] of material and gives back [`PATCH`] of health up to [`PATCH_CAP`].
pub fn patch(condition: &mut Condition, pack: &mut Pack) -> Option<Part> {
    for part in condition.damaged() {
        if condition.of(part) >= PATCH_CAP {
            continue;
        }
        let Some((stuff, mass)) = supply(pack, part.joint()) else {
            continue;
        };
        if pack.take(&stuff, mass) {
            condition.mend(part, PATCH, PATCH_CAP);
            return Some(part);
        }
    }
    None
}

/// Patches the worst part of the robot when the mend key is pressed.
pub fn press(
    (keys, settings): (Res<ButtonInput<KeyCode>>, Res<Persistent<Settings>>),
    (mut condition, mut pack): (ResMut<Condition>, ResMut<Pack>),
) {
    if keys.just_pressed(settings.controls.keys.mend) && !condition.intact() {
        patch(&mut condition, &mut pack);
    }
}
