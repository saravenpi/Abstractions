//! Outcomes: what a swing came to, from the data of the contact, and what each one does to the tool,
//! the hitstop, the camera and the animal.
//!
//! - clean: the edge or the point reached its target and the blow went home;
//! - deflected: the angle to the surface was shallow or a hard layer refused the edge, so the tool
//!   glances off along the surface, loses part of its speed and carries on;
//! - stopped: a blunt blow the body took, so the tool stops dead and rebounds a little;
//! - miss: no contact, so the tool carries on through the follow-through and takes longer to bring back.

use super::act::{Act, Timing};
use super::tune::{BOUNCE, BOUNCE_HALF, GLANCE, KICK, MISS_RECOVER, REBOUND};

/// What a swing came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Outcome {
    /// The blow went home.
    Clean,
    /// The tool glanced off.
    Deflected,
    /// A blunt blow stopped the tool.
    Stopped,
    /// The swing met nothing.
    #[default]
    Miss,
}

/// What the tool met: whether it was blunt, whether a hard layer refused its edge, and the cosine of
/// the angle between its way and the normal of the surface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Contact {
    /// True when a face or a mass struck, not an edge or a point.
    pub blunt: bool,
    /// True when an edge met a layer at least as hard as itself.
    pub refused: bool,
    /// How square it met the surface, 0 along it to 1 straight on.
    pub cos: f32,
}

/// Returns what a swing came to: a miss with no contact, deflected when it met the surface at a
/// shallow angle (under [`GLANCE`]) or an edge met a layer that refused it, stopped when a blunt blow
/// met it square, clean otherwise.
pub fn classify(contact: Option<Contact>) -> Outcome {
    let Some(c) = contact else {
        return Outcome::Miss;
    };
    if c.cos < GLANCE || (!c.blunt && c.refused) {
        Outcome::Deflected
    } else if c.blunt {
        Outcome::Stopped
    } else {
        Outcome::Clean
    }
}

/// Returns the restitution of two bodies of `a` and `b` gigapascals: [`BOUNCE`] times the stiffness of
/// their contact over itself and [`BOUNCE_HALF`], so flesh gives nothing back and bone on stone a good
/// deal.
pub fn rebound(a: f32, b: f32) -> f32 {
    let eff = 1.0 / (1.0 / a.max(1e-3) + 1.0 / b.max(1e-3));
    BOUNCE * eff / (eff + BOUNCE_HALF)
}

impl Outcome {
    /// Returns the share of the hitstop of a clean blow that this outcome gets: none for a miss, a
    /// short one for a glance.
    pub fn hitstop(self) -> f32 {
        match self {
            Outcome::Clean | Outcome::Stopped => 1.0,
            Outcome::Deflected => 0.4,
            Outcome::Miss => 0.0,
        }
    }

    /// Returns the share of the camera kick of a clean blow that this outcome gets.
    pub fn kick(self) -> f32 {
        match self {
            Outcome::Clean => 1.0,
            Outcome::Stopped => 1.2,
            Outcome::Deflected => 0.4,
            Outcome::Miss => 0.0,
        }
    }

    /// Returns the share of the flinch and of the harm of a clean blow that this outcome gets.
    pub fn harm(self) -> f32 {
        match self {
            Outcome::Clean | Outcome::Stopped => 1.0,
            Outcome::Deflected => 0.25,
            Outcome::Miss => 0.0,
        }
    }
}

/// Returns the act a swing takes up on meeting `outcome` at `share` of its path, when the target gave
/// back a kickback stun of `stun` seconds and the contact bounces by `bounce`: a clean blow stuns for
/// the kickback, a stopped one holds the tool and sets it back along the path by [`REBOUND`] times the
/// bounce, a glance carries on.
pub fn after(outcome: Outcome, stun: f32, share: f32, bounce: f32) -> Option<Act> {
    match outcome {
        Outcome::Clean => (stun > 0.0).then_some(Act::Stun(stun, share)),
        Outcome::Stopped => Some(Act::Stun(
            stun.max(0.5 * KICK),
            (share - REBOUND * bounce / BOUNCE).max(0.0),
        )),
        Outcome::Deflected | Outcome::Miss => None,
    }
}

impl Timing {
    /// Returns this timing as `outcome` leaves it: a miss recovers [`MISS_RECOVER`] times longer and
    /// carries on past the follow-through in full, a clean blow was braked by its target and carries on
    /// a quarter as far, a glance three quarters, a stopped tool not at all.
    pub fn braked(self, outcome: Outcome) -> Timing {
        let (over, recover) = match outcome {
            Outcome::Miss => (1.0, MISS_RECOVER),
            Outcome::Deflected => (0.7, 1.0),
            Outcome::Clean => (0.25, 1.0),
            Outcome::Stopped => (0.0, 1.0),
        };
        Timing {
            over: self.over * over,
            recover: self.recover * recover,
            ..self
        }
    }
}
