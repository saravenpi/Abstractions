//! The ground as the game holds it: alone it is the authority, in a session every drop and take is
//! an ask of the server, and the answers come back here.

use abstractions_net::ground::Placed;
use abstractions_net::wire::{Act, Lying};

use super::hand::{restore, stow, wanted};
use super::{Ground, Shed, Thing, codec};
use crate::craft::pack::Pack;
use crate::sim::PlayerId;

/// The line when the thing asked for is gone.
pub const TAKEN: &str = "Already taken";

/// The line when the ground cannot take a thing put down.
pub const NO_ROOM: &str = "No room on the ground here";

impl Ground {
    /// Puts `lying`, the record of `thing`, down. Alone the pile takes it, or the thing goes back
    /// into `pack` and the line says why; in a session it is asked of the server under a new tag.
    pub fn put(&mut self, lying: Lying, thing: Thing, pack: &mut Pack) -> Option<&'static str> {
        if self.remote {
            self.tag = self.tag.wrapping_add(1).max(1);
            let tag = self.tag;
            self.asks.push(Act::Drop(Lying { tag, ..lying }));
            self.pending.push((tag, thing));
            return None;
        }
        match self.pile.lay(lying) {
            Placed::Refused => {
                restore(pack, thing);
                Some(NO_ROOM)
            }
            Placed::Laid(_) | Placed::Joined(_) => None,
        }
    }

    /// Takes the thing `id` into `pack`, as much as `room` kilograms allow, what does not fit into
    /// `shed`. In a session it asks the server, once until it answers. Else the line why not.
    pub fn reach(
        &mut self,
        id: u64,
        room: f32,
        (pack, shed): (&mut Pack, &mut Shed),
    ) -> Result<(), String> {
        let lying = self.pile.get(id).ok_or_else(|| TAKEN.to_string())?;
        let name = codec::thing(lying).map_or("thing", |thing| codec::word(&thing));
        let mass = wanted(lying, room, name)?;
        if self.remote {
            if self.asked.is_none() {
                self.asks.push(Act::Take { id, mass });
                self.asked = Some(id);
            }
            return Ok(());
        }
        let took = self.pile.take(id, mass).ok_or_else(|| TAKEN.to_string())?;
        self.granted(&took.got, room, (pack, shed));
        Ok(())
    }

    /// Puts what was granted, `got`, into `pack` within `room` kilograms, the rest into `shed`.
    pub fn granted(&mut self, got: &Lying, room: f32, (pack, shed): (&mut Pack, &mut Shed)) {
        self.asked = None;
        if let Some(thing) = codec::thing(got) {
            stow(thing, room, pack, shed);
        }
    }

    /// Gives the drop of `tag`, which the server could not take, back to `pack`, and returns the
    /// line that says so.
    pub fn kept(&mut self, tag: u32, pack: &mut Pack) -> &'static str {
        if let Some(at) = self.pending.iter().position(|(held, _)| *held == tag) {
            restore(pack, self.pending.remove(at).1);
        }
        NO_ROOM
    }

    /// Forgets the thing `id`, which another player took first, and returns the line that says so.
    pub fn denied(&mut self, id: u64) -> &'static str {
        self.asked = None;
        self.pile.forget(id);
        TAKEN
    }

    /// Keeps `lying` as the server says it lies, and forgets the drop of this player, `you`, that it
    /// answers.
    pub fn heard(&mut self, lying: Lying, you: Option<PlayerId>) {
        if lying.tag != 0 && you == Some(lying.by) {
            self.pending.retain(|(tag, _)| *tag != lying.tag);
        }
        self.pile.told(lying);
    }

    /// Keeps the ground alone again: the drops the server never answered come back to `pack`.
    pub fn release(&mut self, pack: &mut Pack) {
        for (_, thing) in std::mem::take(&mut self.pending) {
            restore(pack, thing);
        }
        (self.remote, self.asked) = (false, None);
        self.asks.clear();
    }
}
