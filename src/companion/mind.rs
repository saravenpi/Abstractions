//! The idle mind: when the player stands still, which behaviour runs, dozing off and waking up.

use super::idle::{Idle, pick};
use super::pose::{Pose, doze_pose, idle_pose};
use super::rng::Rng;
use super::tune::{DOZE_AFTER, IDLE_AFTER, IDLE_REST, STILL_SPEED};

/// What the mind is told each frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Input {
    /// Speed of the player in meters per second.
    pub speed: f32,
    /// True at night.
    pub night: bool,
    /// True while the player is seated in the ship.
    pub seated: bool,
    /// Frame time in seconds.
    pub dt: f32,
}

/// Something that just happened in the mind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cue {
    /// An idle behaviour started.
    Started(Idle),
    /// It dozed off.
    Dozed,
    /// It woke up because the player moved.
    Woke,
}

/// What the companion does on its own while the player stands still.
#[derive(Debug, Clone)]
pub struct Mind {
    /// Seconds the player has stood still.
    pub still: f32,
    /// The idle in progress.
    pub idle: Option<Idle>,
    /// Seconds into the idle in progress, or into the doze.
    pub time: f32,
    /// True while it sleeps.
    pub dozing: bool,
    /// Seconds left before the next idle may start.
    rest: f32,
    /// The idle that ran last, never chosen twice in a row.
    last: Option<Idle>,
    /// Random numbers for the choices.
    rng: Rng,
}

impl Mind {
    /// Returns an awake mind whose choices follow `seed`.
    pub fn new(seed: u64) -> Self {
        Self {
            still: 0.0,
            idle: None,
            time: 0.0,
            dozing: false,
            rest: 0.0,
            last: None,
            rng: Rng::new(seed),
        }
    }

    /// Drops whatever it was doing because the player moved, and tells whether that woke it up.
    fn rouse(&mut self) -> Option<Cue> {
        let woke = self.dozing;
        self.last = self.idle.or(self.last);
        (self.still, self.idle, self.dozing, self.rest) = (0.0, None, false, 0.0);
        woke.then_some(Cue::Woke)
    }

    /// Runs the idle in progress, or starts the next one after a pause.
    fn play(&mut self, input: &Input) -> Option<Cue> {
        if let Some(idle) = self.idle {
            self.time += input.dt;
            if self.time >= idle.seconds() {
                (self.idle, self.last) = (None, Some(idle));
                self.rest = self.rng.range(IDLE_REST.0, IDLE_REST.1);
            }
            return None;
        }
        self.rest -= input.dt;
        if self.still < IDLE_AFTER || self.rest > 0.0 {
            return None;
        }
        let idle = pick(&mut self.rng, self.last, input.night);
        (self.idle, self.time) = (Some(idle), 0.0);
        Some(Cue::Started(idle))
    }

    /// Advances the mind by one frame and returns what just happened, if anything.
    ///
    /// A moving or seated player stops every idle. After `IDLE_AFTER` seconds of standing still the
    /// idles start, and after `DOZE_AFTER` seconds it dozes until the player moves.
    pub fn advance(&mut self, input: &Input) -> Option<Cue> {
        if input.seated || input.speed > STILL_SPEED {
            return self.rouse();
        }
        self.still += input.dt;
        if self.dozing {
            self.time += input.dt;
            return None;
        }
        if self.still >= DOZE_AFTER && self.idle.is_none() {
            (self.dozing, self.time) = (true, 0.0);
            return Some(Cue::Dozed);
        }
        self.play(input)
    }

    /// Returns how the body is posed by the idle in progress or the doze.
    pub fn pose(&self) -> Pose {
        match (self.dozing, self.idle) {
            (true, _) => doze_pose(self.time),
            (false, Some(idle)) => idle_pose(idle, self.time, idle.seconds()),
            (false, None) => Pose::default(),
        }
    }
}
