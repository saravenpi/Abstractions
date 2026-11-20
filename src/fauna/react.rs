//! How an animal acts on its stance: it grazes, looks up at what it noticed, flees, stands its ground
//! or goes for a robot, and calms down once nothing is noticed. The stance comes from the senses
//! (`super::senses`). Reactions are local to each player: the herd path every player shares stays
//! the same, and a calm animal walks back to its place on it.

use bevy::math::Vec3;

use super::senses::Stance;
use super::tune::ALARM_RISE;
use crate::life::{Guild, Species};

/// What an animal does about the threats around it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    /// Nothing noticed: it follows its herd.
    Graze,
    /// Something noticed: it stops and looks at it.
    Alert,
    /// It runs away from it.
    Flee,
    /// It stands and faces it.
    Threaten,
    /// It walks up to it.
    Approach,
}

/// How a species calms down.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Temper {
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
    /// Last place of the threat it noticed, in the animal's frame.
    pub threat: Vec3,
    /// Its stance at the last step.
    pub stance: Stance,
}

/// Returns how wary a species of `guild` is, 0 bold to 1 shy: plant eaters most, hunters least.
fn wariness(guild: Guild) -> f32 {
    match guild {
        Guild::Grazer | Guild::Browser | Guild::Granivore | Guild::Nectar => 1.0,
        Guild::Frugivore | Guild::Filter => 0.8,
        Guild::Scavenger => 0.6,
        Guild::Hunter | Guild::Ambusher => 0.3,
        Guild::Decomposer | Guild::Parasite | Guild::Producer => 0.2,
    }
}

impl Temper {
    /// Returns the temper of `species`.
    pub fn of(species: &Species) -> Self {
        Self::new(species.role.guild)
    }

    /// Returns the temper of a species of `guild`: shy plant eaters stay alarmed longer than bold
    /// hunters.
    pub fn new(guild: Guild) -> Self {
        Self {
            calm: 6.0 + 10.0 * wariness(guild),
        }
    }
}

impl Mood {
    /// Moves the alarm `dt` seconds towards what `sensed` calls for: the place in the animal's frame
    /// of the threat it is most aware of and its stance, when it noticed one.
    pub fn feel(&mut self, temper: &Temper, sensed: Option<(Vec3, Stance)>, dt: f32) {
        let stance = sensed.map_or(Stance::Calm, |s| s.1);
        if let Some((at, _)) = sensed.filter(|s| s.1 != Stance::Calm) {
            self.threat = at;
        }
        let goal = match stance {
            Stance::Calm => 0.0,
            Stance::Flee => 1.0,
            _ => 0.6,
        };
        let rate = if goal > self.alarm {
            ALARM_RISE
        } else {
            1.0 / temper.calm
        };
        self.alarm += (goal - self.alarm) * (rate * dt).min(1.0);
        let running = !matches!(stance, Stance::Threaten | Stance::Attack);
        self.fleeing = running && (self.alarm > 0.8 || (self.fleeing && self.alarm > 0.4));
        self.stance = stance;
    }

    /// Returns what the animal does now.
    pub fn act(&self) -> Act {
        match (self.fleeing, self.stance) {
            (true, _) => Act::Flee,
            (false, Stance::Threaten) => Act::Threaten,
            (false, Stance::Attack) => Act::Approach,
            _ if self.alarm > 0.3 => Act::Alert,
            _ => Act::Graze,
        }
    }
}
