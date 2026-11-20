//! How an animal reacts to robots and hunters: it grazes, looks up when one comes near, flees when it
//! comes nearer, and calms down once it is gone. Reactions are local to each player: the herd path
//! every player shares stays the same, and a calm animal walks back to its place on it.

use bevy::math::Vec3;

use super::tune::{ALARM_RISE, FLEE_SHARE, NOTICE, NOTICE_PER_METRE};
use crate::life::{Guild, Species};

/// What an animal does about the threats around it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    /// Nothing near: it follows its herd.
    Graze,
    /// Something near: it stops and looks at it.
    Alert,
    /// Something too near: it runs away from it.
    Flee,
}

/// How a species reacts: the distances it notices and flees from, and how slowly it calms down.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Temper {
    /// Metres from which a threat is noticed.
    pub notice: f32,
    /// Metres under which a threat makes it flee.
    pub flee: f32,
    /// Seconds the alarm takes to fall away once the threat is gone.
    pub calm: f32,
}

/// The alarm of one animal.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Mood {
    /// Alarm from 0 calm to 1 fleeing.
    pub alarm: f32,
    /// True while it flees; it stops once the alarm has fallen back.
    pub fleeing: bool,
    /// Last place of the nearest threat in the animal's frame.
    pub threat: Vec3,
}

/// Returns how wary a species of `guild` is, 0 bold to 1 shy: plant eaters most, hunters least.
fn wariness(guild: Guild) -> f32 {
    match guild {
        Guild::Grazer | Guild::Browser | Guild::Granivore | Guild::Nectar => 0.9,
        Guild::Frugivore | Guild::Filter => 0.75,
        Guild::Scavenger => 0.6,
        Guild::Hunter | Guild::Ambusher => 0.3,
        Guild::Decomposer | Guild::Parasite | Guild::Producer => 0.15,
    }
}

impl Temper {
    /// Returns the temper of `species`: shy plant eaters keep farther than bold hunters, big animals
    /// farther than small ones, and the top of the food web fears least.
    pub fn of(species: &Species) -> Self {
        let top = (1.5 - 0.2 * species.role.level).clamp(0.3, 1.0);
        let wary = wariness(species.role.guild) * top;
        let notice = (NOTICE + NOTICE_PER_METRE * species.body.length) * (0.5 + wary);
        Self {
            notice,
            flee: notice * FLEE_SHARE * wary,
            calm: 6.0 + 10.0 * wary,
        }
    }
}

impl Mood {
    /// Moves the alarm `dt` seconds towards what the nearest threat calls for: `near` is its place in
    /// the animal's frame and its distance, when there is one.
    pub fn feel(&mut self, temper: &Temper, near: Option<(Vec3, f32)>, dt: f32) {
        let goal = match near {
            Some((_, d)) if d < temper.flee => 1.0,
            Some((_, d)) if d < temper.notice => 0.6,
            _ => 0.0,
        };
        if let Some((at, _)) = near.filter(|n| n.1 < temper.notice) {
            self.threat = at;
        }
        let rate = if goal > self.alarm {
            ALARM_RISE
        } else {
            1.0 / temper.calm
        };
        self.alarm += (goal - self.alarm) * (rate * dt).min(1.0);
        self.fleeing = self.alarm > 0.8 || (self.fleeing && self.alarm > 0.4);
    }

    /// Returns what the animal does now.
    pub fn act(&self) -> Act {
        match (self.fleeing, self.alarm > 0.3) {
            (true, _) => Act::Flee,
            (false, true) => Act::Alert,
            (false, false) => Act::Graze,
        }
    }
}
