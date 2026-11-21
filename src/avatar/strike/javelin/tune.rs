//! Tuning constants of the javelin: the ground it meets, its flight, its trail, the pose that
//! draws it back along the aim and the reach that takes it again. Lengths are metres; the frames
//! of the poses are +X right, +Y up, +Z back.

/// Stiffness in gigapascals and mass in kilograms of rock or ice a spear glances off, for its
/// clack: granite is about 50 GPa.
pub const ROCK: (f32, f32) = (50.0, 2000.0);

/// Pressure the soils put against a point driven into them, in pascals: earth under grass, sand,
/// mud, snow. Our estimates of the cone resistance of each.
pub const SOILS: [f32; 4] = [1.0e6, 0.6e6, 0.3e6, 0.15e6];

/// Deepest a spear drives into the ground, in metres.
pub const DEEPEST: f32 = 0.3;

/// Least cosine of the angle between the shaft and the path for the tip to lead, unitless: 25°.
pub const LEADS: f32 = 0.9;

/// Least sine of the angle between the path and the ground for a point to bite, unitless: 12°.
pub const STEEP: f32 = 0.2;

/// Share of its speed along a surface a spear keeps when it glances off it, unitless.
pub const SKID: f32 = 0.6;

/// Metres of flight over which the shaft of a javelin turns 63 % of the way to its path: its
/// centre of pressure lies behind its centre of mass, so the air turns it nose first.
pub const VANE: f32 = 1.5;

/// Turn of a spear balanced behind its middle, which tumbles end over end, in radians per second.
pub const TUMBLE: f32 = 8.0;

/// Height of the axis of a lying shaft above the ground, in metres: about its radius.
pub const LIE: f32 = 0.015;

/// Halvings of a step that finds where a spear meets the drawn ground: 2⁻¹² of a step.
pub const HALVINGS: usize = 12;

/// Metres along the chart between the samples that give the slope of the drawn ground.
pub const SLOPE: f32 = 0.5;

/// Wobble of a planted shaft: radians per metre per second of arrival, the most, its frequency in
/// hertz and the seconds it takes to fall by e.
pub const QUIVER: [f32; 4] = [0.004, 0.12, 7.0, 0.25];

/// Seconds a planted shaft wobbles before it stands still.
pub const QUIVER_LIFE: f32 = 1.0;

/// Farthest from the robot a spear on the ground can be and still be taken, in metres.
pub const TAKE_REACH: f32 = 2.5;

/// Farthest from the line of sight a spear on the ground can lie and still be taken, in metres.
pub const TAKE_AIM: f32 = 0.8;

/// Seconds a point of the trail of a flying javelin lasts before it has faded.
pub const STREAK_LIFE: f32 = 0.5;

/// Least seconds between two points of the trail of a flying javelin: rows of the ring last the
/// whole life of the trail at any frame rate.
pub const STREAK_EVERY: f32 = 1.0 / 60.0;

/// Half the width of the trail of a flying javelin, in metres: a few pixels at ten metres.
pub const STREAK_HALF: f32 = 0.03;

/// Opacity of the trail of a flying javelin at its brightest, unitless.
pub const STREAK_PEAK: f32 = 0.6;

/// Farthest the crosshair looks along the drawn ground for the point a javelin is thrown at, in
/// metres; past it the throw goes along the ray.
pub const THROW_SIGHT: f32 = 80.0;

/// Steps of the march along the crosshair ray over the drawn ground: the first, in metres, and the
/// share of the distance each step adds.
pub const MARCH: (f32, f32) = (0.5, 0.08);

/// Where the throwing hand holds a drawn javelin from the right shoulder, in the frame of the aim
/// turned about the up only: out to the right, up and back. The arm is almost straight behind.
pub const DRAWN_BACK: [f32; 3] = [0.24, 0.04, 0.42];

/// Where the throwing hand holds a drawn javelin from the eyes in first person, in the frame of
/// the view: right, up, back. It sits at the lower right of the picture.
pub const VIEW_HOLD: [f32; 3] = [0.20, -0.17, -0.30];

/// Where the neck sits above the middle of the body, and the eyes from the neck in the frame of the
/// view (right, up, back), as the camera rig puts them in first person.
pub const EYES: ([f32; 3], [f32; 3]) = ([0.0, 0.48, 0.0], [0.0, 0.22, -0.16]);

/// Turn and bend of the chest with a javelin drawn back, in radians: coiled away from the throw.
pub const COIL: [f32; 2] = [-0.4, -0.12];

/// Turn and bend of the chest at the end of the throw, in radians: through the aim line.
pub const THROUGH: [f32; 2] = [0.35, 0.2];

/// Seconds the hand takes to whip through the aim line, and the seconds the release then takes
/// to fade back to carrying nothing.
pub const RELEASE: (f32, f32) = (0.1, 0.25);

/// Metres ahead of the shoulder the throwing hand reaches at the end of the whip, along the aim.
pub const WHIP: f32 = 0.44;

/// Metres ahead of the left shoulder the free hand points toward the target while drawn.
pub const POINT: f32 = 0.42;
