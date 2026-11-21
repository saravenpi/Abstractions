//! Every number of the companion the owner may want to tune, in one place.

/// Longest frame time a companion simulates, in seconds, so a hitch never throws it around.
pub const MAX_STEP: f32 = 0.05;

/// Half extents of the rounded head-body in meters: about 45 cm wide with its hands.
pub const HALF: [f32; 3] = [0.17, 0.16, 0.145];

/// How far to the right of the eyes the companion reappears, in meters.
pub const SIDE: f32 = 1.5;

/// How far from the eyes the companion rests, in meters.
pub const REACH: f32 = 2.3;

/// Smallest and largest angle between the way the player faces and a place the companion
/// chooses, in radians: off to one side, never straight ahead.
pub const PERCH_ARC: (f32, f32) = (0.6, 1.2);

/// Seconds the companion stays out of place before it moves: behind a walking player or in its
/// way, and behind the back of a player who stands.
pub const PERCH_PATIENCE: (f32, f32) = (0.9, 3.5);

/// Shortest and longest time the companion keeps one place before it picks another by itself, in seconds.
pub const PERCH_REST: (f32, f32) = (10.0, 26.0);

/// Height of the resting spot relative to the eyes, in meters: below the line of sight.
pub const RISE: f32 = -0.3;

/// Tangent of the half angle of the cone around the line of sight it keeps out of, so it never
/// floats in front of what the player looks at: about 24 degrees.
pub const YIELD: f32 = 0.45;

/// How far behind the eyes it reappears after being left behind, in meters.
pub const BEHIND: f32 = 2.0;

/// Gap kept between the resting spot and the ground or the water, in meters.
pub const CLEARANCE: f32 = 0.55;

/// Gap kept while it inspects something on the ground, in meters.
pub const LOW_CLEARANCE: f32 = 0.32;

/// Gap the body itself never goes under, whatever the spring does, in meters.
pub const FLOOR: f32 = 0.24;

/// Radius of the ball the companion is to the drawn solids, in meters: its shell and its hands,
/// just under [`FLOOR`] so the floor alone never presses it into the ground.
pub const SHELL_RADIUS: f32 = 0.22;

/// Spring rate in radians per second while it is close to its spot. Higher is tighter.
pub const FOLLOW_RATE: f32 = 4.0;

/// Spring rate once it has fallen far behind.
pub const CATCH_RATE: f32 = 12.0;

/// Distances from its spot between which the spring stiffens from one rate to the other, in meters.
pub const CATCH_SPAN: (f32, f32) = (1.0, 5.0);

/// Share of the player's velocity it is carried with, from 0 (pure spring) to 1 (rigid).
pub const FEED: f32 = 0.5;

/// Distance from its spot beyond which it was truly left behind, by a fast travel, and starts
/// again out of view, in meters. Following never takes it that far.
pub const LEASH: f32 = 60.0;

/// Where it starts again once left behind: how far to the side of the camera and how far
/// behind it, in meters. That is outside any view narrower than 140 degrees, and close enough
/// that it flies into the picture within a second.
pub const ENTRY: (f32, f32) = (7.0, 2.5);

/// Where it rides in the cockpit, in the space of the pilot seat: right, up and back (negative
/// is ahead), in meters. It hovers over the high front of the left console, which carries no
/// lever, inside the wall of the tub, ahead of the hand on the throttle: beside the displays
/// and under the forward view, where the pilot sees it by looking down to the left.
pub const SEAT: [f32; 3] = [-0.3, -0.545, -0.42];

/// Its size in the cockpit, as a share of its size outside, so it stays a small part of the view.
pub const SEAT_SCALE: f32 = 0.33;

/// How fast it grows or shrinks between its cockpit size and its own, per second.
pub const SCALE_RATE: f32 = 6.0;

/// Spring rate of the ride in the cockpit.
pub const SEAT_RATE: f32 = 7.0;

/// Farthest it may be from the seat when the player sits down, in meters, before it is pulled in.
pub const BOARD_REACH: f32 = 3.0;

/// Fastest the head turns toward what it looks at, in radians per second: about 200 degrees.
pub const HEAD_RATE: f32 = 3.5;

/// Spring rate of the head turning toward what it looks at, in radians per second.
pub const HEAD_SPRING: f32 = 7.0;

/// Fastest the body turns after its head on its own, in radians per second. The neck range
/// may push it faster, never faster than the head.
pub const BODY_RATE: f32 = 3.0;

/// Spring rate of the body following its head, in radians per second: it trails, then settles.
pub const BODY_SPRING: f32 = 5.5;

/// Farthest the head turns from the body, sideways and up or down, in radians: about 52 and
/// 30 degrees. Past that the body has to turn with it.
pub const NECK: (f32, f32) = (0.9, 0.5);

/// Height of the hover bob in meters and its rate in radians per second.
pub const BOB: (f32, f32) = (0.035, 2.1);

/// Lean in radians per meter per second of speed, and the largest lean in radians.
pub const TILT: (f32, f32) = (0.07, 0.38);

