//! Firearm shot relay in a session: local shots convert to the planet frame and go to the server,
//! and remote shots on the same body replay tracer visuals and gunshot audio.

use abstractions_net::wire::{ClientMsg, ServerMsg, Shot};
use bevy::math::DVec3;
use bevy::prelude::*;

use crate::audio::Gunshot;
use crate::craft::gun::{GunKind, ShotFired};
use crate::craft::matter::Mineral;
use crate::net::link::{FromServer, Link};
use crate::space::{Anchor, Frame};

/// Encodes a firearm chassis kind into its wire tag (`0` pistol, `1` rifle).
pub fn gun_kind_code(kind: GunKind) -> u8 {
    match kind {
        GunKind::FlintlockPistol => 0,
        GunKind::MineralRifle => 1,
    }
}

/// Decodes a wire firearm kind tag, falling back to pistol for unknown values.
pub fn gun_kind_of(code: u8) -> GunKind {
    match code {
        1 => GunKind::MineralRifle,
        _ => GunKind::FlintlockPistol,
    }
}

/// Encodes a mineral as its stable index in [`Mineral::ALL`].
pub fn mineral_code(mineral: Mineral) -> u8 {
    Mineral::ALL.iter().position(|m| *m == mineral).unwrap_or(0) as u8
}

/// Decodes a mineral index in [`Mineral::ALL`], falling back to basalt for out-of-range tags.
pub fn mineral_of(code: u8) -> Mineral {
    Mineral::ALL
        .get(usize::from(code))
        .copied()
        .unwrap_or(Mineral::Basalt)
}

/// Converts a render-space [`ShotFired`] into a planet-frame wire [`Shot`], or `None` off-planet.
pub fn to_wire(shot: &ShotFired, anchor: &Anchor) -> Option<Shot> {
    let Frame::Surface(body) = anchor.frame else {
        return None;
    };
    let origin = anchor.to_frame(shot.origin).as_vec3().to_array();
    let dir = (anchor.rotation.inverse() * shot.dir.as_dvec3())
        .normalize_or(DVec3::NEG_Z)
        .as_vec3()
        .to_array();
    Some(Shot {
        body,
        origin,
        dir,
        kind: gun_kind_code(shot.kind),
        barrel: mineral_code(shot.barrel),
        ammo: mineral_code(shot.ammo),
    })
}

/// Converts a planet-frame wire [`Shot`] into a remote render-space [`ShotFired`] when on the same
/// surface frame.
pub fn from_wire(shot: &Shot, anchor: &Anchor) -> Option<ShotFired> {
    let Frame::Surface(body) = anchor.frame else {
        return None;
    };
    if shot.body != body {
        return None;
    }
    let origin = anchor.to_render(Vec3::from_array(shot.origin).as_dvec3());
    let dir = (anchor.rotation * Vec3::from_array(shot.dir).as_dvec3())
        .normalize_or(DVec3::NEG_Z)
        .as_vec3();
    Some(ShotFired {
        origin,
        dir,
        kind: gun_kind_of(shot.kind),
        barrel: mineral_of(shot.barrel),
        ammo: mineral_of(shot.ammo),
        local: false,
    })
}

/// Forwards local firearm shots to the server in the planet's rotating frame.
pub fn send(mut link: ResMut<Link>, anchor: Res<Anchor>, mut fired: MessageReader<ShotFired>) {
    for ev in fired.read() {
        if ev.local
            && let Some(wire) = to_wire(ev, &anchor)
        {
            link.send(ClientMsg::Shot(wire));
        }
    }
}

/// Replays remote firearm shots on the current body as tracers and spatial gunshot audio.
pub fn receive(
    mut from: MessageReader<FromServer>,
    (link, anchor): (Res<Link>, Res<Anchor>),
    mut fired: MessageWriter<ShotFired>,
    mut sounds: MessageWriter<Gunshot>,
) {
    for FromServer(msg) in from.read() {
        let ServerMsg::Shot { by, shot } = msg else {
            continue;
        };
        if Some(*by) == link.you {
            continue;
        }
        if let Some(ev) = from_wire(shot, &anchor) {
            sounds.write(Gunshot {
                at: ev.origin,
                rifle: ev.kind == GunKind::MineralRifle,
            });
            fired.write(ev);
        }
    }
}
