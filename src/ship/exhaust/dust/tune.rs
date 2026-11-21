//! Tuning constants of the dust the jets of a ship raise. Lengths are in metres, times in
//! seconds, speeds in metres per second.

use crate::avatar::Soil;

/// Puffs in the pool, made once: the most dust in the air at a time, from every ship.
pub const COUNT: usize = 64;

/// Metres along its axis within which one lift nozzle at its full push of 92 kN still lifts dust
/// off the ground. Any jet reaches as far as the root of its thrust allows: the dynamic pressure
/// of a round jet falls as its thrust over the square of the distance.
pub const REACH: f32 = 30.0;

/// Puffs per second a jet raises where it meets the ground right under its nozzle. It falls off
/// to none at the end of its reach.
pub const RATE: f32 = 8.0;

/// Puffs a jet may owe at once, so that a long frame never empties the pool.
pub const OWED: f32 = 6.0;

/// Speed at which the flow spreading out along the ground flings a puff from right under the
/// nozzle. It falls off to none at the end of the reach of the jet.
pub const FLING: f32 = 12.0;

/// Share of its outward speed with which the turbulence of the jet lifts a puff off the ground.
pub const RISE: f32 = 0.25;

/// Seconds over which the air that carries a puff slows from the flow of the jet to the wind:
/// the time the flow along the ground takes to spread out and die.
pub const DRAG_TIME: f32 = 0.8;

/// Size of a puff when it is thrown, and how much it widens per root second: turbulent mixing
/// spreads a cloud with the root of time.
pub const SIZE: (f32, f32) = (1.2, 1.3);

/// Shortest and longest seconds a puff lasts.
pub const LIFE: (f32, f32) = (2.5, 4.5);

/// Least air, as a share of the air at sea level at home, that holds a cloud of dust up. In
/// thinner air the dust flies off as grit and is not drawn.
pub const THIN: f32 = 0.05;

/// Least cosine between a jet and the way down for it to meet the ground: a jet that only
/// grazes it blows along it and raises nothing here.
pub const GRAZE: f64 = 0.2;

/// Metres a puff drifts before the height of the ground under it is read again.
pub const REREAD: f64 = 3.0;

/// Speed at which a puff of each soil settles through still air under standard gravity: the
/// Stokes speed of its grains, fine silt and clay for earth, fine sand, the dust of bare ground,
/// and powder snow. It grows with gravity and does not depend on how thick the air is.
pub const SETTLE: [(Soil, f32); 4] = [
    (Soil::Earth, 0.15),
    (Soil::Land, 0.25),
    (Soil::Sand, 0.4),
    (Soil::Snow, 0.5),
];
