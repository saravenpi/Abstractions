//! What one animal senses this frame: each threat it sees over the ground, hears or smells fills its
//! meter, and the one it is most aware of becomes its focus.

use bevy::math::{DVec3, Vec2, Vec3, Vec3Swizzles};

use super::cues::{clear, heard, smelt};
use super::meter::{Meter, fill_seconds, in_sight, stealth};
use super::stance::hunter;
use super::tune::{CROUCH_DROP, CROUCH_SIGHT, DRAIN, NOSE_GRAZER, NOSE_HUNTER, ROBOT_CHEST};
use crate::fauna::place::Spot;
use crate::life::Guild;

/// Something an animal can notice: a robot, or a hunter on the chase.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Threat {
    /// Planet position in metres.
    pub pos: DVec3,
    /// Speed in metres per second.
    pub speed: f32,
    /// Loudness, 1 for a robot at a walk.
    pub loud: f32,
    /// Mass in kilograms.
    pub mass: f32,
    /// Key of its meter.
    pub key: u64,
    /// Herd it belongs to, 0 for a robot: an animal does not fear its own herd.
    pub herd: u64,
    /// True for a robot.
    pub robot: bool,
    /// How far a robot is crouched, from 0 to 1.
    pub crouch: f32,
}

/// One animal as it senses: its frame, its place, its eyes, its range and its herd.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Senser {
    /// Its flat frame.
    pub spot: Spot,
    /// Its root in that frame.
    pub at: Vec3,
    /// Height of its eyes over its root in metres.
    pub eye: f32,
    /// Range factor of its senses: more for a lookout.
    pub keen: f32,
    /// Metres it smells from, downwind.
    pub nose: f32,
    /// Identity of its herd.
    pub herd: u64,
    /// True while it flees: a threat it already knows of is kept in mind without a sight test.
    pub fleeing: bool,
}

/// The air and ground an animal senses through this frame.
pub struct Air<'a> {
    /// Wind velocity in metres per second, in planet axes.
    pub wind: DVec3,
    /// Height of the ground over `(x, z)` in the animal's frame.
    pub ground: &'a dyn Fn(Vec2) -> f32,
    /// Seconds of this frame.
    pub dt: f32,
}

/// Returns the metres a species of `guild` smells from: hunters and scavengers farther.
pub fn nose(guild: Guild) -> f32 {
    if hunter(guild) || guild == Guild::Scavenger {
        NOSE_HUNTER
    } else {
        NOSE_GRAZER
    }
}

/// Returns true when `me` notices `threat` at `local` in its frame, `metres` away, with the wind at
/// `wind` `(x, z)` in its frame: by ear, by nose, or by eye over the ground. A `sure` threat, whose
/// meter is full or which a fleeing animal already knows, needs no sight test within range.
fn noticed(
    me: &Senser,
    (threat, local, metres): (&Threat, Vec3, f32),
    air: &Air,
    (wind, sure): (Vec2, bool),
) -> bool {
    if heard(threat.loud, metres, me.keen) || smelt(wind, (local - me.at).xz(), me.nose * me.keen) {
        return true;
    }
    let eye = me.at + Vec3::Y * me.eye;
    let chest = ROBOT_CHEST - CROUCH_DROP * threat.crouch;
    let range = me.keen * (1.0 - CROUCH_SIGHT * threat.crouch);
    in_sight(metres, range) && (sure || clear(air.ground, eye, local + Vec3::Y * chest))
}

/// Steps the `meter` of `me` by one frame against `threats`, and returns the index of the threat it
/// is most aware of and that awareness, when any is above zero.
pub fn perceive(
    meter: &mut Meter,
    me: &Senser,
    threats: &[Threat],
    air: &Air,
) -> Option<(usize, f32)> {
    meter.drain(air.dt);
    let wind = (me.spot.rot.inverse() * air.wind).as_vec3().xz();
    let mut best: Option<(usize, f32)> = None;
    for (index, threat) in threats
        .iter()
        .enumerate()
        .filter(|(_, t)| t.herd != me.herd)
    {
        let local = me.spot.local(threat.pos);
        let metres = local.distance(me.at);
        let held = meter.of(threat.key);
        let sure = held >= 1.0 - DRAIN * air.dt || (me.fleeing && held > 0.0);
        let aware = if noticed(me, (threat, local, metres), air, (wind, sure)) {
            let rate =
                stealth(threat.speed, threat.loud, threat.crouch) / fill_seconds(metres / me.keen);
            meter.raise(threat.key, (rate + DRAIN) * air.dt)
        } else {
            meter.of(threat.key)
        };
        if aware > 0.0 && best.is_none_or(|(_, a)| aware > a) {
            best = Some((index, aware));
        }
    }
    best
}
