//! Tests of the room's batched creature state relay: same-star forwarding, multi-chunk frames,
//! cross-star filtering and invalid record rejection.

use super::kit::{batch, heads, hello, join, room, star};
use crate::host::{Host, Outbox};
use crate::log::PlayerId;
use crate::wire::{FAUNA_BODY_MAX, Frame, Head, Kind, Record, push};

/// Builds a batch holding one [`Kind::Fauna`] record in `frame`.
fn fauna_batch(player: u32, frame: Frame, time: f64, size: usize) -> Vec<u8> {
    let head = Head {
        player: PlayerId(player),
        kind: Kind::Fauna,
        frame,
        time,
        flags: 0,
    };
    let body = vec![9; size];
    let mut bytes = Vec::new();
    push(&mut bytes, &Record { head, body: &body });
    bytes
}

/// Same-star players receive all chunks of a multi-chunk fauna frame without self-echo, while
/// players around another star receive none.
#[test]
fn fauna_relays_to_same_star_including_multichunk_and_skips_other_stars() {
    let mut room = room(8, None);
    let mut out = Outbox::default();
    let ada = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    let bob = join(&mut room, 2, hello("Bob", 2), 100.0, &mut out);
    let cy = join(&mut room, 3, hello("Cy", 3), 100.0, &mut out);

    room.datagram(
        bob,
        &batch(2, Kind::Robot, star(0), 1.0, 40),
        100.0,
        &mut out,
    );
    room.datagram(
        cy,
        &batch(3, Kind::Robot, star(5), 1.0, 40),
        100.0,
        &mut out,
    );
    out.clear();

    let surface = Frame::Surface(crate::ids::BodyId {
        star: star(0),
        index: 1,
        moon: 0,
    });
    let mut multi = fauna_batch(1, surface, 2.0, 600);
    multi.extend(fauna_batch(1, surface, 2.0, 500));
    room.datagram(ada, &multi, 100.1, &mut out);
    room.tick(100.1, &mut out);

    let bob_fauna: Vec<_> = heads(&out, bob)
        .into_iter()
        .filter(|h| h.kind == Kind::Fauna)
        .collect();
    assert_eq!(bob_fauna.len(), 2);
    assert!(
        bob_fauna
            .iter()
            .all(|h| h.player == PlayerId(1) && h.time == 2.0)
    );
    assert!(heads(&out, ada).iter().all(|h| h.kind != Kind::Fauna));
    assert!(heads(&out, cy).iter().all(|h| h.kind != Kind::Fauna));
}

/// Spoofed player ids, non-surface frames and bodies over [`FAUNA_BODY_MAX`] are dropped, and
/// leaving clears stored fauna snapshots for future players.
#[test]
fn fauna_drops_invalid_records_and_clears_on_leave() {
    let mut room = room(4, None);
    let mut out = Outbox::default();
    let ada = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    let bob = join(&mut room, 2, hello("Bob", 2), 100.0, &mut out);
    room.datagram(
        bob,
        &batch(2, Kind::Robot, star(0), 1.0, 40),
        100.0,
        &mut out,
    );
    out.clear();

    let body_id = crate::ids::BodyId {
        star: star(0),
        index: 1,
        moon: 0,
    };
    room.datagram(
        ada,
        &fauna_batch(2, Frame::Surface(body_id), 2.0, 100),
        100.1,
        &mut out,
    );
    room.datagram(
        ada,
        &fauna_batch(1, Frame::Around(body_id), 2.0, 100),
        100.1,
        &mut out,
    );
    room.datagram(
        ada,
        &fauna_batch(1, Frame::Surface(body_id), 2.0, FAUNA_BODY_MAX + 1),
        100.1,
        &mut out,
    );
    room.tick(100.1, &mut out);
    assert!(heads(&out, bob).iter().all(|h| h.kind != Kind::Fauna));

    room.datagram(
        ada,
        &fauna_batch(1, Frame::Surface(body_id), 3.0, 200),
        100.2,
        &mut out,
    );
    room.message(ada, crate::wire::ClientMsg::Bye, 100.2, &mut out);
    out.clear();
    room.tick(100.2, &mut out);
    assert!(heads(&out, bob).iter().all(|h| h.kind != Kind::Fauna));
}
