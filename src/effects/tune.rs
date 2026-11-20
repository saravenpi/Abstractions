//! The numbers behind what the plants shed: how hard leaves hang on, how fast a gust or a shower tears
//! them off and how fast they sink.

/// Density of the air of the home planet in kilograms per cubic metre, at a thickness of 1.
pub const DENSITY: f32 = 1.2;

/// Dynamic pressure in pascals under which a leaf in full leaf holds on: a wind of about 3.6 m/s in
/// home air. Drag on a leaf grows with density times the square of the wind speed.
pub const HOLD_LEAF: f32 = 8.0;

/// Dynamic pressure in pascals under which a petal holds on: lighter and looser than a leaf.
pub const HOLD_PETAL: f32 = 3.0;

/// Leaves per second per crown a wind at twice the holding pressure tears off a plant whose leaves
/// hang fully loose. The rate grows with the square of the excess pressure.
pub const WIND_GAIN: f32 = 0.4;

/// Leaves per second per crown that full rain knocks off a plant whose leaves hang fully loose.
pub const RAIN_GAIN: f32 = 0.2;

/// How much more a plant sheds at full water stress than at none: the factor is `1 + STRESS_GAIN * stress`.
pub const STRESS_GAIN: f32 = 2.0;

/// Most leaves per second one crown sheds.
pub const MOST: f32 = 3.0;

/// Petals per second per crown in calm air.
pub const PETAL_CALM: f32 = 0.08;

/// Petals per second per crown a wind at twice the holding pressure tears off.
pub const PETAL_WIND: f32 = 0.6;

/// Petals per second per crown that full rain knocks off.
pub const PETAL_RAIN: f32 = 0.3;

/// Pollen specks per second per crown on a dry day with the wind up.
pub const POLLEN: f32 = 0.5;

/// Wind speeds in metres per second between which the pollen goes from still to lifted.
pub const POLLEN_WIND: (f32, f32) = (1.0, 3.0);

/// Rain from which no pollen lifts, from 0 to 1.
pub const POLLEN_RAIN: f32 = 0.1;

/// How a plant holds its leaves in one season.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grip {
    /// Leaves per second per crown that fall in still air.
    pub calm: f32,
    /// How loose the leaves hang, from 0 (nothing to tear off) to 1 (about to fall).
    pub loose: f32,
}

/// An evergreen, in every season: a slow trickle and a firm hold.
pub const EVERGREEN: Grip = Grip {
    calm: 0.01,
    loose: 0.25,
};

/// A deciduous plant on a planet without seasons: it renews its leaves all year.
pub const RENEWING: Grip = Grip {
    calm: 0.01,
    loose: 0.4,
};

/// A deciduous plant in spring: new leaves, firmly attached.
pub const SPRING: Grip = Grip {
    calm: 0.005,
    loose: 0.2,
};

/// A deciduous plant in summer: in full leaf and healthy.
pub const SUMMER: Grip = Grip {
    calm: 0.005,
    loose: 0.25,
};

/// A deciduous plant in autumn: its leaves let go even in still air, and a breeze takes them.
pub const AUTUMN: Grip = Grip {
    calm: 0.5,
    loose: 1.0,
};

/// A deciduous plant in winter: bare, nothing left to shed.
pub const WINTER: Grip = Grip {
    calm: 0.0,
    loose: 0.0,
};

/// Mass of a leaf of edge 0.1 m per square metre of its area, in kilograms per square metre.
pub const AREAL_MASS: f32 = 0.1;

/// Drag coefficient of a leaf tumbling flat to the air.
pub const DRAG: f32 = 2.0;

/// Thinnest air in thickness units the sinking speed is worked out for, and fastest sink in metres per second.
pub const SINK: (f32, f32) = (0.02, 20.0);

/// Respawns per second of a pool that is mostly parked: its size over the two seconds a parked
/// particle waits. A share of the respawns is the rate of that kind over this.
pub const RESPAWNS: f32 = super::pool::POOL_SIZE as f32 / 2.0;
