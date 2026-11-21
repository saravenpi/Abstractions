//! Tuning constants of how a strike feels: the aim, the body behind the blow, the stow, the knap,
//! the flecks of a contact and the mark of a blow. Lengths are metres in the frame of the body: +X
//! right, +Y up, -Z ahead.

/// Metres the camera ray looks along for an animal under the crosshair.
pub const AIM_SIGHT: f32 = 12.0;

/// Padding of the bounds of a part for the aim, in metres: a small animal near the crosshair is
/// under it.
pub const AIM_PAD: f32 = 0.12;

/// Metres past the reach of the tool and the arm that the body adds by stooping and stepping into a
/// blow.
pub const LUNGE: f32 = 0.35;

/// Least height of the aim above the soles, in metres: a swing never aims into the ground.
pub const GROUND_CLEAR: f32 = 0.08;

/// Height in the frame of the body under which a target bends the body, and the span under it over
/// which it bends fully, in metres: the hips, and about the soles.
pub const LOW: (f32, f32) = (0.0, 0.75);

/// Drop of the pelvis in metres, forward lean of the body and bend of the chest in radians of a
/// body fully bent to a target at its feet, as a person stoops with bent knees.
pub const STOOP: [f32; 3] = [0.38, 0.4, 0.35];

/// Least and most tilt of the path of a strike toward its aim, in radians: negative aims down.
pub const TILT: (f32, f32) = (-1.2, 0.7);

/// Most turn of the path of a strike to either side toward its aim, in radians.
pub const YAW: f32 = 0.7;

/// Metres the front foot stands ahead of the hip and the back foot behind it during a strike.
pub const STAGGER: f32 = 0.12;

/// Metres the front foot steps further into a swing, at the most coiled and the most followed
/// through.
pub const STEP: f32 = 0.08;

/// How much longer the step into a thrust or a throw is than into a swing, unitless: a lunge.
pub const LUNGE_STEP: f32 = 2.2;

/// Lean back over the back foot when fully cocked and lean into the blow when followed through, in
/// radians.
pub const LEAN: (f32, f32) = (0.06, 0.14);

/// Shift of the pelvis over the back foot when cocked and over the front one when followed
/// through, in metres. The pose keeps it under 5 cm.
pub const SWAY: f32 = 0.035;

/// Drop of the pelvis as the knees give through the blow, in metres.
pub const DIP: f32 = 0.05;

/// Share of the body in a blow for the lightest tool and what a tool of great inertia adds,
/// unitless: a heavy tool is swung from the legs, a light one from the arm.
pub const HEFT: (f32, f32) = (0.6, 0.8);

/// Shoulder swing, arm lift and elbow bend of the free arm reaching ahead while the tool is
/// cocked, at rest, and tucked back while it follows through, in radians: it balances the blow.
pub const COUNTER: [[f32; 3]; 3] = [[0.9, 0.25, 0.5], [0.15, 0.12, 0.5], [-0.35, 0.35, 1.3]];

/// Rate at which the legs and the free arm follow the strike, per second.
pub const EASE: f32 = 30.0;

/// Seconds to sling the tool over the back before the wingsuit opens or the robot boards.
pub const STOW: f32 = 0.35;

/// Where the grip of a slung tool sits in the frame of the torso: behind the right hip.
pub const SLING: [f32; 3] = [0.12, -0.05, 0.15];

/// Which way the shaft of a slung tool points from its grip in the frame of the torso: up across
/// the back to the left shoulder.
pub const SLING_AXIS: [f32; 3] = [-0.35, 1.0, 0.05];

/// Seconds of a knap: the wind-up, the blow down to the rock, and the way back.
pub const KNAP: [f32; 3] = [0.22, 0.1, 0.22];

/// Share of the overhead path at which the head of a tool meets what it aims at: where the path
/// points straight ahead of the shoulder.
pub const KNAP_MEET: f32 = 0.7;

/// Contact stiffness above which a blow on stone throws sparks, in gigapascals: flint on a hard
/// head sparks, a fist or a soft head does not.
pub const SPARKING: f32 = 15.0;

/// Seconds a hit mark shows at the crosshair.
pub const MARK_LIFE: f32 = 0.35;

/// Seconds the state of the animal last struck shows after the blow.
pub const FIGHT: f32 = 6.0;

/// Most flecks one contact throws of each kind.
pub const FLECKS: f32 = 8.0;

/// Radii of a fleck of dust, of a tuft of fur, of a chip and of a spark, in metres.
pub const RADII: [f32; 4] = [0.012, 0.008, 0.007, 0.004];

/// Linear RGB of dust off stone and soil, of fur off hide, of chips off bone or horn, of flakes off
/// shell and of the robot's plate.
pub const TONES: [[f32; 3]; 5] = [
    [0.42, 0.38, 0.32],
    [0.30, 0.24, 0.18],
    [0.80, 0.76, 0.66],
    [0.16, 0.14, 0.12],
    [0.55, 0.57, 0.60],
];

/// Linear RGB of a spark, above 1 so it glows: hot metal or flint.
pub const SPARK: [f32; 3] = [6.0, 2.6, 0.7];

/// Metres past the tip a held tool keeps clear of the world: it rises before it touches a wall.
pub const MARGIN: f32 = 0.1;

/// Seconds a held tool raised out of the way of a wall takes to come back down once it is clear.
pub const CRAMP_EASE: f32 = 0.15;

/// Halvings of the raise that keeps a held tool clear: 2⁻⁶ of the way up.
pub const CRAMP_HALVES: usize = 6;
