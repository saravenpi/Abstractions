//! Tuning constants of the water a source churns: spray, bubbles, wakes and ripples.

/// Dynamic pressure in pascals of the flow over the sea above which it tears spray off the
/// surface: a wind of 8 m/s over home water, where spume starts to fly off the crests.
pub const SPRAY_PRESSURE: f32 = 38.0;

/// Dynamic pressure in pascals of the flow over the sea above which a hovering source rings the
/// surface with ripples: a wind of about 4 m/s.
pub const RIPPLE_PRESSURE: f32 = 10.0;

/// Share of the dynamic pressure of its own flight that a flat surface right under a source
/// feels: the air a moving body shoves aside.
pub const BOW_SHARE: f32 = 0.1;

/// Spray flecks thrown per metre a source travels at full strength.
pub const SPRAY_PER_METRE: f32 = 6.0;

/// Bubbles a full jet leaves per metre it travels.
pub const BUBBLES_PER_METRE: f32 = 5.0;

/// Bubbles a full jet leaves per second while it barely moves.
pub const BUBBLES_PER_SECOND: f32 = 12.0;

/// Metres a source travels between two segments of its wake.
pub const WAKE_STEP: f32 = 1.5;

/// Seconds a wake segment and a ripple ring last.
pub const FOAM_LIFE: f32 = 4.0;

/// Seconds between two ripple rings under a hovering source.
pub const RIPPLE_PERIOD: f32 = 0.45;

/// Width in metres of the band of one ripple ring.
pub const RIPPLE_BAND: f32 = 0.35;

/// Share of its speed at which the turbulent band along the track of a source widens, in metres
/// per second per metre per second.
pub const CENTRE_SPREAD: f32 = 0.03;

/// Half width in metres of the foam of an arm of a wake when it is laid.
pub const ARM_BAND: f32 = 0.25;

/// Share of the speed at which an arm moves out that its half width grows by.
pub const ARM_WIDEN: f32 = 0.15;

/// Share of the strength of the spray that the foam of an arm gets.
pub const ARM_SHARE: f32 = 0.8;

/// Metres between the last point of a wake and a new one past which the wake breaks off, as when
/// a source jumps.
pub const WAKE_BREAK: f32 = 40.0;

/// Height in metres of the foam above the still sea, so that it lies on the water.
pub const FOAM_LIFT: f32 = 0.06;

/// Depth in metres under the surface down to which a source skims it and leaves a wake.
pub const SKIM_DEPTH: f32 = 0.9;

/// Speeds in metres per second between which a source skimming the surface throws its spray.
pub const SKIM_SPRAY: (f32, f32) = (2.0, 9.0);

/// Share of the wash at the surface a spray fleck is thrown up at.
pub const SPRAY_KICK: f32 = 0.35;

/// Share of the velocity of the source a spray fleck carries.
pub const SPRAY_CARRY: f32 = 0.4;

/// Rate per second at which the air slows a spray fleck.
pub const SPRAY_DRAG: f32 = 1.5;

/// Speed in metres per second at which a bubble rises: a bubble of about 2 mm.
pub const BUBBLE_RISE: f32 = 0.25;

/// Seconds a spray fleck and a bubble live at most.
pub const DROP_LIFE: (f32, f32) = (1.2, 3.5);

/// Edge in metres of a spray fleck and of a bubble.
pub const DROP_SIZE: (f32, f32) = (0.09, 0.05);

/// Pooled spray flecks and bubbles.
pub const DROPS: usize = 256;

/// Pooled quads of foam: three per wake segment, the centre band and the two arms, and the pieces
/// of ripple rings.
pub const QUADS: usize = 1536;

/// Pieces of one ripple ring.
pub const RING_PIECES: usize = 16;

/// Most drops one source throws in one frame, whatever the distance it covered.
pub const MAX_THROWN: u32 = 24;

/// Spray flecks per second a full wash throws off the sea, on top of the ones per metre.
pub const SPRAY_PER_SECOND: f32 = 40.0;

/// Metres a source may move in one frame and still be followed: a longer step is a jump of the
/// world, such as a teleport or a change of planet, and starts its trace afresh.
pub const JUMP: f32 = 500.0;

/// Greatest opacity of the foam, from 0 to 1.
pub const FOAM_OPACITY: f32 = 0.85;
