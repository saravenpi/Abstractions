//! Tests of the ring buffer of wakes: it wraps, entries expire and an idle sea costs nothing.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::block::{FOAM_ROWS, FoamWakes, SEA_ROWS, SeaWakes};
use super::packet::Packet;
use super::tune::{LIFE, WAKES};
use super::wakes::{Wake, Wakes};

/// Returns a wake laid at clock `born` at sea position `(x, 0, 0)` going along +X.
fn wake(born: f32, x: f64) -> Wake {
    Wake {
        at: DVec3::new(x, 0.0, 0.0),
        ahead: DVec3::new(x + 1.0, 0.0, 0.0),
        over: DVec3::new(x, 1.0, 0.0),
        packet: Packet::moving(20.0, 9.81, 2.0, (0.2, born)),
    }
}

/// Returns the sea coordinates of a frame position: its x and z.
fn locate(at: DVec3) -> Vec2 {
    Vec2::new(at.x as f32, at.z as f32)
}

/// Writing more entries than the buffer holds keeps its size and drops the oldest first.
#[test]
fn the_buffer_wraps() {
    let mut wakes = Wakes::default();
    for i in 0..WAKES + 8 {
        wakes.push(wake(i as f32 * 0.01, i as f64));
    }
    assert_eq!(wakes.len(), WAKES);
    let now = (WAKES + 8) as f32 * 0.01;
    let oldest = wakes
        .live(now)
        .map(|wake| wake.packet.born)
        .fold(f32::MAX, f32::min);
    assert!((oldest - 0.08).abs() < 1e-6, "{oldest}");
    assert_eq!(wakes.live(now).count(), WAKES);
}

/// An entry shows for [`LIFE`] seconds and not before it was laid.
#[test]
fn entries_expire() {
    let mut wakes = Wakes::default();
    assert!(wakes.is_empty() && !wakes.stirred(0.0));
    wakes.push(wake(1.0, 0.0));
    assert!(!wakes.stirred(0.5));
    assert!(wakes.stirred(1.0));
    assert!(wakes.stirred(1.0 + LIFE - 0.01));
    assert!(!wakes.stirred(1.0 + LIFE));
}

/// With no live entry the blocks are empty and raise nothing anywhere.
#[test]
fn no_live_entry_moves_nothing() {
    let mut wakes = Wakes::default();
    wakes.push(wake(0.0, 0.0));
    let now = LIFE + 1.0;
    let sea = SeaWakes::pack(&wakes, now, locate);
    let foam = FoamWakes::pack(&wakes, now, |at| at.as_vec3());
    assert_eq!((sea.count.x, foam.count.x), (0.0, 0.0));
    for i in 0..50 {
        assert_eq!(sea.height(Vec2::new(i as f32, 3.0), now), 0.0);
    }
}

/// The blocks pack the live entries from the first row and count them, however many were laid.
#[test]
fn the_blocks_pack_the_live_entries() {
    let mut wakes = Wakes::default();
    for i in 0..5 * WAKES {
        wakes.push(wake(i as f32 * 0.1, i as f64));
    }
    let now = 5.0 * WAKES as f32 * 0.1;
    let live = wakes.live(now).count();
    let sea = SeaWakes::pack(&wakes, now, locate);
    let foam = FoamWakes::pack(&wakes, now, |at| at.as_vec3());
    assert_eq!(sea.count.x as usize, live);
    assert_eq!(foam.count.x as usize, live);
    assert!(live > 0 && live <= WAKES);
    assert_eq!(sea.rows.len(), SEA_ROWS * WAKES);
    assert_eq!(foam.rows.len(), FOAM_ROWS * WAKES);
    assert_eq!(foam.rows[2].truncate(), Vec3::Y);
    assert_eq!(foam.rows[1].truncate(), Vec3::X);
}
