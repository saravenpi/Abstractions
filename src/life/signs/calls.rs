//! Calls: the voice of each species and when herds call, as pure functions of the herd, the hunt
//! and universe time, so every player hears a herd call at the same moment.
//!
//! The pitch of a call falls with body size: the dominant frequency of animal calls scales about as
//! the mass to the power -0.4, from about 1 kHz for a kilogram (Fletcher 2004, "A simple
//! frequency-scaling rule for animal communication", JASA 115, 2334-2338). The timbre comes from the
//! guild and the rung: shelled life clicks and buzzes, soft life gurgles, animals with skeletons voice.

use crate::life::herd::{Herd, HerdState};
use crate::life::hunt::{Hunt, Outcome};
use crate::life::{Guild, Habitat, Rung, Species};
use crate::universe::hash::{Rng, child};

/// Frequency in hertz of the call of a one kilogram animal.
const HZ_PER_KG: f32 = 1000.0;

/// Exponent of the mass in the frequency of a call.
const HZ_EXPONENT: f32 = -0.4;

/// Lowest and highest frequency in hertz of a call.
const HZ_SPAN: (f32, f32) = (45.0, 4000.0);

/// Seconds between two chances of a contact call of a herd of one member; a herd of `n` calls `n`
/// times as often, up to [`CONTACT_FASTEST`].
const CONTACT_GAP: f64 = 90.0;

/// Fewest seconds between two chances of a contact call of one herd.
const CONTACT_FASTEST: f64 = 12.0;

/// Seconds between two alarm calls of a chased herd.
const ALARM_GAP: f64 = 1.6;

/// Salt of the contact calls.
const CONTACT: u64 = 0xCA_11ED;

/// How a call sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Timbre {
    /// A low, falling moan: grazers.
    Low,
    /// A round hoot: browsers.
    Hoot,
    /// A whistle gliding up: flyers, seed and fruit eaters, filter feeders.
    Whistle,
    /// A rough growl: hunters.
    Growl,
    /// A breathy hiss: ambushers.
    Hiss,
    /// A gurgling croak: scavengers and soft life.
    Croak,
    /// A train of clicks: shelled life and the smallest.
    Click,
}

/// Every timbre, in code order.
pub const TIMBRES: [Timbre; 7] = [
    Timbre::Low,
    Timbre::Hoot,
    Timbre::Whistle,
    Timbre::Growl,
    Timbre::Hiss,
    Timbre::Croak,
    Timbre::Click,
];

/// Why an animal calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cry {
    /// Keeps the herd together: now and then.
    Contact,
    /// Warns the herd: when it flees.
    Alarm,
    /// A hunter on the chase.
    Chase,
}

/// The voice of a species.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voice {
    /// Main frequency in hertz.
    pub hz: f32,
    /// How it sounds.
    pub timbre: Timbre,
    /// Notes in one call, 1 to 4: more for evolved life.
    pub notes: u8,
}

/// One call: who calls, why, and the universe time it starts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Calling {
    /// Identity of the calling herd.
    pub herd: u64,
    /// Species index of the caller.
    pub species: u16,
    /// Why it calls.
    pub cry: Cry,
    /// Universe time in seconds the call starts.
    pub at: f64,
}

/// Returns the timbre of `species` at `rung`.
fn timbre(species: &Species, rung: Rung) -> Timbre {
    match (rung, species.role.guild, species.role.habitat) {
        (Rung::Mats | Rung::Soft, ..) => Timbre::Croak,
        (Rung::Shelled, ..) => Timbre::Click,
        (_, Guild::Hunter, _) => Timbre::Growl,
        (_, Guild::Ambusher, _) => Timbre::Hiss,
        (_, Guild::Scavenger, _) => Timbre::Croak,
        (_, Guild::Decomposer | Guild::Parasite | Guild::Producer, _) => Timbre::Click,
        (_, _, Habitat::Air | Habitat::Water) => Timbre::Whistle,
        (_, Guild::Grazer, _) => Timbre::Low,
        (_, Guild::Browser, _) => Timbre::Hoot,
        _ => Timbre::Whistle,
    }
}

/// Returns the voice of `species` on a planet whose life reached `rung`: lower for bigger animals.
pub fn voice(species: &Species, rung: Rung) -> Voice {
    let mass = species.history.mass.max(1e-4);
    let hz = (HZ_PER_KG * mass.powf(HZ_EXPONENT)).clamp(HZ_SPAN.0, HZ_SPAN.1);
    let timbre = timbre(species, rung);
    let notes = match (timbre, rung) {
        (Timbre::Click, _) => 4,
        (_, Rung::Old) => 3,
        (_, Rung::Skeletal) => 2,
        _ => 1,
    };
    Voice { hz, timbre, notes }
}

/// Returns the contact call `herd` makes within the universe times `(from, to]`, if any: one chance
/// per window of the herd's call gap, by a seeded roll, never while it sleeps.
pub fn contact(herd: &Herd, (from, to): (f64, f64)) -> Option<Calling> {
    let chance = match herd.state {
        HerdState::Sleep | HerdState::Stalk => return None,
        HerdState::Rest => 0.3,
        _ => 0.8,
    };
    let gap = (CONTACT_GAP / f64::from(herd.count.max(1))).max(CONTACT_FASTEST);
    let window = |k: i64| {
        let mut rng = Rng::new(child(herd.id ^ CONTACT, k as u64));
        let at = (k as f64 + 0.9 * rng.unit()) * gap;
        (rng.unit() < chance).then_some(at)
    };
    let (first, last) = ((from / gap).floor() as i64, (to / gap).floor() as i64);
    let at = (first..=last)
        .filter_map(window)
        .find(|at| from < *at && *at <= to)?;
    Some(Calling {
        herd: herd.id,
        species: herd.species,
        cry: Cry::Contact,
        at,
    })
}

/// Calls `heard` with every call of `hunt` within the universe times `(from, to]`: the hunters cry
/// as the chase starts and at a kill, and the prey calls the alarm through the chase.
pub fn cries(hunt: &Hunt, (from, to): (f64, f64), mut heard: impl FnMut(Calling)) {
    let chase = hunt.start + hunt.stalk;
    let end = hunt.outcome_time();
    let hunter = |at| Calling {
        herd: hunt.hunter,
        species: hunt.hunter_species,
        cry: Cry::Chase,
        at,
    };
    let kill = matches!(hunt.outcome, Outcome::Kill { .. }).then_some(end);
    for at in [Some(chase), kill].into_iter().flatten() {
        if from < at && at <= to {
            heard(hunter(at));
        }
    }
    let first = ((from.max(chase) - chase) / ALARM_GAP).floor() as i64;
    let last = ((to.min(end) - chase) / ALARM_GAP).floor() as i64;
    for k in first.max(0)..=last {
        let jitter = Rng::new(child(hunt.prey, k as u64)).unit() * 0.5;
        let at = chase + (k as f64 + jitter) * ALARM_GAP;
        if from < at && at <= to && at <= end {
            heard(Calling {
                herd: hunt.prey,
                species: hunt.prey_species,
                cry: Cry::Alarm,
                at,
            });
        }
    }
}
