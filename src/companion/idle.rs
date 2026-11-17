//! Idle behaviours: what the companion does while the player stands still, and how one is chosen.

use super::face::Expression;
use super::rng::Rng;

/// Something the companion does on its own while the player stands still.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Idle {
    /// Looks left and right.
    LookAround,
    /// Bobs as if humming a tune.
    Hum,
    /// Stretches its hands out.
    Stretch,
    /// Drifts off to inspect something nearby and comes back.
    Inspect,
    /// Turns once on itself, slowly and happily.
    Spin,
    /// Peeks up at the night sky.
    SkyPeek,
    /// Does a little dance.
    Dance,
}

/// How likely each idle is. The weights are uneven on purpose: the last ones are rare treats.
pub const WEIGHTS: [(Idle, f32); 7] = [
    (Idle::LookAround, 30.0),
    (Idle::Hum, 22.0),
    (Idle::Stretch, 16.0),
    (Idle::Inspect, 14.0),
    (Idle::Spin, 9.0),
    (Idle::SkyPeek, 6.0),
    (Idle::Dance, 3.0),
];

impl Idle {
    /// Returns how long the idle lasts, in seconds.
    pub fn seconds(self) -> f32 {
        match self {
            Idle::LookAround => 5.0,
            Idle::Hum => 6.0,
            Idle::Stretch => 3.5,
            Idle::Inspect => 9.0,
            Idle::Spin => 3.2,
            Idle::SkyPeek => 6.5,
            Idle::Dance => 5.0,
        }
    }

    /// Returns the face the idle is done with.
    pub fn face(self) -> Expression {
        match self {
            Idle::LookAround | Idle::Inspect => Expression::Curious,
            Idle::SkyPeek => Expression::Love,
            Idle::Stretch => Expression::Sleepy,
            Idle::Hum | Idle::Spin | Idle::Dance => Expression::Happy,
        }
    }
}

/// Returns true when `idle` may be chosen after `last`: never the same twice, and the sky only at night.
pub fn allowed(idle: Idle, last: Option<Idle>, night: bool) -> bool {
    Some(idle) != last && (night || idle != Idle::SkyPeek)
}

/// Picks the next idle by weight among the allowed ones. The same generator state gives the same pick.
pub fn pick(rng: &mut Rng, last: Option<Idle>, night: bool) -> Idle {
    let open = WEIGHTS
        .iter()
        .filter(|(idle, _)| allowed(*idle, last, night));
    let total: f32 = open.clone().map(|(_, weight)| weight).sum();
    let mut roll = rng.next() * total;
    let mut chosen = Idle::LookAround;
    for (idle, weight) in open {
        chosen = *idle;
        if roll < *weight {
            break;
        }
        roll -= weight;
    }
    chosen
}