/// How far a hand trails behind per meter per second, and the farthest it trails, in meters.
pub const HAND_LAG: (f32, f32) = (0.035, 0.11);

/// Luminous power of the screen light in lumens at full glow and full size, in the dark: the
/// power of the lamp of the robot. A torch gives 26 000.
pub const LIGHT_POWER: f32 = 850.0;

/// Illuminance of the place in lux under which the screen light is at full power, and over which
/// it is off: the night is about 310 lux and a full day 13 000. It is a lamp for the dark.
pub const LAMP_LUX: (f32, f32) = (400.0, 2000.0);

/// Radius of the source of the screen light at full size, in meters: smaller than its distance
/// to the screen, so the light is never inside what it lights.
pub const LAMP_RADIUS: f32 = 0.04;

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

/// Weights of what it may look at when its eyes move on: the player, the way a fast player
/// runs, the ship, the door of the home base, and a look around at nothing.
pub const GAZE_WEIGHT: [f32; 5] = [3.0, 9.0, 1.6, 1.2, 2.4];

/// Player speed in metres per second above which the companion looks where the player runs.
pub const FAST_SPEED: f32 = 5.0;

/// Distance in metres within which the ship and the door of the home base catch its eye.
pub const SIGHT_REACH: (f32, f32) = (45.0, 35.0);

/// Seconds a still player must keep the camera on the companion before it looks back at the
/// camera, and seconds before it may do that again.
pub const WATCHED: (f32, f32) = (2.5, 15.0);

/// Night level, from 0 (day) to 1 (night), above which the companion is sleepy.
pub const SLEEPY_NIGHT: f32 = 0.75;

/// Depth of the eyes under water in meters beyond which the companion worries.
pub const WORRY_DEPTH: f32 = 0.25;

/// Climb rate of the ship in meters per second beyond which the companion is wide-eyed.
pub const TAKEOFF_CLIMB: f32 = 6.0;

/// Seconds the delighted face stays after a discovery.
pub const DELIGHT_SECONDS: f32 = 6.0;

/// Seconds the wide-eyed face stays after the ship leaves the ground.
pub const TAKEOFF_SECONDS: f32 = 2.5;

/// Seconds the surprised face stays after waking up.
pub const WAKE_SECONDS: f32 = 1.4;

/// Where it flies beside a flying player, in metres: to the left, behind and above the eyes.
pub const WING: (f32, f32, f32) = (2.6, 2.4, 0.4);

/// Speeds in metres per second between which the formation turns from the view of a hovering
/// player to the course of the flight.
pub const WING_SPEED: (f32, f32) = (2.0, 7.0);

/// Share of the velocity of the player it carries along on foot, and in flight.
pub const FEED_FLIGHT: f32 = 1.0;

/// Spring rate in flight, in radians per second: stiff enough that a player stopping dead from
/// full speed carries it barely past its place, never back and forth.
pub const FLIGHT_STIFFNESS: f32 = 10.0;

/// Seconds ahead it looks for the ground and the water at speed.
pub const LOOK_AHEAD: f32 = 0.8;

/// Height it keeps above the ground and the water in flight, in metres.
pub const FLIGHT_CLEARANCE: f32 = 1.6;

/// Distance in metres past which it counts as left behind by a flying player.
pub const FLIGHT_LEASH: f32 = 160.0;

/// Rate per second at which it moves between following on foot and flying in formation.
pub const FLIGHT_RATE: f32 = 2.5;

/// Push it needs against the air per metre per second of its speed, in metres per second
/// squared: what tips its nose down in flight, 25 degrees at 30 m/s and 52 at 80.
pub const SOAR_DRAG: f32 = 0.155;

/// Farthest its flying pose tips from the vertical, in radians: about 57 degrees.
pub const SOAR_LIMIT: f32 = 1.0;

/// Rate per second at which the change of its velocity is eased before the flying pose leans
/// along it, so a turn banks it over a few frames and frame noise never shakes it.
pub const GAIN_RATE: f32 = 5.0;

/// Half width of its trail in metres: under a fifth of the boot exhaust of the wingsuit, and
/// thinner than a wingtip trail, so it reads as the smaller of the two.
pub const TAIL_WIDTH: f32 = 0.02;

/// Seconds a point of its trail glows for: a third of a wingtip trail.
pub const TAIL_LIFE: f32 = 0.5;

/// Its speeds in metres per second between which the trail appears: none while it hovers.
pub const TAIL_SPEED: (f32, f32) = (8.0, 22.0);

/// Brightness of the trail, as a multiple of the colour of its light, so bloom catches it.
pub const TAIL_GAIN: f32 = 1.5;

/// How much brighter its thruster glows in flight.
pub const THRUST_GLOW: f32 = 1.8;

/// Seconds after anything it said before it says something else, but a warning.
pub const SPEAK_GAP: f32 = 8.0;

/// Seconds between two warnings.
pub const WARN_GAP: f32 = 30.0;

/// Seconds between two sounds it makes on its own, when it speaks often and when it speaks rarely.
pub const CHATTER_GAP: (f32, f32) = (60.0, 150.0);
