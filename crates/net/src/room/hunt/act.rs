//! What a joined player says about hunting: checked, kept and told to the others.

use crate::host::{ConnId, Outbox};
use crate::log::PlayerId;
use crate::room::Room;
use crate::room::conn::HUNTS_PER_SECOND;
use crate::wire::{Act, AnimalKey, Event, Hit, Kill, PARTS_MAX, ServerMsg};

/// Most joules a blow may carry on the wire. A larger number is held to it.
const JOULES_MAX: f32 = 1.0e6;

impl Room {
    /// Takes the hunting act of the player on `conn`, unless the player is not in or sent too
    /// many this second.
    pub(in crate::room) fn hunt(&mut self, conn: ConnId, act: Act, now: f64, out: &mut Outbox) {
        let me = self
            .seat_of(conn)
            .and_then(|slot| self.seats[slot].as_ref())
            .map(|seat| seat.info.id);
        let open = self.conns.get_mut(&conn);
        let allowed = open.is_some_and(|open| open.hunt.allow(now, HUNTS_PER_SECOND));
        let (Some(me), true) = (me, allowed) else {
            return;
        };
        let clock = self.clock(now);
        match act {
            Act::Hit(hit) => self.struck(conn, (me, clock), hit, out),
            Act::Broke { animal, part } => self.cracked(conn, (me, clock), (animal, part), out),
            Act::Kill(kill) => self.killed((me, clock), kill, out),
            Act::Carve { animal, part } => self.carved(conn, me, (animal, part), out),
            Act::Stick(spear) => self.stuck(conn, me, spear, out),
            Act::Pull(key) => self.pulled(conn, me, key, out),
            Act::Held(item) => self.held(conn, me, item, out),
            Act::Blow(blow) => self.blow(me, blow, out),
            Act::Wound { animal, wound } => self.wounded(conn, (animal, wound), out),
            Act::Plug { animal, part } => self.plugged(conn, (animal, part), out),
            Act::Impact(impact) => self.impact(conn, impact, out),
            Act::Attack(attack) => self.attack(conn, attack, out),
            Act::Rock(rock) => self.rock(conn, rock, out),
            Act::Drop(lying) => self.dropped(conn, (me, clock), lying, out),
            Act::Take { id, mass } => self.picked(conn, me, (id, mass), out),
        }
    }

    /// Notes the blow of `by` at universe time `clock` and tells the others.
    fn struck(&mut self, conn: ConnId, (by, clock): (PlayerId, f64), hit: Hit, out: &mut Outbox) {
        let sane = hit.part < PARTS_MAX && hit.joules.is_finite();
        if !sane || self.hunting.dead(&hit.animal) {
            return;
        }
        let hit = Hit {
            joules: hit.joules.clamp(0.0, JOULES_MAX),
            ..hit
        };
        self.hunting.strike(hit.animal, by, clock);
        self.send_others(conn, &ServerMsg::Hunt(Event::Hit { by, hit }), out);
    }

    /// Notes that `part` of `animal` broke, and tells the others when it was whole.
    fn cracked(
        &mut self,
        conn: ConnId,
        (_, clock): (PlayerId, f64),
        (animal, part): (AnimalKey, u8),
        out: &mut Outbox,
    ) {
        let alive = part < PARTS_MAX && !self.hunting.dead(&animal);
        if alive && self.hunting.crack(animal, part, clock) {
            let msg = ServerMsg::Hunt(Event::Broke { animal, part });
            self.send_others(conn, &msg, out);
        }
    }

    /// Lays the carcass of the kill `kill` of `by` and tells everyone, the killer included, so
    /// all agree on its time. A kill of an animal already dead, or one that is not a number, is
    /// dropped.
    fn killed(&mut self, (by, clock): (PlayerId, f64), kill: Kill, out: &mut Outbox) {
        let finite = kill.at.iter().all(|x| x.is_finite());
        if !finite || !kill.since.is_finite() || !kill.lasts.is_finite() {
            return;
        }
        let Some(carcass) = self.hunting.kill(kill, by, clock) else {
            return;
        };
        self.dirty = true;
        self.broadcast(&ServerMsg::Hunt(Event::Killed(carcass)), out);
    }

    /// Takes `part` off the carcass of `animal` for `by` and tells the others, when it was
    /// still there.
    fn carved(
        &mut self,
        conn: ConnId,
        by: PlayerId,
        (animal, part): (AnimalKey, u8),
        out: &mut Outbox,
    ) {
        if part < PARTS_MAX && self.hunting.carve(animal, part) {
            self.dirty = true;
            let msg = ServerMsg::Hunt(Event::Carved { animal, part, by });
            self.send_others(conn, &msg, out);
        }
    }
}
