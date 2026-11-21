//! Tests of the ground in a session: what the server tells comes into the ground, the pack and the
//! banner lines.

use abstractions_net::wire::{Event, Lying};
use bevy::math::DVec3;

use super::hear;
use crate::craft::build::Place;
use crate::craft::ground::{Ground, NO_ROOM, Shed, TAKEN, Thing, lying};
use crate::craft::matter::{Mineral, Stuff};
use crate::craft::pack::{Pack, Stack};
use crate::sim::PlayerId;
use crate::universe::BodyId;

/// Returns `kilograms` of basalt.
fn stone(kilograms: f32) -> Thing {
    Thing::Heap(Stack {
        stuff: Stuff::Mineral(Mineral::Basalt),
        material: Mineral::Basalt.material(),
        mass: kilograms,
    })
}

/// Returns the record of `kilograms` of basalt as a server tells it, with id `id`, put down by
/// player 2.
fn basalt(id: u64, kilograms: f32) -> Lying {
    let at = (BodyId::default(), DVec3::new(0.0, 6_400_000.0, 0.0));
    let earth = Place {
        gravity: 9.81,
        air: 1.225,
    };
    let record = lying(&stone(kilograms), at, &earth, 1.0).expect("a record");
    Lying {
        id,
        by: PlayerId(2),
        ..record
    }
}

/// The ground, the pack and what waits to be put down of player 3 in a session.
#[derive(Default)]
struct Side {
    /// The ground, kept by the server.
    ground: Ground,
    /// The pack, empty at first.
    pack: Pack,
    /// What waits to be put down again.
    shed: Shed,
}

impl Side {
    /// Returns the side of a player whose ground the server keeps.
    fn new() -> Self {
        let mut side = Self::default();
        side.ground.remote = true;
        side
    }

    /// Tells `event` to the player, with 25 kg free in the pack, and returns the line of the banner.
    fn tell(&mut self, event: Event) -> Option<&'static str> {
        let into = (&mut self.ground, &mut self.pack, &mut self.shed, 25.0);
        hear(event, Some(PlayerId(3)), into)
    }
}

/// What lies after joining and what another player puts down come into the ground, a thing taken
/// leaves it, a grant fills the pack, and a refusal gives the line of the banner.
#[test]
fn the_server_moves_the_ground_and_the_pack() {
    let mut side = Side::new();
    side.ground.asked = Some(1);
    side.tell(Event::Ground(vec![basalt(1, 2.0), basalt(2, 1.0)]));
    side.tell(Event::Lies(basalt(3, 4.0)));
    assert_eq!(side.ground.pile.lying.len(), 3);
    side.tell(Event::Granted(basalt(1, 2.0)));
    let by = PlayerId(3);
    side.tell(Event::Gone { id: 1, by });
    assert_eq!((side.ground.asked, side.ground.pile.lying.len()), (None, 2));
    assert!((side.pack.mass() - 2.0).abs() < 1e-5);
    assert_eq!(side.tell(Event::Denied(2)), Some(TAKEN));
    assert_eq!(side.tell(Event::Kept(9)), Some(NO_ROOM));
    assert_eq!(side.ground.pile.lying.len(), 1);
}

/// A thing this player put down, told back with the tag of its drop, ends the wait for it; the same
/// tag from another player does not.
#[test]
fn the_answer_to_a_drop_ends_its_wait() {
    let mut side = Side::new();
    side.ground.pending.push((5, stone(1.0)));
    let theirs = Lying {
        tag: 5,
        ..basalt(7, 1.0)
    };
    side.tell(Event::Lies(theirs));
    assert_eq!(side.ground.pending.len(), 1);
    let mine = Lying {
        tag: 5,
        by: PlayerId(3),
        ..basalt(8, 1.0)
    };
    side.tell(Event::Lies(mine));
    assert!(side.ground.pending.is_empty());
    assert_eq!(side.ground.pile.lying.len(), 2);
}
