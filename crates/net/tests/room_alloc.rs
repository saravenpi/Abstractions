//! The room's tick does not allocate in steady state: sixty-four players each send a robot
//! record every tick for a thousand ticks, and allocations inside `tick` are counted.
//!
//! This file is its own test binary, so its counting allocator touches no other test.

use std::alloc::{GlobalAlloc, Layout, System};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicUsize, Ordering};

use abstractions_net::host::{ConnId, Host, Outbox, TICK_HZ};
use abstractions_net::ids::BodyId;
use abstractions_net::log::PlayerId;
use abstractions_net::room::{Room, RoomConfig};
use abstractions_net::store::World;
use abstractions_net::wire::{ClientMsg, Frame, Head, Hello, Kind, PROTOCOL, Record, push};

/// The system allocator, counting every allocation and reallocation.
struct Counting;

/// Allocations made so far by the whole test binary.
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    /// Counts the allocation and hands it to the system.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    /// Hands the memory back to the system.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    /// Counts the reallocation and hands it to the system.
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

/// The allocator of this test binary.
#[global_allocator]
static GLOBAL: Counting = Counting;

/// Players in the room.
const PLAYERS: u32 = 64;

/// Ticks run, the first ones warming the buffers up.
const TICKS: u32 = 1000;

/// Ticks given to warm up before allocations count.
const WARM_UP: u32 = 10;

/// Returns a room with sixty-four joined players and their connections.
fn full_room(out: &mut Outbox) -> (Room, Vec<ConnId>) {
    let config = RoomConfig {
        name: "Bench".to_string(),
        password: None,
        cap: 64,
        version: "0.9.0".to_string(),
    };
    let mut room = Room::new(config, World::default(), 0.0);
    let conns: Vec<ConnId> = (1..=PLAYERS).map(|n| ConnId(u64::from(n))).collect();
    for (n, conn) in (1u8..).zip(&conns) {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, n)), 24650);
        room.connected(*conn, addr, 0.0, out);
        let hello = Hello {
            protocol: PROTOCOL,
            name: format!("P{n}"),
            key: [n; 32],
            password: None,
        };
        room.message(*conn, ClientMsg::Hello(hello), 0.0, out);
    }
    (room, conns)
}

/// Writes into `bytes` the robot record of `player` sampled at `time`.
fn robot(bytes: &mut Vec<u8>, player: u32, time: f64) {
    let head = Head {
        player: PlayerId(player),
        kind: Kind::Robot,
        frame: Frame::Surface(BodyId::default()),
        time,
        flags: 0,
    };
    bytes.clear();
    push(
        bytes,
        &Record {
            head,
            body: &[3; 44],
        },
    );
}

/// After a short warm-up, no tick allocates.
#[test]
fn tick_does_not_allocate() {
    let mut out = Outbox::default();
    let (mut room, conns) = full_room(&mut out);
    let mut bytes = Vec::with_capacity(256);
    let mut counted = 0;
    for tick in 0..TICKS {
        let now = f64::from(tick) / TICK_HZ;
        out.clear();
        for (player, conn) in (1..).zip(&conns) {
            robot(&mut bytes, player, now);
            room.datagram(*conn, &bytes, now, &mut out);
        }
        let before = ALLOCATIONS.load(Ordering::Relaxed);
        room.tick(now, &mut out);
        let made = ALLOCATIONS.load(Ordering::Relaxed) - before;
        if tick >= WARM_UP {
            counted += made;
        }
        assert!(!out.ops().is_empty());
    }
    assert_eq!(
        counted,
        0,
        "allocations in {} steady ticks",
        TICKS - WARM_UP
    );
}
