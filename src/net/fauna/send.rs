//! Outgoing creature pose broadcasting for creatures authored by this client.

use abstractions_net::host::DATAGRAM_BUDGET;
use abstractions_net::ids::BodyId;
use abstractions_net::wire::{
    FaunaPose, Frame as WireFrame, Head, Kind, Record, encode_fauna, max_fauna_poses, push_within,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{FAUNA_EVERY, FaunaNet, stance_to_u8};
use crate::fauna::{Beast, Dead, Lead};
use crate::net::link::Link;
use crate::net::state::Remotes;
use crate::player::Player;
use crate::space::{Anchor, Clock, Frame, Location};

/// World state read when sending authoritative creature batches.
#[derive(SystemParam)]
pub(super) struct SendQuery<'w, 's> {
    /// Active server connection.
    link: ResMut<'w, Link>,
    /// Current body underfoot.
    location: Option<Res<'w, Location>>,
    /// Render-to-planet anchor frame.
    anchor: Option<Res<'w, Anchor>>,
    /// Universe simulation clock.
    clock: Res<'w, Clock>,
    /// Real wall-clock time for rate limiting.
    time: Res<'w, Time<Real>>,
    /// Other players' robots and ships in the session.
    remotes: Res<'w, Remotes>,
    /// Local fauna sync state and reused buffers.
    net: ResMut<'w, FaunaNet>,
    /// Local player transform for distance-sorted batch ordering.
    player: Query<'w, 's, &'static Transform, With<Player>>,
    /// Drawn living and dead creatures on the planet.
    beasts: Query<'w, 's, (&'static Beast, Has<Dead>)>,
}

/// Returns true when at least one remote player is on or orbiting `body`.
fn peer_on_body(remotes: &Remotes, body: BodyId) -> bool {
    let same =
        |frame: WireFrame| frame == WireFrame::Surface(body) || frame == WireFrame::Around(body);
    remotes
        .robots
        .values()
        .map(|track| track.frame)
        .chain(remotes.ships.values().map(|track| track.frame))
        .any(same)
}

/// Rebuilds `owned` from currently drawn living non-carcass beasts not owned remotely, and fills `poses`.
fn gather_poses(net: &mut FaunaNet, beasts: &Query<(&Beast, Has<Dead>)>) {
    net.owned.clear();
    net.poses.clear();
    for (beast, dead) in beasts {
        if dead || matches!(beast.lead, Lead::Carcass { .. }) || beast.fade.leaving() {
            continue;
        }
        let (herd, slot) = beast.lead.key();
        if net.owners.contains_key(&(herd, slot)) {
            continue;
        }
        net.owned.insert((herd, slot));
        let pos = beast.spot.planet(beast.motion.root.place);
        let vel = (beast.spot.rot * beast.motion.root.vel.at.as_dvec3()).as_vec3();
        let fwd_local = (beast.motion.root.turn * Vec3::NEG_Z).normalize_or(Vec3::NEG_Z);
        let heading = (beast.spot.rot * fwd_local.as_dvec3())
            .as_vec3()
            .normalize_or(Vec3::NEG_Z);
        net.poses.push(FaunaPose {
            herd,
            slot,
            species: beast.index,
            pos,
            vel,
            heading,
            stance: stance_to_u8(beast.wits.stance),
            excite: beast.mood.alarm,
        });
    }
}

/// Sends up to date authoritative creature batches at 10 Hz while another player shares this planet.
pub(super) fn send(mut q: SendQuery) {
    let (Some(you), Some(loc), Some(anchor)) =
        (q.link.you, q.location.as_deref(), q.anchor.as_deref())
    else {
        return;
    };
    if anchor.frame != Frame::Surface(loc.body) {
        q.net.owned.clear();
        return;
    }
    if q.net.body != Some(loc.body) {
        q.net.owners.clear();
        q.net.owned.clear();
        q.net.next_send = 0.0;
        q.net.body = Some(loc.body);
    }
    if !peer_on_body(&q.remotes, loc.body) {
        q.net.owned.clear();
        return;
    }
    let max = q.link.max_datagram().min(DATAGRAM_BUDGET);
    let now = q.time.elapsed_secs_f64();
    if max == 0 || now < q.net.next_send {
        return;
    }
    q.net.next_send = now + FAUNA_EVERY;
    gather_poses(&mut q.net, &q.beasts);
    if q.net.poses.is_empty() {
        return;
    }
    let origin = q
        .player
        .single()
        .map_or_else(|_| q.net.poses[0].pos, |t| anchor.to_frame(t.translation));
    q.net.poses.sort_unstable_by(|a, b| {
        let da = (a.pos - origin).length_squared();
        let db = (b.pos - origin).length_squared();
        da.total_cmp(&db)
            .then((a.herd, a.slot).cmp(&(b.herd, b.slot)))
    });
    let per_chunk = max_fauna_poses(max).max(1);
    let head = Head {
        player: you,
        kind: Kind::Fauna,
        frame: WireFrame::Surface(loc.body),
        time: q.clock.seconds,
        flags: 0,
    };
    let net = &mut *q.net;
    for chunk in net.poses.chunks(per_chunk) {
        net.buf.clear();
        encode_fauna(origin, chunk, &mut net.buf);
        net.batch.clear();
        let record = Record {
            head,
            body: &net.buf,
        };
        if push_within(&mut net.batch, &record, max) {
            q.link.send_records(&net.batch);
        }
    }
}
