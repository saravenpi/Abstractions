//! Tests of state records and batches: sizes, reading without decoding, splitting, bad bytes.

use super::body_tests::{planet, robot};
use super::{AT_REST, Frame, Head, Kind, Record, push, push_within, records, split};
use crate::log::PlayerId;

/// Returns the head of a robot record of player `player` on the test planet.
fn head(player: u32) -> Head {
    Head {
        player: PlayerId(player),
        kind: Kind::Robot,
        frame: Frame::Surface(planet()),
        time: 86_400.0 * 365.0 + f64::from(player) * 0.033,
        flags: 0,
    }
}

/// Returns the packed body of the test robot.
fn body() -> Vec<u8> {
    let mut bytes = Vec::new();
    robot().encode(&Frame::Surface(planet()), &mut bytes);
    bytes
}

/// Sixteen robot records fit in one datagram of 1162 bytes, the size measured on loopback, and
/// one record takes 68 bytes at most: a 22 byte head and the 46 byte robot body of protocol 3, so
/// sixteen also fit the 1,100 byte budget of the room.
#[test]
fn sixteen_robots_fit_in_one_datagram() {
    let body = body();
    let mut batch = Vec::new();
    for player in 0..16 {
        let record = Record {
            head: head(player),
            body: &body,
        };
        push(&mut batch, &record);
    }
    assert!(batch.len() <= 1162, "{} bytes", batch.len());
    assert!(batch.len() <= 16 * 68, "{} bytes", batch.len());
    assert_eq!(records(&batch).count(), 16);
}

/// A batch gives its records back in order, heads read and bodies untouched.
#[test]
fn a_batch_gives_its_records_back() {
    let body = body();
    let resting = Head {
        kind: Kind::Ship,
        frame: Frame::System(planet().star),
        flags: AT_REST,
        ..head(7)
    };
    let mut batch = Vec::new();
    push(
        &mut batch,
        &Record {
            head: head(3),
            body: &body,
        },
    );
    push(
        &mut batch,
        &Record {
            head: resting,
            body: &[1, 2, 3],
        },
    );
    let got: Vec<Record<'_>> = records(&batch).collect();
    assert_eq!(got.len(), 2);
    assert_eq!((got[0].head, got[0].body), (head(3), &body[..]));
    assert_eq!((got[1].head, got[1].body), (resting, &[1u8, 2, 3][..]));
    assert!(got[1].head.at_rest() && !got[0].head.at_rest());
}

/// Reading stops at a record that is cut short, and keeps the whole ones before it.
#[test]
fn a_cut_batch_keeps_its_whole_records() {
    let body = body();
    let mut batch = Vec::new();
    for player in 0..3 {
        push(
            &mut batch,
            &Record {
                head: head(player),
                body: &body,
            },
        );
    }
    batch.truncate(batch.len() - 5);
    assert_eq!(records(&batch).count(), 2);
    assert_eq!(records(&[]).count(), 0);
    assert_eq!(records(&[0xff; 9]).count(), 0);
}

/// A record that would pass the limit is not added, and the batch stays as it was.
#[test]
fn a_record_past_the_limit_is_not_added() {
    let body = body();
    let record = Record {
        head: head(1),
        body: &body,
    };
    let mut batch = Vec::new();
    assert!(push_within(&mut batch, &record, 100));
    let one = batch.clone();
    assert!(!push_within(&mut batch, &record, 100));
    assert_eq!(batch, one);
}

/// Splitting keeps every record, in order, in batches under the limit, and drops a record
/// that cannot fit alone.
#[test]
fn splitting_keeps_the_order_and_the_limit() {
    let body = body();
    let huge = vec![0; 400];
    let mut all: Vec<Record<'_>> = (0..16)
        .map(|p| Record {
            head: head(p),
            body: &body,
        })
        .collect();
    all.insert(
        5,
        Record {
            head: head(99),
            body: &huge,
        },
    );
    let batches = split(all, 300);
    assert!(batches.len() >= 4 && batches.iter().all(|batch| batch.len() <= 300));
    let players: Vec<u32> = batches
        .iter()
        .flat_map(|batch| records(batch).map(|record| record.head.player.0))
        .collect();
    assert_eq!(players, (0..16).collect::<Vec<u32>>());
    assert!(split(Vec::new(), 300).is_empty());
}
