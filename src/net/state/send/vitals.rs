//! What the records of this machine carry of its craft: the glow of an entry around it and how
//! hurt each part of the robot is.

use abstractions_net::wire::{Frame, HARM_PARTS, RobotBody, VitalsBody};

use abstractions_net::wire::Kind;

use super::{Here, Outbox, Pilot, Scene, Sent, Stamp};
use crate::condition::Condition;
use crate::ship::WarpState;

/// Adds `body` to `out`. A vitals record has no position, so the frame only names where it is.
pub fn encode(body: &VitalsBody, _: &Frame, out: &mut Vec<u8>) {
    body.encode(out);
}

impl Outbox {
    /// Adds the vitals record to the batch when it is due.
    pub(super) fn offer_vitals(&mut self, stamp: &Stamp, vitals: Option<Sent<VitalsBody>>) {
        self.offer(stamp, (Kind::Vitals, vitals), encode);
    }
}

impl Scene<'_> {
    /// Returns the glow of the entry around the craft of this machine, from 0 to 1.
    fn glow(&self) -> f32 {
        self.heat.as_deref().map_or(0.0, |heat| heat.glow.heat)
    }

    /// Returns the vitals record that goes with the robot record `walker`: its glow only while the
    /// player is not seated in the ship, whose record carries it then, and how hurt each part is.
    pub(super) fn vitals(
        &self,
        here: &Here,
        walker: Option<Sent<RobotBody>>,
    ) -> Option<Sent<VitalsBody>> {
        let whole = self.condition.as_deref().map(Condition::saved);
        let whole = whole.unwrap_or([1.0; HARM_PARTS]);
        walker.map(|sent| Sent {
            frame: sent.frame,
            body: VitalsBody {
                harm: whole.map(|health| 1.0 - health),
                heat: if here.seated { 0.0 } else { self.glow() },
            },
            rest: true,
        })
    }

    /// Returns what the ship of this machine flies with: the warp drive, the pilot's hands and the
    /// glow of an entry, which the ship carries while the player is seated in it.
    pub(super) fn flown(&self, here: &Here) -> (WarpState, Pilot, f32) {
        let heat = if here.seated { self.glow() } else { 0.0 };
        (self.drive(), self.pilot(), heat)
    }
}
