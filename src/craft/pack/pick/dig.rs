//! Terrain digging and underground mineral vein harvesting, boosted by Crush pickaxes.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::{Foraged, HarvestTarget, striker};
use crate::audio::{Body, Impact, Outer};
use crate::craft::build::{Intent, Place};
use crate::craft::matter::{Mineral, Stuff};
use crate::craft::pack::tune::{PIECE_SPEED, STONE_TONES};
use crate::craft::pack::{Notice, Pack};
use crate::craft::wield::Held;
use crate::effects::Fling;
use crate::space::{Anchor, Location, up};
use crate::universe::PlanetRecord;
use crate::world::Terrain;
use crate::world::digs::{DIG_DEPTH, DIG_RADIUS, DigStamp, Digs, TerrainDug};
use crate::world::rocks::mined_vein;

/// Returns `(dig_depth, ore_chunks)` from held tool intent: `Crush` pickaxes dig deeper and yield more.
pub fn dig_power(held: Option<Intent>) -> (f32, u16) {
    match held {
        Some(Intent::Crush) => (0.85, 2),
        Some(_) => (0.60, 1),
        None => (DIG_DEPTH, 1),
    }
}

/// Samples the underground mineral vein struck at `(pos_m, total_depth)`, falling back to surface rock.
pub fn dug_mineral(record: &PlanetRecord, seed: u64, pos_m: DVec3, total_depth: f32) -> Mineral {
    mined_vein(record, seed, pos_m, total_depth)
        .or_else(|| Mineral::SURFACE.into_iter().find(|m| m.lies_on(record)))
        .unwrap_or(Mineral::Basalt)
}

/// Returns dirt and mineral flecks flung out of a fresh dig hole.
fn dig_clods(mineral: Mineral, at: Vec3, up_dir: Vec3) -> [Fling; 4] {
    let (u, v) = up_dir.any_orthonormal_pair();
    let stone_tone = STONE_TONES[mineral as usize];
    std::array::from_fn(|k| {
        let a = k as f32 * 1.57 + 0.3;
        let dir = (up_dir * 0.95 + u * a.cos() + v * a.sin()).normalize_or(up_dir);
        Fling {
            at,
            velocity: dir * PIECE_SPEED * (1.0 + 0.2 * k as f32),
            radius: 0.04,
            colour: if k == 0 {
                stone_tone
            } else {
                [0.28, 0.22, 0.16]
            },
            wet: false,
            tag: None,
        }
    })
}

/// Stamps one crater, samples an underground ore vein, and deposits discrete chunks in `pack`.
pub(super) fn dig_ground(
    at: Vec3,
    (terrain, location, held, anchor): (&Terrain, &Location, &Held, &Anchor),
    (pack, digs): (&mut Pack, &mut Digs),
    (dug_tx, sounds, flings, notice): (
        &mut MessageWriter<TerrainDug>,
        &mut MessageWriter<Impact>,
        &mut MessageWriter<Fling>,
        &mut Notice,
    ),
) {
    let intent = held.0.as_ref().map(|h| h.intent);
    let (depth, want_chunks) = dig_power(intent);
    let pos_m = anchor.to_frame(at);
    let dir = pos_m.normalize_or(DVec3::Y);
    let stamp = DigStamp {
        planet: location.body,
        dir: dir.to_array(),
        radius: DIG_RADIUS,
        depth,
    };
    digs.push(stamp);
    dug_tx.write(TerrainDug(stamp));
    let globe = terrain.globe();
    let total_depth = digs.depth_at(location.body, dir, globe.radius());
    let mineral = dug_mineral(
        globe.record(),
        globe.seed(),
        dir * globe.radius(),
        total_depth,
    );
    let stuff = Stuff::Mineral(mineral);
    let room = pack.room_for(&stuff);
    let grant = want_chunks.min(room.min(u32::from(u16::MAX)) as u16);
    let added = if grant > 0 {
        pack.add_stack(stuff, mineral.material(), grant)
    } else {
        0
    };
    if added > 0 {
        *notice = Notice::of(&stuff, f32::from(added) * stuff.unit_mass());
    } else {
        notice.line = "Pack full!".to_string();
        notice.left = crate::craft::pack::tune::NOTICE;
    }
    let place = Place::on(globe.gravity(), globe.record());
    let (tool, joules) = striker(held, &place);
    let mat = mineral.material();
    sounds.write(Impact {
        at,
        joules,
        cut: false,
        outer: Outer::Ground,
        struck: Body {
            stiffness: mat.stiffness,
            mass: 40.0,
        },
        tool,
        deflected: false,
    });
    let up_dir = up(anchor, at);
    for clod in dig_clods(mineral, at, up_dir) {
        flings.write(clod);
    }
}

/// ECS system handling `Foraged` ground dig strikes.
pub(crate) fn handle_dig(
    mut foraged: MessageReader<Foraged>,
    (terrain, location, held, anchor): (Res<Terrain>, Res<Location>, Res<Held>, Res<Anchor>),
    (mut pack, mut digs): (ResMut<Pack>, ResMut<Digs>),
    (mut dug_tx, mut sounds, mut flings, mut notice): (
        MessageWriter<TerrainDug>,
        MessageWriter<Impact>,
        MessageWriter<Fling>,
        ResMut<Notice>,
    ),
) {
    for ev in foraged.read() {
        if ev.target == HarvestTarget::Ground {
            dig_ground(
                ev.at,
                (&terrain, &location, &held, &anchor),
                (&mut pack, &mut digs),
                (&mut dug_tx, &mut sounds, &mut flings, &mut notice),
            );
        }
    }
}
