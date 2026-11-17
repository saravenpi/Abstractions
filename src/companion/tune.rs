//! Every number of the companion the owner may want to tune, in one place.

/// Half extents of the rounded head-body in meters: about 45 cm wide with its hands.
pub const HALF: [f32; 3] = [0.17, 0.16, 0.145];

/// How far to the right of the eyes the companion rests, in meters.
pub const SIDE: f32 = 0.85;

/// How far ahead of the eyes the companion rests, in meters.
pub const AHEAD: f32 = 1.35;

/// Height of the resting spot relative to the eyes, in meters.
pub const RISE: f32 = -0.12;

/// How far behind the eyes it reappears after being left behind, in meters.
pub const BEHIND: f32 = 2.0;

/// Gap kept between the resting spot and the ground or the water, in meters.
pub const CLEARANCE: f32 = 0.55;

/// Gap kept while it inspects something on the ground, in meters.
pub const LOW_CLEARANCE: f32 = 0.32;

/// Gap the body itself never goes under, whatever the spring does, in meters.
pub const FLOOR: f32 = 0.24;

/// Spring rate in radians per second while it is close to its spot. Higher is tighter.
pub const FOLLOW_RATE: f32 = 4.0;

/// Spring rate once it has fallen far behind.
pub const CATCH_RATE: f32 = 12.0;

/// Distances from its spot between which the spring stiffens from one rate to the other, in meters.
pub const CATCH_SPAN: (f32, f32) = (1.0, 5.0);

/// Share of the player's velocity it is carried with, from 0 (pure spring) to 1 (rigid).
pub const FEED: f32 = 0.5;

/// Distance from its spot beyond which it quietly reappears behind the player, in meters.
pub const LEASH: f32 = 60.0;

/// Where it rides in the cockpit, in camera space: right, up and back (negative is ahead), in meters.
pub const SEAT: [f32; 3] = [0.52, -0.2, -0.95];

/// Spring rate of the ride in the cockpit.
pub const SEAT_RATE: f32 = 7.0;

/// Farthest it may be from the camera when the player sits down, in meters, before it is pulled in.
pub const BOARD_REACH: f32 = 3.0;

/// How fast the screen turns toward what it looks at, per second.
pub const TURN_RATE: f32 = 5.0;

/// Height of the hover bob in meters and its rate in radians per second.
pub const BOB: (f32, f32) = (0.035, 2.1);

/// Lean in radians per meter per second of speed, and the largest lean in radians.
pub const TILT: (f32, f32) = (0.07, 0.38);

/// How far a hand trails behind per meter per second, and the farthest it trails, in meters.
pub const HAND_LAG: (f32, f32) = (0.035, 0.11);

/// Luminous power of the screen light in lumens at full glow. A village torch gives 26 000.
pub const LIGHT_POWER: f32 = 2600.0;

/// Reach of the screen light in meters.
pub const LIGHT_RANGE: f32 = 5.5;

/// Share of the light power kept in full daylight.
pub const LIGHT_DAY_SHARE: f32 = 0.3;

/// Brightness of the lit screen pixels, as a multiple of white, so bloom makes the face glow.
pub const SCREEN_HDR: f32 = 3.2;

/// Brightness of the antenna tip and the thruster, as a multiple of white.
pub const GLOW_HDR: f32 = 2.4;

/// Player speed in meters per second under which the player counts as standing still.
pub const STILL_SPEED: f32 = 0.4;

/// Seconds the player stands still before the first idle behaviour.
pub const IDLE_AFTER: f32 = 4.0;

/// Shortest and longest pause between two idle behaviours, in seconds.
pub const IDLE_REST: (f32, f32) = (2.0, 5.5);

/// Seconds the player stands still before the companion dozes off.
pub const DOZE_AFTER: f32 = 50.0;

/// Seconds of a steady view before the companion looks at what the player stares at.
pub const STARE_AFTER: f32 = 1.6;

/// Camera turn rate in radians per second under which the view counts as steady.
pub const STARE_TURN: f32 = 0.35;

/// Distance in meters of the point the player is taken to stare at.
pub const STARE_REACH: f32 = 7.0;

/// Chance that a fresh glance goes to the player rather than ahead.
pub const PLAYER_SHARE: f32 = 0.6;

/// Night level, from 0 (day) to 1 (night), above which the companion is sleepy.
pub const SLEEPY_NIGHT: f32 = 0.75;

/// Depth of the eyes under water in meters beyond which the companion worries.
pub const WORRY_DEPTH: f32 = 0.25;

/// Climb rate of the ship in meters per second beyond which the companion is wide-eyed.
pub const TAKEOFF_CLIMB: f32 = 6.0;

/// Distance in meters within which a structure makes the companion curious, and it may go and inspect it.
pub const CURIOUS_REACH: f32 = 14.0;

/// Seconds the delighted face stays after a discovery.
pub const DELIGHT_SECONDS: f32 = 6.0;

/// Seconds the wide-eyed face stays after the ship leaves the ground.
pub const TAKEOFF_SECONDS: f32 = 2.5;

/// Seconds the surprised face stays after waking up.
pub const WAKE_SECONDS: f32 = 1.4;
