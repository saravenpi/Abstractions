//! Fight or flight: what an animal does about the threat it is most aware of.
//!
//! An animal fights when it outweighs the threat enough, `own mass × health / threat mass` above
//! [`FIGHT`], when it is cornered, or when it guards young; plant eaters flee anything over
//! [`PREY_SHARE`] of their size; hungry hunters go for a robot they outweigh, never in the
//! [`super::tune::GRACE`] after a landing. One that would stand its ground goes for a robot that hurt
//! it instead, for the grudge of its temper (`crate::fauna::react::Temper::grudge`).

use super::tune::{ALARM, FIGHT, PREY_SHARE, WARY};
use crate::life::Guild;

/// What an animal does about the threats it noticed. Other systems read it: the walk, the calls and,
/// later, the attacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Stance {
    /// It noticed nothing.
    #[default]
    Calm,
    /// It noticed something and watches it.
    Wary,
    /// It runs away.
    Flee,
    /// It stands its ground and faces the threat.
    Threaten,
    /// It goes for the threat.
    Attack,
}

/// What an animal weighs about itself.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Me {
    /// Mass in kilograms.
    pub mass: f32,
    /// Share of its health left, 0 to 1.
    pub health: f32,
    /// What it eats.
    pub guild: Guild,
    /// True when its herd is not feeding or resting.
    pub hungry: bool,
    /// True when the ground blocks its way out.
    pub cornered: bool,
    /// True for an adult of a species that tends young, in a herd that has some.
    pub guarding: bool,
    /// True in the grace after a landing.
    pub grace: bool,
    /// True while it goes for the robot that hurt it.
    pub provoked: bool,
}

/// What an animal weighs about the threat.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Foe {
    /// Mass in kilograms.
    pub mass: f32,
    /// True for a robot.
    pub robot: bool,
}

/// Returns true for the guilds that eat plants.
pub fn plant_eater(guild: Guild) -> bool {
    matches!(
        guild,
        Guild::Grazer | Guild::Browser | Guild::Granivore | Guild::Nectar | Guild::Frugivore
    )
}

/// Returns true for the guilds that hunt.
pub fn hunter(guild: Guild) -> bool {
    matches!(guild, Guild::Hunter | Guild::Ambusher)
}

/// Returns true when `me` goes for `foe`: a hungry hunter that outweighs a robot, out of the grace.
fn hunts(me: &Me, foe: &Foe) -> bool {
    hunter(me.guild) && foe.robot && me.hungry && !me.grace && me.mass > foe.mass
}

/// Returns the stance of `me` towards `foe` at awareness `aware`.
pub fn choose(aware: f32, me: &Me, foe: &Foe) -> Stance {
    if aware < WARY {
        Stance::Calm
    } else if hunts(me, foe) {
        Stance::Attack
    } else if aware < ALARM {
        Stance::Wary
    } else {
        match fight_or_flight(me, foe) {
            Stance::Threaten if me.provoked && foe.robot => Stance::Attack,
            stance => stance,
        }
    }
}

/// Returns whether `me`, fully alarmed by `foe`, threatens it or flees.
fn fight_or_flight(me: &Me, foe: &Foe) -> Stance {
    if me.cornered {
        Stance::Threaten
    } else if plant_eater(me.guild) && foe.mass > PREY_SHARE * me.mass {
        Stance::Flee
    } else if me.guarding || me.mass * me.health / foe.mass.max(1e-3) > FIGHT {
        Stance::Threaten
    } else {
        Stance::Flee
    }
}
