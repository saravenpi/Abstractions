//! Tuning of the sounds of blows.

/// Energy in joules at which a blow is as loud as it gets; loudness runs on the logarithm of the energy.
pub const JOULES_MAX: f32 = 2000.0;

/// Frequency in hertz that a struck body of one kilogram of contact stiffness 10 GPa rings at: the
/// ring follows the square root of the stiffness and falls with the cube root of the mass.
pub const RING_HZ: f32 = 1400.0;

/// Contact stiffness in gigapascals that sounds as dull as it gets and as bright as it gets, the
/// range the brightness runs over on a logarithmic scale: flesh and hide at the bottom, bone and
/// plate at the top.
pub const BRIGHT: [f32; 2] = [0.05, 20.0];

/// Lowest and highest ring in hertz.
pub const RING: [f32; 2] = [60.0, 4000.0];

/// Metres at which a blow is half as loud as beside the head.
pub const HEARD: f32 = 10.0;

/// Tip speed in metres per second at which a swing whooshes at its loudest.
pub const FAST: f32 = 15.0;

/// Centre of the whoosh of a light tool at full speed, in hertz.
pub const WHOOSH_HZ: f32 = 1100.0;

/// Moment of inertia about the hand in kg·m² at which a tool is half as heavy as it gets for the sound.
pub const HEAVY: f32 = 0.1;
