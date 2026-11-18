//! How far the minimap sees: the range steps, the step a speed asks for, the cap a small planet
//! puts on them, and the easing from one range to the next.

/// Distance in metres from the player to the edge of the map, one per step: on foot, gliding
/// and flying fast.
pub const RANGES: [f32; 3] = [250.0, 800.0, 2500.0];

/// Speed in metres per second above which the map takes the next step: faster than a sprint,
/// then faster than a glide.
const FASTER: [f32; 2] = [25.0, 120.0];

/// Share of the speed that raised a step under which the map comes back down, so a speed
/// that hovers around a limit does not flip the map.
const SLOWER: f32 = 0.7;

/// Largest range as a share of the radius of the planet: past it the ground curves away.
const REACH: f32 = 0.35;

/// How fast the shown range closes on its step, per second.
const EASE: f32 = 4.0;

/// How close to its step, as a ratio, an easing range snaps onto it.
const SNAP: f32 = 0.005;

/// What the minimap key cycles through.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// The range follows the speed.
    #[default]
    Auto,
    /// The range stays on this step.
    Fixed(usize),
    /// The minimap is hidden.
    Off,
}

impl Mode {
    /// Returns the mode the key goes to: the speed, each fixed step from the nearest, off,
    /// and the speed again.
    pub fn next(self) -> Mode {
        match self {
            Mode::Auto => Mode::Fixed(0),
            Mode::Fixed(step) if step + 1 < RANGES.len() => Mode::Fixed(step + 1),
            Mode::Fixed(_) => Mode::Off,
            Mode::Off => Mode::Auto,
        }
    }
}

/// Returns the step for a ground speed in metres per second, coming from step `current`: it
/// goes up past a limit and comes back down only well under it.
pub fn step(speed: f32, current: usize) -> usize {
    let over = |share: f32| {
        FASTER
            .iter()
            .filter(|limit| speed > **limit * share)
            .count()
    };
    current.clamp(over(1.0), over(SLOWER))
}

/// Returns the range in metres the map should show on a planet of `planet` metres of radius,
/// or nothing while the map is off.
pub fn range(mode: Mode, step: usize, planet: f64) -> Option<f32> {
    let step = match mode {
        Mode::Auto => step,
        Mode::Fixed(step) => step,
        Mode::Off => return None,
    };
    let most = planet as f32 * REACH;
    Some(RANGES[step.min(RANGES.len() - 1)].min(most))
}

/// Returns the shown range `seconds` later on its way from `radius` to `goal`: by ratio, so
/// zooming out looks as fast as zooming in. A map that showed nothing yet starts on the goal.
pub fn eased(radius: f32, goal: f32, seconds: f32) -> f32 {
    if radius <= 0.0 {
        return goal;
    }
    let share = 1.0 - (-EASE * seconds).exp();
    let next = radius * (goal / radius).powf(share);
    match (next / goal).ln().abs() < SNAP {
        true => goal,
        false => next,
    }
}
