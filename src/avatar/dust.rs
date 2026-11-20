//! What the body kicks up, as pure rules: which real event throws which puffs, in what the
//! ground under the feet is made of.
//!
//! Nothing here plays by itself. A landing throws a ring sized by the speed the body really
//! lost, a running foot a puff when it really plants, a hard stop a skid, a jump a kick back,
//! and the wings a breath of the air they really catch as they spread.

use bevy::prelude::*;

use super::clock::ramp;
use super::fill::RUN_ABOVE;
use super::{AvatarPose, Mode};
use crate::universe::PlanetClass;
use crate::world::Surface;

/// Loss of ground speed in meters per second squared beyond which the feet skid.
pub const SKID: f32 = 14.0;

/// Airspeeds in meters per second between which the spreading wings catch enough air to show.
pub const CATCH: (f32, f32) = (8.0, 30.0);

/// What a puff is made of. The order is the order of the materials of the pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Soil {
    /// Pale dry sand.
    #[default]
    Sand,
    /// Brown earth under grass.
    Earth,
    /// Grey rock dust.
    Rock,
    /// Powder snow.
    Snow,
    /// The bare ground of a world without life, in the colour of its class.
    Land,
    /// Drops of the liquid of the planet.
    Splash,
    /// Air made visible for a moment.
    Air,
}

impl Soil {
    /// Every soil, in declaration order.
    pub const ALL: [Soil; 7] = [
        Soil::Sand,
        Soil::Earth,
        Soil::Rock,
        Soil::Snow,
        Soil::Land,
        Soil::Splash,
        Soil::Air,
    ];

    /// Returns the sRGB colour and the alpha of a puff of this soil on a planet of `class`.
    pub fn tint(self, class: PlanetClass) -> [f32; 4] {
        let look = class.look();
        let of = |c: Vec3, alpha: f32| [c.x, c.y, c.z, alpha];
        let watery = matches!(class, PlanetClass::Ocean | PlanetClass::Island);
        match self {
            Soil::Sand => [0.86, 0.78, 0.58, 0.55],
            Soil::Earth => [0.46, 0.37, 0.25, 0.55],
            Soil::Rock => [0.56, 0.55, 0.53, 0.5],
            Soil::Snow => [0.95, 0.97, 1.0, 0.7],
            Soil::Land => of(look.land, 0.55),
            Soil::Splash if watery => [0.9, 0.95, 1.0, 0.5],
            Soil::Splash => of(look.sea, 0.6),
            Soil::Air => [1.0, 1.0, 1.0, 0.16],
        }
    }
}

/// Returns what the feet raise from `surface` on a planet of `class`: a splash from any
/// liquid, snow from snow, and from dry ground the soil of a living world or the bare ground
/// of the class.
pub fn soil(class: PlanetClass, surface: Surface) -> Soil {
    let living = matches!(class, PlanetClass::Ocean | PlanetClass::Island);
    match surface {
        Surface::Water | Surface::Shallows | Surface::Reef | Surface::Pond | Surface::River => {
            Soil::Splash
        }
        Surface::Snow => Soil::Snow,
        Surface::Rock if living => Soil::Rock,
        Surface::Sand if living => Soil::Sand,
        Surface::Grass if living => Soil::Earth,
        _ => Soil::Land,
    }
}

/// Where on the body a burst leaves from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum From {
    /// The ground between the two soles.
    Feet,
    /// One sole: 0 the left, 1 the right.
    Foot(usize),
    /// Both wingtips, in turn.
    Tips,
}

/// Which way a burst is thrown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Way {
    /// Out in a ring along the ground.
    Ring,
    /// Back against the way the body goes.
    Back,
    /// Ahead, the way the body goes.
    Ahead,
    /// Left behind in the air the body flies through.
    Wake,
}

/// Puffs to throw at once.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Burst {
    /// How many puffs.
    pub count: f32,
    /// Where they leave from.
    pub from: From,
    /// Which way they go.
    pub way: Way,
    /// Their speed in meters per second.
    pub speed: f32,
    /// Their largest size in meters.
    pub size: f32,
    /// Seconds they last.
    pub life: f32,
}

/// What the body was doing on the last frame, as far as the puffs care.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Seen {
    /// Whether the knees were taking a landing in, or the landing mode was on.
    pub landing: bool,
    /// Whether it stood on its feet.
    pub standing: bool,
    /// Whether it was in the air.
    pub airborne: bool,
    /// Whether each foot carried the body.
    pub planted: [bool; 2],
    /// How far the wings were spread, from 0 to 1.
    pub wings: f32,
    /// Speed along the local up, in meters per second.
    pub vertical: f32,
}

impl Seen {
    /// Returns what the body does in the state `input`, with the feet `planted` or not.
    pub fn of(input: &AvatarPose, planted: [bool; 2]) -> Self {
        let standing = matches!(input.mode, Mode::Idle | Mode::Walk | Mode::Run | Mode::Land)
            || input.mode.floored();
        Self {
            landing: input.absorb || input.mode == Mode::Land,
            standing,
            airborne: !standing && !input.mode.wet() && input.mode != Mode::Seated,
            planted,
            wings: input.wings,
            vertical: input.vertical_speed,
        }
    }
}

/// Returns the ring a touch-down at `impact` meters per second throws: more, faster, larger
/// and longer-lived puffs the harder it is, and next to nothing for a step down.
pub fn landing(impact: f32) -> Burst {
    let hard = ramp(1.5, 18.0, impact);
    Burst {
        count: (18.0 * ramp(1.0, 14.0, impact)).round(),
        from: From::Feet,
        way: Way::Ring,
        speed: 1.0 + 4.5 * hard,
        size: 0.14 + 0.36 * hard,
        life: 0.5 + 0.5 * hard,
    }
}

/// Returns the bursts of one frame of `dt` seconds: what `now` does that `was` did not, in
/// the state `input`. Everything comes from a real event: a touch-down or a plunge, a foot
/// that plants at a run, a hard stop, a jump leaving the ground, wings that spread in moving air.
pub fn bursts(was: &Seen, now: &Seen, input: &AvatarPose, dt: f32) -> [Option<Burst>; 5] {
    let puff = |count: f32, from: From, way: Way, (speed, size, life): (f32, f32, f32)| Burst {
        count,
        from,
        way,
        speed,
        size,
        life,
    };
    let plunge = was.airborne && input.mode.wet();
    let touched = (now.landing && !was.landing) || plunge;
    let impact = if plunge { -was.vertical } else { input.impact };
    let running = input.mode == Mode::Run && input.speed > RUN_ABOVE;
    let stepped = |side: usize| running && now.planted[side] && !was.planted[side];
    let skid = ramp(SKID, 2.0 * SKID, -input.gain) * ramp(1.0, 3.0, input.speed);
    let caught = (now.wings - was.wings).max(0.0) * ramp(CATCH.0, CATCH.1, input.airspeed);
    let step = |side: usize| puff(1.0, From::Foot(side), Way::Back, (0.8, 0.11, 0.35));
    [
        touched.then(|| landing(impact.max(0.0))),
        (0..2).find(|side| stepped(*side)).map(step),
        (now.standing && skid > 0.0)
            .then(|| puff(40.0 * skid * dt, From::Feet, Way::Ahead, (2.0, 0.16, 0.45))),
        (was.standing && input.mode == Mode::Jump)
            .then(|| puff(5.0, From::Feet, Way::Back, (1.6, 0.13, 0.4))),
        (caught > 0.0).then(|| puff(14.0 * caught, From::Tips, Way::Wake, (1.2, 0.3, 0.4))),
    ]
}
