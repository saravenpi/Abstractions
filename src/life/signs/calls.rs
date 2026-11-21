//! Calls: when herds call and why, as pure functions of the herd, the hunt and universe time, so
//! every player hears a herd call at the same moment and sees the same animal call it.
//!
//! The voice of each species comes from its body (see [`super::voice`]) and the timing of each call
//! from its size (see [`super::rhythm`]). Herds call to keep in touch, more around sunrise and sunset
//! as many animals do in a dawn and a dusk chorus (<https://en.wikipedia.org/wiki/Dawn_chorus>), and
//! young answer their parents. A hunter growls a threat before it charges and cries through the
//! chase, the prey calls the alarm, and a caught animal cries in distress.

pub use super::rhythm::{MOST_NOTES, Rhythm, rhythm};
pub use super::voice::{TIMBRES, Timbre, Voice, voice};

use crate::life::body::FeatureKind;
use crate::life::herd::{Herd, HerdState};
use crate::life::hunt::{Hunt, Outcome};
use crate::life::{Guild, Species};
use crate::universe::hash::{Rng, child};

/// Seconds between two chances of a contact call of a herd of one member; a herd of `n` calls `n`
/// times as often, up to [`CONTACT_FASTEST`].
const CONTACT_GAP: f64 = 90.0;

/// Fewest seconds between two chances of a contact call of one herd.
const CONTACT_FASTEST: f64 = 12.0;

/// How many times more often a herd calls at the height of a chorus.
const CHORUS: f64 = 3.0;

/// Share of the contact calls of a herd with young that are young answering.
const YOUNG_SHARE: f64 = 0.3;

/// Seconds between two alarm calls of a chased herd.
const ALARM_GAP: f64 = 1.6;

/// Seconds before its charge a hunter growls its threat.
const THREAT_LEAD: f64 = 1.5;

/// Seconds between two distress calls of a badly hurt animal; a lightly hurt one calls less.
const DISTRESS_GAP: f64 = 2.5;

/// Salt of the contact calls.
const CONTACT: u64 = 0xCA_11ED;

/// Salt of the distress calls.
const DISTRESS: u64 = 0xD1_57E5;

/// Why an animal calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cry {
    /// Keeps the herd together: now and then, more at dawn and dusk.
    Contact,
    /// Warns the herd: when it flees.
    Alarm,
    /// A hunter on the chase.
    Chase,
    /// Wounded or caught.
    Distress,
    /// A growl with a display: before a charge, or when it stands its ground.
    Threat,
    /// A young keeping in touch with its parents.
    Young,
    /// The last cry of an animal as it dies: a falling, breaking call and a last breath.
    Death,
}

/// Every cry, in code order.
pub const CRIES: [Cry; 7] = [
    Cry::Contact,
    Cry::Alarm,
    Cry::Chase,
    Cry::Distress,
    Cry::Threat,
    Cry::Young,
    Cry::Death,
];

/// One call: who calls, why, and the universe time it starts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Calling {
    /// Identity of the calling herd.
    pub herd: u64,
    /// Species index of the caller.
    pub species: u16,
    /// Slot in the herd of the member that calls; every member calls a [`Cry::whole`] cry.
    pub slot: u8,
    /// Why it calls.
    pub cry: Cry,
    /// Universe time in seconds the call starts.
    pub at: f64,
}

impl Cry {
    /// Returns true when the whole herd calls it together: alarms, chases and threats.
    pub fn whole(self) -> bool {
        matches!(self, Cry::Alarm | Cry::Chase | Cry::Threat)
    }
}

/// Returns how much a herd joins the chorus under `light`, the daylight from 0 at night to 1 by
/// day: 1 halfway through sunrise or sunset, 0 by full day or night.
pub fn chorus(light: f32) -> f32 {
    let light = light.clamp(0.0, 1.0);
    4.0 * light * (1.0 - light)
}

/// Returns true when `species` stands its ground and threatens rather than flees: hunters, ambushers
/// and animals with horns.
pub fn bold(species: &Species) -> bool {
    let horned = species
        .body
        .features
        .iter()
        .any(|f| f.kind == FeatureKind::Horn);
    horned || matches!(species.role.guild, Guild::Hunter | Guild::Ambusher)
}

