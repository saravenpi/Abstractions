//! The sound of a blow, boiled down to what crosses to the audio thread: how loud, how high and how
//! bright. Where it is travels with it as a [`crate::audio::Sound`].

use super::impact::{Impact, Whoosh};
use super::tune::{BRIGHT, FAST, HEAVY, JOULES_MAX, RING, RING_HZ, WHOOSH_HZ};

/// What kind of sound it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A blow that landed: a thud, a ring on something hard, a wet noise when it cut.
    Clean,
    /// A blow that glanced off: one short high tick.
    Tick,
    /// A tool through the air.
    Whoosh,
}

/// One sound of a blow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitSound {
    /// Which sound.
    pub kind: Kind,
    /// How loud, 0 to 1.
    pub level: f32,
    /// Ring of what was struck, or centre of the whoosh, in hertz.
    pub hz: f32,
    /// How bright, 0 (a thud) to 1 (a ring).
    pub hard: f32,
    /// True when a short wet noise goes with it.
    pub wet: bool,
}

/// Returns the loudness, 0 to 1, of a blow of `joules`: the logarithm of the energy over that of
/// [`JOULES_MAX`].
pub fn loudness(joules: f32) -> f32 {
    (joules.max(0.0).ln_1p() / JOULES_MAX.ln_1p()).clamp(0.0, 1.0)
}

/// Returns the stiffness of the contact of two bodies of `a` and `b` gigapascals: two springs in
/// series, so a soft body deadens a hard one.
pub fn contact(a: f32, b: f32) -> f32 {
    1.0 / (1.0 / a.max(1e-3) + 1.0 / b.max(1e-3))
}

/// Returns the sound of `impact`: loud with the logarithm of the energy,
/// ringing at a pitch that follows the square root of the contact stiffness over the cube root of the
/// mass struck, brighter the stiffer the contact, wet only when the blow cut.
pub fn of_impact(impact: &Impact) -> HitSound {
    let eff = contact(impact.struck.stiffness, impact.tool.stiffness);
    let hz = RING_HZ * (eff / 10.0).sqrt() / impact.struck.mass.max(0.01).cbrt();
    let scale = (eff / BRIGHT[0]).ln() / (BRIGHT[1] / BRIGHT[0]).ln();
    HitSound {
        kind: if impact.deflected {
            Kind::Tick
        } else {
            Kind::Clean
        },
        level: loudness(impact.joules),
        hz: hz.clamp(RING[0], RING[1]),
        hard: scale.clamp(0.0, 1.0),
        wet: impact.cut && !impact.deflected,
    }
}

/// Returns the whoosh of `swing`: louder with the tip speed and the inertia of the tool, and lower
/// the heavier the tool is.
pub fn of_whoosh(swing: &Whoosh) -> HitSound {
    let fast = (swing.speed / FAST).clamp(0.0, 1.0);
    let heavy = swing.inertia / (swing.inertia + HEAVY);
    HitSound {
        kind: Kind::Whoosh,
        level: (0.3 + 0.7 * fast) * (0.4 + 0.6 * heavy),
        hz: WHOOSH_HZ * (1.0 - 0.7 * heavy) * (0.6 + 0.4 * fast),
        hard: 0.0,
        wet: false,
    }
}
