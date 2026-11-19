//! Tuning of the odds of life and of how fast it climbs.

/// Rate of abiogenesis on an ideal world, in events per billion years.
///
/// Anchored on the Earth: its surface was habitable by about 4.4 billion years ago and held life
/// by about 3.9 to 4.1, so on an ideal world the mean wait is about half a billion years. An Earth
/// twin 4.5 billion years old then lives with a chance of `1 - exp(-9)`. Lineweaver and Davis use
/// this quick start to argue that life is common on old wet rocky worlds:
/// https://arxiv.org/abs/astro-ph/0205014
pub const RATE: f32 = 2.0;

/// Billions of years evolution takes to climb one rung at the chemical speed of water at 288 K.
pub const RUNG_GYR: f32 = 2.2;

/// Oldest a star gets, in billions of years.
pub const OLDEST: f32 = 12.0;

/// Activation barrier of the chemistry of life over the gas constant, in kelvin.
///
/// Softened from the 5,000 K and more of organic reactions in water, which would stop cryogenic
/// chemistry outright: at 600 K, life in hydrocarbons at 94 K runs about 70 times slower than
/// water life at 288 K, and ammonia life at 220 K about half as fast.
pub const BARRIER: f32 = 600.0;