/// Returns the contact call `herd` makes within the universe times `(from, to]`, if any: one chance
/// per window of the herd's call gap, by a seeded roll, never while it sleeps. `chorus` gives how much
/// the herd joins the chorus at a universe time; it must be the same for every player.
pub fn contact(
    herd: &Herd,
    chorus: &dyn Fn(f64) -> f32,
    (from, to): (f64, f64),
) -> Option<Calling> {
    let chance = match herd.state {
        HerdState::Sleep | HerdState::Stalk => return None,
        HerdState::Rest => 0.3,
        _ => 0.8,
    };
    let gap = (CONTACT_GAP / f64::from(herd.count.max(1))).max(CONTACT_FASTEST) / CHORUS;
    let window = |k: i64| {
        let mut rng = Rng::new(child(herd.id ^ CONTACT, k as u64));
        let start = k as f64 * gap;
        let boost = 1.0 + (CHORUS - 1.0) * f64::from(chorus(start).clamp(0.0, 1.0));
        let at = start + 0.9 * rng.unit() * gap;
        (rng.unit() < chance * boost / CHORUS).then_some((at, rng))
    };
    let (first, last) = ((from / gap).floor() as i64, (to / gap).floor() as i64);
    let (at, mut rng) = (first..=last)
        .filter_map(window)
        .find(|(at, _)| from < *at && *at <= to)?;
    let young = herd.young > 0 && rng.unit() < YOUNG_SHARE;
    let (cry, slots) = match young {
        true => (Cry::Young, herd.young),
        false => (Cry::Contact, herd.base.max(1)),
    };
    Some(Calling {
        herd: herd.id,
        species: herd.species,
        slot: rng.below(u32::from(slots)) as u8,
        cry,
        at,
    })
}

/// Returns the distress call member `slot` of herd `herd` of species `species` makes within the
/// universe times `(from, to]` while it is hurt by `hurt`, from 0 whole to 1 dying: more often the
/// worse it is. Every player who knows the same hurt hears the same calls.
pub fn distress(
    (herd, species, slot): (u64, u16, u8),
    hurt: f32,
    (from, to): (f64, f64),
) -> Option<Calling> {
    if hurt <= 0.0 {
        return None;
    }
    let gap = DISTRESS_GAP / f64::from(hurt.clamp(0.1, 1.0));
    let seed = child(herd ^ DISTRESS, u64::from(slot));
    let at = |k: i64| (k as f64 + 0.5 * Rng::new(child(seed, k as u64)).unit()) * gap;
    let (first, last) = ((from / gap).floor() as i64, (to / gap).floor() as i64);
    let at = (first..=last).map(at).find(|at| from < *at && *at <= to)?;
    Some(Calling {
        herd,
        species,
        slot,
        cry: Cry::Distress,
        at,
    })
}

/// Calls `heard` with every call of `hunt` within the universe times `(from, to]`: the hunters
/// threaten before the chase, cry as it starts and at a kill, and the prey calls the alarm through
/// the chase and cries in distress when caught.
pub fn cries(hunt: &Hunt, span: (f64, f64), mut heard: impl FnMut(Calling)) {
    hunter(hunt, span, &mut heard);
    prey(hunt, span, &mut heard);
}

/// Calls `heard` with the threat and the cries of the hunters of `hunt` within `(from, to]`.
fn hunter(hunt: &Hunt, (from, to): (f64, f64), heard: &mut impl FnMut(Calling)) {
    let chase = hunt.start + hunt.stalk;
    let threat = (chase - THREAT_LEAD > hunt.start).then_some((chase - THREAT_LEAD, Cry::Threat));
    let kill =
        matches!(hunt.outcome, Outcome::Kill { .. }).then_some((hunt.outcome_time(), Cry::Chase));
    for (at, cry) in [threat, Some((chase, Cry::Chase)), kill]
        .into_iter()
        .flatten()
    {
        if from < at && at <= to {
            heard(Calling {
                herd: hunt.hunter,
                species: hunt.hunter_species,
                slot: 0,
                cry,
                at,
            });
        }
    }
}

/// Calls `heard` with the alarms and the distress of the prey of `hunt` within `(from, to]`.
fn prey(hunt: &Hunt, (from, to): (f64, f64), heard: &mut impl FnMut(Calling)) {
    let (chase, end) = (hunt.start + hunt.stalk, hunt.outcome_time());
    let call = |slot, cry, at| Calling {
        herd: hunt.prey,
        species: hunt.prey_species,
        slot,
        cry,
        at,
    };
    let first = ((from.max(chase) - chase) / ALARM_GAP).floor() as i64;
    let last = ((to.min(end) - chase) / ALARM_GAP).floor() as i64;
    for k in first.max(0)..=last {
        let jitter = Rng::new(child(hunt.prey, k as u64)).unit() * 0.5;
        let at = chase + (k as f64 + jitter) * ALARM_GAP;
        if from < at && at <= to && at <= end {
            heard(call(0, Cry::Alarm, at));
        }
    }
    if let Outcome::Kill { slot, .. } = hunt.outcome
        && from < end
        && end <= to
    {
        heard(call(slot, Cry::Distress, end));
    }
}
