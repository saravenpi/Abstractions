//! Tuning of the signs of life: how long they last, how many there are and how big they are.
//!
//! Sizes follow body mass: a foot, a dropping or a burrow mouth grows as the cube root of the mass,
//! as lengths do in animals of one build (Schmidt-Nielsen 1984, "Scaling: why is animal size so
//! important?", Cambridge University Press).

/// Seconds a footprint lasts.
pub const TRAIL: f64 = 240.0;

/// Seconds between two reads of a herd along its trail; prints between two reads are interpolated.
pub const SAMPLE: f64 = 2.0;

/// Members of a herd whose prints are laid: the rest walk in their steps.
pub const TRACKERS: u8 = 4;

/// Slowest speed in metres per second of a herd centre that leaves prints.
pub const MOVING: f32 = 0.15;

/// Length of one step in leg lengths at a walk.
pub const STEP_PER_LEG: f32 = 0.9;

/// Length of a footprint in metres per cube root of a kilogram of the animal.
pub const FOOT_PER_KG: f32 = 0.035;

/// Seconds droppings and grazed patches last.
pub const LINGER: f64 = 1200.0;

/// Seconds between two reads of a herd for droppings and grazed patches.
pub const LINGER_STEP: f64 = 30.0;

/// Size of a dropping in metres per cube root of a kilogram of the animal.
pub const DUNG_PER_KG: f32 = 0.03;

/// Chance per member that a herd standing at one read leaves a dropping.
pub const DUNG_CHANCE: f64 = 0.04;

/// Least parental care, 0 to 1, of a species that keeps a nest or a burrow for its young.
pub const NEST_CARE: f32 = 0.35;

/// Heaviest animal in kilograms of the ground that digs a burrow; heavier ones make a nest.
pub const BURROW_KG: f32 = 25.0;

/// Most burrow mouths of one herd.
pub const BURROWS: u8 = 5;

/// Metres past the search radius a herd can stand and still have left signs inside it, for
/// [`TRAIL`] seconds at its walking speed.
pub const REACH: f64 = 250.0;

/// Most signs returned by one search.
pub const MOST: usize = 2048;
