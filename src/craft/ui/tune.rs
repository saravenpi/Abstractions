//! Tuning constants of the workshop: the full scale of each bar a tool card draws. A number at its
//! full scale fills its bar; one past it fills it and no more.

/// Reach that fills the reach bar, in metres: the longest shaft the solver tries, with its grip.
pub const FULL_REACH: f32 = 1.6;

/// Mass that fills the mass bar, in kilograms.
pub const FULL_MASS: f32 = 3.0;

/// Depth that fills the edge, point or impact bar, in layers of the reference target: it has two.
pub const FULL_DEPTH: f32 = 2.0;

/// Speed of the head that fills the swing bar, in metres per second.
pub const FULL_SPEED: f32 = 20.0;

/// Scratch hardness that fills the hardness bar, in Mohs.
pub const FULL_HARDNESS: f32 = 8.0;

/// Fracture toughness that fills the toughness bar, in MPa*m^0.5.
pub const FULL_TOUGHNESS: f32 = 6.5;

/// Tensile or bending strength that fills the strength bar, in MPa.
pub const FULL_STRENGTH: f32 = 380.0;

/// Young's modulus that fills the stiffness bar, in GPa.
pub const FULL_STIFFNESS: f32 = 200.0;
