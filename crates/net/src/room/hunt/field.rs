//! What a joined player says about the field besides hunting: blows on robots, wounds, impacts,
//! attacks of animals and rocks taken. The room tells the right players and keeps what a player
//! who joins later must see.

use super::{ROCKS_MAX, WOUNDS_MAX};
use crate::host::{ConnId, Outbox};
use crate::log::PlayerId;
use crate::room::Room;
use crate::wire::{
    AnimalKey, Attack, Blow, Event, Impact, PARTS_MAX, Rock, SKEW, ServerMsg, Wound, finite,
};

impl Room {
    /// Tells the victim of `blow` that `by` struck its robot. Nobody else hears it, and a blow that
    /// is not a number, or on a player who is not in, is dropped. The victim decides what it does.
    pub(super) fn blow(&mut self, by: PlayerId, blow: Blow, out: &mut Outbox) {
        let sane = finite(&[blow.joules, blow.momentum]) && finite(&blow.at) && finite(&blow.dir);
        let seat = self
            .seats
            .iter()
            .flatten()
            .find(|seat| seat.info.id == blow.victim);
        if let (true, Some(seat)) = (sane && blow.victim != by, seat) {
            out.send(seat.conn, ServerMsg::Hunt(Event::Blow { by, blow }));
        }
    }

    /// Keeps a wound opened on a part of `animal` and tells the others. A wound on a dead animal,
    /// or one that is not a number, is dropped. An opening time farther than [`SKEW`] from `clock`
    /// is clamped to `clock`.
    pub(super) fn wounded(
        &mut self,
        (conn, clock): (ConnId, f64),
        (animal, wound): (AnimalKey, Wound),
        out: &mut Outbox,
    ) {
        let sane = wound.part < PARTS_MAX
            && finite(&[wound.depth, wound.width])
            && wound.opened.is_finite();
        if !sane || self.hunting.dead(&animal) || self.hunting.wounds.len() >= WOUNDS_MAX {
            return;
        }
        let opened = if (wound.opened - clock).abs() > SKEW {
            clock
        } else {
            wound.opened
        };
        let wound = Wound {
            opened,
            plugged: false,
            ..wound
        };
        self.hunting.wounds.push((animal, wound));
        self.dirty = true;
        self.send_others(conn, &ServerMsg::Hunt(Event::Wound { animal, wound }), out);
    }

    /// Marks the wounds of `part` of `animal` as plugged and tells the others, when one was open.
    pub(super) fn plugged(
        &mut self,
        conn: ConnId,
        (animal, part): (AnimalKey, u8),
        out: &mut Outbox,
    ) {
        let mut any = false;
        for (at, wound) in &mut self.hunting.wounds {
            if *at == animal && wound.part == part && !wound.plugged {
                wound.plugged = true;
                any = true;
            }
        }
        if any {
            self.dirty = true;
            self.send_others(conn, &ServerMsg::Hunt(Event::Plug { animal, part }), out);
        }
    }

    /// Tells the others where a swing or a throw landed.
    pub(super) fn impact(&mut self, conn: ConnId, impact: Impact, out: &mut Outbox) {
        let sane = finite(&[impact.joules]) && impact.at.iter().all(|x| x.is_finite());
        if sane {
            self.send_others(conn, &ServerMsg::Hunt(Event::Impact(impact)), out);
        }
    }

    /// Tells the others that an animal starts an attack.
    pub(super) fn attack(&mut self, conn: ConnId, attack: Attack, out: &mut Outbox) {
        if attack.started.is_finite() {
            self.send_others(conn, &ServerMsg::Hunt(Event::Attack(attack)), out);
        }
    }

    /// Adds a piece to what was taken of its rock and tells the others, when something of the rock
    /// is left and the room has room for it. The taken mass never passes the whole.
    pub(super) fn rock(&mut self, conn: ConnId, rock: Rock, out: &mut Outbox) {
        let key = (rock.planet, rock.id);
        let known = self.hunting.rocks.contains_key(&key);
        if rock.grams == 0 || rock.whole == 0 || !(known || self.hunting.rocks.len() < ROCKS_MAX) {
            return;
        }
        let entry = self.hunting.rocks.entry(key).or_insert((0, rock.whole));
        if entry.0 >= entry.1 {
            return;
        }
        entry.0 = entry.0.saturating_add(rock.grams).min(entry.1);
        self.dirty = true;
        self.send_others(conn, &ServerMsg::Hunt(Event::Rock(rock)), out);
    }
}
