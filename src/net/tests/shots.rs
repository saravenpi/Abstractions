//! Tests of firearm shot relay frame conversion and held-gun byte tag codec.

use abstractions_net::io::ClientEvent;
use abstractions_net::log::PlayerId;
use abstractions_net::wire::{ClientMsg, ServerMsg, Shot};
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::kept::{kept, watch};
use super::rig::{push, run, sent};
use super::sync::joined;
use crate::audio::Gunshot;
use crate::avatar::strike::RemoteGun;
use crate::craft::gun::{Gun, GunKind, ShotFired};
use crate::craft::matter::Mineral;
use crate::net::hunt::item::{GUN_TAG, pack_gun, unpack_gun};
use crate::space::{Anchor, Frame};
use crate::universe::BodyId;

/// Returns a surface anchor with a non-trivial origin and rotation for frame-conversion tests.
fn rotated_anchor(body: BodyId) -> Anchor {
    Anchor {
        frame: Frame::Surface(body),
        origin: DVec3::new(10.0, 20.0, 30.0),
        rotation: DQuat::from_rotation_y(std::f64::consts::FRAC_PI_2),
        flat: false,
    }
}

/// Extracts the first [`Shot`] sent over `script`.
fn sent_shot(script: &crate::net::link::Script) -> Shot {
    sent(script)
        .into_iter()
        .find_map(|m| match m {
            ClientMsg::Shot(s) => Some(s),
            _ => None,
        })
        .expect("shot sent")
}

/// Returns a sample local rifle shot for relay tests.
fn sample_shot() -> ShotFired {
    ShotFired {
        origin: Vec3::new(1.0, 2.0, 3.0),
        dir: Vec3::NEG_Z,
        kind: GunKind::MineralRifle,
        barrel: Mineral::Cobaltite,
        ammo: Mineral::Galena,
        local: true,
    }
}

/// Local firearm shots convert to planet frame, self echoes are skipped, and remote shots replay.
#[test]
fn shot_relay_transforms_frames_and_skips_self() {
    let (mut app, script) = joined("shot-relay");
    app.insert_resource(rotated_anchor(BodyId::default()));
    watch::<ShotFired>(&mut app);
    watch::<Gunshot>(&mut app);
    let shot = sample_shot();
    app.world_mut().write_message(shot);
    run(&mut app, 2);
    let wire = sent_shot(&script);
    for by in [PlayerId(3), PlayerId(2)] {
        push(
            &script,
            ClientEvent::Message(ServerMsg::Shot { by, shot: wire }),
        );
    }
    run(&mut app, 2);
    let replayed: Vec<_> = kept::<ShotFired>(&app)
        .into_iter()
        .filter(|s| !s.local)
        .collect();
    assert_eq!(replayed.len(), 1);
    assert_eq!(
        (replayed[0].kind, replayed[0].barrel, replayed[0].ammo),
        (shot.kind, shot.barrel, shot.ammo)
    );
    assert!(replayed[0].origin.distance(shot.origin) < 1e-3);
    assert!(replayed[0].dir.distance(shot.dir) < 1e-3);
    assert_eq!(kept::<Gunshot>(&app).len(), 1);
    assert!(kept::<Gunshot>(&app)[0].rifle);
}

/// Held firearms encode as `[0xFF, kind, barrel]` and reject malformed tags.
#[test]
fn held_gun_tag_codec_round_trips() {
    let pistol = Gun::new(1, GunKind::FlintlockPistol, Mineral::Basalt);
    let rifle = Gun::new(2, GunKind::MineralRifle, Mineral::Lonsdaleite);
    let p_bytes = pack_gun(&pistol);
    let r_bytes = pack_gun(&rifle);
    assert_eq!(p_bytes, vec![GUN_TAG, 0, 0]);
    assert_eq!(r_bytes, vec![GUN_TAG, 1, (Mineral::ALL.len() - 1) as u8]);
    assert_eq!(
        unpack_gun(&p_bytes),
        Some(RemoteGun {
            kind: GunKind::FlintlockPistol,
            barrel: Mineral::Basalt
        })
    );
    assert_eq!(
        unpack_gun(&r_bytes),
        Some(RemoteGun {
            kind: GunKind::MineralRifle,
            barrel: Mineral::Lonsdaleite
        })
    );
    assert_eq!(unpack_gun(&[0x00, 0, 0]), None);
    assert_eq!(unpack_gun(&[GUN_TAG, 0]), None);
    assert_eq!(unpack_gun(&[GUN_TAG, 2, 0]), None);
    assert_eq!(unpack_gun(&[GUN_TAG, 0, 99]), None);
}
