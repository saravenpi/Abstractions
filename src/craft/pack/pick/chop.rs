//! Tree chopping: barehanded or boosted by a Cut tool (hatchet), yielding wood logs.

use bevy::prelude::*;

use super::{Foraged, HarvestTarget, striker};
use crate::audio::{Body, Impact, Outer};
use crate::craft::build::{Intent, Place};
use crate::craft::matter::{Stuff, WoodKind};
use crate::craft::pack::tune::PIECE_SPEED;
use crate::craft::pack::{Notice, Pack};
use crate::craft::wield::Held;
use crate::effects::Fling;
use crate::space::{Anchor, Location, up};
use crate::world::Terrain;
use crate::world::trees::{Felled, Tree, TreeChopped};

/// RGB tone of fresh wood chips flying off a chopped trunk.
const CHIP_TONES: [[f32; 3]; 3] = [[0.54, 0.36, 0.20], [0.64, 0.46, 0.26], [0.42, 0.27, 0.15]];

/// Returns `(damage, bonus_logs)` for chopping with `held`: `Cut` tools cut twice as hard.
pub fn chop_power(held: Option<Intent>) -> (u8, u16) {
    match held {
        Some(Intent::Cut) => (2, 2),
        Some(_) => (1, 1),
        None => (1, 1),
    }
}

/// Pure chop step for tests and gameplay: damages `tree_id`, emits logs into `pack`, and returns
/// `(felled_now, added_units)`.
pub fn apply_chop(
    tree_id: crate::world::trees::TreeId,
    power: (u8, u16),
    felled: &mut Felled,
    pack: &mut Pack,
) -> (bool, u16) {
    let fell_now = felled.chop(tree_id, power.0);
    let want = power.1 + u16::from(fell_now) * 2;
    let room = pack.room_for(&Stuff::Wood(WoodKind::Log));
    let grant = want.min(room.min(u32::from(u16::MAX)) as u16);
    let added = if grant > 0 {
        pack.add_stack(Stuff::Wood(WoodKind::Log), WoodKind::Log.material(), grant)
    } else {
        0
    };
    (fell_now, added)
}

/// Returns three flying wood splinter flecks scattered away from `at`.
fn wood_chips(at: Vec3, up_dir: Vec3) -> [Fling; 3] {
    let (u, v) = up_dir.any_orthonormal_pair();
    std::array::from_fn(|k| {
        let a = k as f32 * 2.1 + 0.4;
        let dir = (up_dir * 0.9 + u * a.cos() + v * a.sin()).normalize_or(up_dir);
        Fling {
            at,
            velocity: dir * PIECE_SPEED * (0.9 + 0.2 * k as f32),
            radius: 0.045,
            colour: CHIP_TONES[k % CHIP_TONES.len()],
            wet: false,
            tag: None,
        }
    })
}

/// Applies tree chops when [`Foraged`] targets a tree.
pub(super) fn chop_tree(
    tree: Tree,
    at: Vec3,
    (terrain, location, held, anchor): (&Terrain, &Location, &Held, &Anchor),
    (pack, felled): (&mut Pack, &mut Felled),
    (told, sounds, flings, notice): (
        &mut MessageWriter<TreeChopped>,
        &mut MessageWriter<Impact>,
        &mut MessageWriter<Fling>,
        &mut Notice,
    ),
) {
    let intent = held.0.as_ref().map(|h| h.intent);
    let (power, logs) = chop_power(intent);
    let (_, added) = apply_chop(tree.id, (power, logs), felled, pack);
    told.write(TreeChopped {
        planet: location.body,
        id: tree.id,
        power,
    });
    if added > 0 {
        *notice = Notice::of(
            &Stuff::Wood(WoodKind::Log),
            f32::from(added) * WoodKind::Log.unit_mass(),
        );
    } else {
        notice.line = "Pack full!".to_string();
        notice.left = crate::craft::pack::tune::NOTICE;
    }
    let place = Place::on(terrain.globe().gravity(), terrain.globe().record());
    let (tool, joules) = striker(held, &place);
    let mat = WoodKind::Log.material();
    sounds.write(Impact {
        at,
        joules,
        cut: matches!(intent, Some(Intent::Cut)),
        outer: Outer::Ground,
        struck: Body {
            stiffness: mat.stiffness,
            mass: 80.0,
        },
        tool,
        deflected: false,
    });
    let up_dir = up(anchor, at);
    for chip in wood_chips(at, up_dir) {
        flings.write(chip);
    }
}

/// ECS system handling `Foraged` tree chops.
pub(crate) fn handle_chop(
    mut foraged: MessageReader<Foraged>,
    (terrain, location, held, anchor): (Res<Terrain>, Res<Location>, Res<Held>, Res<Anchor>),
    (mut pack, mut felled): (ResMut<Pack>, ResMut<Felled>),
    (mut told, mut sounds, mut flings, mut notice): (
        MessageWriter<TreeChopped>,
        MessageWriter<Impact>,
        MessageWriter<Fling>,
        ResMut<Notice>,
    ),
) {
    for ev in foraged.read() {
        if let HarvestTarget::Tree(tree) = ev.target {
            chop_tree(
                tree,
                ev.at,
                (&terrain, &location, &held, &anchor),
                (&mut pack, &mut felled),
                (&mut told, &mut sounds, &mut flings, &mut notice),
            );
        }
    }
}
