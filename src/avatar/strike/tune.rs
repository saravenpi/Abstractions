//! Tuning constants of strikes: the reading of the mouse, the guard, the hitstop and the flight of a
//! thrown spear. Lengths are metres in the frame of the torso: +X right, +Y up, -Z ahead.

/// Seconds of mouse motion that choose the direction of a swing (Mount & Blade reads it just before
/// the click).
pub const WINDOW: f32 = 0.12;

/// Mouse samples kept, more than the frames of [`WINDOW`] at any frame rate a player plays at.
pub const STROKES: usize = 32;

/// Least mouse motion over the window that picks a new direction, in pixels.
pub const DEADZONE: f32 = 3.0;

/// Seconds a raised guard parries a blow.
pub const PARRY: f32 = 0.15;

/// Seconds a raised guard parries a blow with a very light tool.
pub const PARRY_LIGHT: f32 = 0.10;

/// Mass under which a tool counts as very light, in kilograms.
pub const LIGHT: f32 = 0.5;

/// Share of the mass in the head above which a build chops or smashes overhead, unitless.
pub const HEAVY_HEAD: f32 = 0.6;

/// Least seconds from the end of a swing back to carrying the tool.
pub const RECOVER: f32 = 0.25;

/// Share of its wind-up that the arm takes to stop the tool after a swing, unitless: the arm that
/// gave the tool its speed in the wind-up takes as long to take it away, so a heavy or badly held
/// tool recovers slowly.
pub const BRAKE: f32 = 0.8;

/// Most the head carries on past the follow-through, in shares of the path, for a tool of great
/// inertia.
pub const OVERSHOOT: f32 = 0.2;

/// Moment of inertia about the hand at which a tool carries on half of [`OVERSHOOT`], in kg·m².
pub const COAST: f32 = 0.1;

/// Share of the path by which the chest leads the arm in a swing, unitless: hips and chest start the
/// blow and the arm follows.
pub const LEAD: f32 = 0.15;

/// Seconds to raise the guard.
pub const GUARD_IN: f32 = 0.1;

/// Seconds to draw a spear back fully before a throw.
pub const DRAW: f32 = 0.8;

/// Share of a full draw under which a release throws nothing, unitless.
pub const LEAST_DRAW: f32 = 0.25;

/// Longest kickback stun after a blow that gave none of its energy, in seconds.
pub const KICK: f32 = 0.1;

/// Distance from the shoulder line to the circle the grip sweeps in a swing, in metres.
pub const GRIP_RADIUS: f32 = 0.36;

/// Distance from the wrist to the middle of the grip in the hand, in metres.
pub const HAND_GRIP: f32 = 0.05;

/// Distance along the shaft from the leading hand to the second hand of a two-handed grip, in metres.
pub const SECOND_HAND: f32 = 0.12;

/// Turn of the chest over the hips at the ends of a sideways swing, in radians.
pub const TWIST: f32 = 0.45;

/// Bend of the chest at the ends of an overhead swing, in radians.
pub const SPINE: f32 = 0.35;

/// Share of the energy a blow keeps when it lands at the grip, unitless: the sweet spot is the head.
pub const SWEET_FLOOR: f32 = 0.3;

/// Hitstop frames: the least, the gain per square root joule, and the most (Smash scales it by
/// damage, https://www.ssbwiki.com/Hitlag).
pub const STOP: [f32; 3] = [3.0, 0.4, 12.0];

/// Frame rate the hitstop frames count at, in frames per second.
pub const STOP_FPS: f32 = 60.0;

/// Shake of the struck animal at the start of a hitstop, in metres; it falls to 0 by its end.
pub const SHAKE: f32 = 0.03;

/// Kick of the camera along the blow at the start of a hitstop that involves the local robot, in
/// metres, for a blow of average energy; the hitstop scales it from half to twice that and it falls
/// to nothing by the end.
pub const CAMERA_KICK: f32 = 0.02;

/// Shake of the camera at the start of a hitstop that involves the local robot, in metres.
pub const CAMERA_SHAKE: f32 = 0.012;

/// Stiffness in gigapascals and mass in kilograms of the ground a spear lands in, for its sound: a
/// loose soil, our estimate.
pub const GROUND: (f32, f32) = (0.05, 500.0);

/// Pressure soil puts against a point driven into it, in pascals.
pub const SOIL: f32 = 1.0e6;

/// Least depth a spear must reach in the ground to stand in it, in metres.
pub const STICK: f32 = 0.05;

/// Share of its speed a spear keeps when it bounces, unitless.
pub const RESTITUTION: f32 = 0.3;

/// Speed under which a bouncing spear comes to rest, in metres per second.
pub const REST_SPEED: f32 = 1.0;

/// Distance within which the robot picks a lying spear up, in metres.
pub const PICK: f32 = 1.5;

/// Upward tilt of a throw over the view, in radians.
pub const LOFT: f32 = 0.08;

/// Metres from the robot within which a spear stuck in an animal can be pulled out.
pub const PULL: f32 = 3.0;

/// How much longer a tool made for two hands takes to wind up in one hand, unitless: the grip
/// budget falls from 30 to 15 newton metres.
pub const ONE_HAND: f32 = 2.0;

/// Tip speed relative to the body, in metres per second, above which the tool leaves a trail: a tool
/// carried or raised moves the tip slower than this, a swing or a thrust faster. The trail glows
/// from nothing at this speed to full at twice it.
pub const TRAIL_SPEED: f32 = 3.0;

/// Seconds a point of the trail of a tool lasts before it has faded.
pub const TRAIL_LIFE: f32 = 0.15;

/// Opacity of the trail at its brightest, unitless.
pub const TRAIL_PEAK: f32 = 0.5;

/// Share of the colour of the tool head in the trail, unitless; the rest is white.
pub const TRAIL_TINT: f32 = 0.25;

/// Share of the surface normal under which a blow counts as glancing: the cosine of the angle between
/// the way the tool moves and the normal of what it hits. Under 0.35 (past about 70 degrees) the tool
/// slides along the surface instead of biting.
pub const GLANCE: f32 = 0.35;

/// How much longer the recovery takes after a swing that met nothing: no target braked the tool, so
/// the arm has to, unitless.
pub const MISS_RECOVER: f32 = 1.5;

/// Share of the path a tool rebounds along after a blunt blow that the body took, for the most
/// elastic contact, unitless.
pub const REBOUND: f32 = 0.15;

/// Restitution of the stiffest contact, unitless: two bodies of infinite stiffness bounce this much.
pub const BOUNCE: f32 = 0.6;

/// Contact stiffness in gigapascals at which the restitution is half of [`BOUNCE`].
pub const BOUNCE_HALF: f32 = 5.0;

/// Metres from the tip at which a swing that met nothing is seen and heard by an animal.
pub const MISS_RANGE: f32 = 4.0;

/// Awareness, 0 to 1, that a swing that passes at the tip adds to an animal: it falls to nothing at
/// [`MISS_RANGE`].
pub const MISS_ALERT: f32 = 0.5;

/// Metres a resting spear moves before the pack notes its new place: a spear in an animal that
/// walks is noted every few steps, not every frame.
pub const STRAY_SLACK: f64 = 0.5;

/// Metres from the robot within which a tool that is out and has no spear is put back in the
/// world. The ground is solid that near, so the spear lands on it.
pub const STRAY_NEAR: f32 = 60.0;

/// Metres above its noted place a tool put back in the world falls from.
pub const STRAY_LIFT: f32 = 1.0;
