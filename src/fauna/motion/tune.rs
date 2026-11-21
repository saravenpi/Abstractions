//! Species tuning from mass: heavy bodies lumber on slow, well damped springs, small ones twitch.

use super::spring::Spring;
use crate::life::body::Body;

/// Mass of one cubic metre of body, in kilograms, times the share of its box an ellipsoid fills.
const DENSITY: f32 = 1000.0 * std::f32::consts::FRAC_PI_6;

/// Share of its rest distance a neck drawn back a full radian brings its tip nearer its root.
pub const DRAW_IN: f32 = 0.35;

/// Distance a neck thrown out all the way reaches for, in lengths of the neck: past its reach, so it
/// straightens.
pub const THROW: f32 = 1.2;

/// Drop of the way a neck is thrown out, in metres per metre ahead: a little down, toward the target.
pub const THROW_DIP: f32 = 0.25;

/// Share by which one breath may be shorter or longer than the resting rate, drawn at each breath.
pub const BREATH_JITTER: f32 = 0.12;

/// Breath rate at full effort over the resting rate: a body that just ran or fought pants.
pub const PANT: f32 = 2.5;

/// Seconds the effort of a body takes to rise to a run or an alarm.
pub const EFFORT_RISE: f32 = 1.5;

/// Seconds the effort of a body takes to fall back once it rests: its breath slows over them.
pub const RECOVER: f32 = 20.0;

/// Weight shifts per second of a standing body, each individual a little faster or slower.
pub const SHIFT_RATE: f32 = 0.13;

/// How a species moves, from its mass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tune {
    /// Mass in kilograms.
    pub mass: f32,
    /// Natural frequency of the root in hertz.
    pub f: f32,
    /// Damping of the root.
    pub zeta: f32,
}

impl Tune {
    /// Returns the tuning of a body of `mass` kilograms: 3 Hz and 0.55 at 1 kg, 0.9 Hz and 0.85 at a tonne.
    pub fn of_mass(mass: f32) -> Self {
        let mass = mass.max(1e-4);
        Self {
            mass,
            f: (3.0 * mass.powf(-0.17)).clamp(0.5, 8.0),
            zeta: (0.55 + 0.1 * mass.log10()).clamp(0.35, 0.95),
        }
    }

    /// Returns the tuning of `body`, its mass estimated from the volume of its parts.
    pub fn of_body(body: &Body) -> Self {
        let volume: f32 = body
            .parts
            .iter()
            .map(|p| p.size.x * p.size.y * p.size.z)
            .sum();
        let mass = match volume > 0.0 {
            true => DENSITY * volume,
            false => DENSITY * body.length.max(0.05).powi(3),
        };
        Self::of_mass(mass)
    }

    /// Returns the spring the root velocity follows its drive through.
    pub fn root(&self) -> Spring {
        Spring::new(self.f, self.zeta, 0.0)
    }

    /// Returns the spring of the heading: a little quicker than the root.
    pub fn heading(&self) -> Spring {
        Spring::new(1.3 * self.f, self.zeta.max(0.6), 0.0)
    }

    /// Returns the spring that holds the body over its feet: quick and nearly critical.
    pub fn body(&self) -> Spring {
        Spring::new(2.0 * self.f, self.zeta.max(0.8), 0.0)
    }

    /// Returns the breathing rate in hertz, which falls with mass as `mass^-0.26`.
    pub fn breath(&self) -> f32 {
        (0.75 * self.mass.powf(-0.26)).clamp(0.1, 3.0)
    }
}
