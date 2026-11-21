//! The mark of a blow for the crosshair: what the last blow of the local robot came to, how long
//! ago, and the animal it is fighting. A swing that met nothing leaves no mark: its whoosh says it.

use bevy::prelude::*;

use super::super::hit::Hit;
use super::super::input::Robot;
use super::super::outcome::Outcome;
use super::tune::{FIGHT, MARK_LIFE};

/// The last blow of the local robot, as the crosshair shows it.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct Mark {
    /// What the last blow came to: clean, glanced off or stopped; nothing before the first.
    pub outcome: Option<Outcome>,
    /// Seconds since that blow.
    pub age: f32,
    /// The animal it struck, while the fight shows.
    pub victim: Option<Entity>,
    /// Seconds the state of that animal still shows.
    pub fight: f32,
}

impl Mark {
    /// Returns what the hit mark at the crosshair shows now: the outcome of a blow younger than
    /// [`MARK_LIFE`] and how far it faded, 1 fresh to 0 gone.
    pub fn shown(&self) -> Option<(Outcome, f32)> {
        let left = 1.0 - self.age / MARK_LIFE;
        self.outcome.filter(|_| left > 0.0).map(|o| (o, left))
    }
}

/// Run condition: a hit mark or the state of a foe still shows.
pub fn marking(mark: Res<Mark>) -> bool {
    mark.fight > 0.0 || mark.age < MARK_LIFE
}

/// Marks every blow the local robot landed: its outcome, fresh, and the fight with its animal.
pub fn note(mut hits: MessageReader<Hit>, robot: Robot, mut mark: ResMut<Mark>) {
    let body = robot.body();
    for hit in hits.read().filter(|hit| Some(hit.attacker) == body) {
        *mark = Mark {
            outcome: Some(hit.outcome),
            age: 0.0,
            victim: Some(hit.victim),
            fight: FIGHT,
        };
    }
}

/// Ages the mark and lets the fight go once it has shown long enough.
pub fn age(time: Res<Time>, mut mark: ResMut<Mark>) {
    let dt = time.delta_secs();
    mark.age += dt;
    mark.fight = (mark.fight - dt).max(0.0);
    if mark.fight <= 0.0 {
        mark.victim = None;
    }
}
