//! How the focus moves: its smoothed velocity on the planet and the point it will reach in a
//! moment, which is where the streamer has to be ready.

use bevy::math::DVec3;

use super::pace::Pace;

/// Seconds over which the measured velocity is smoothed, so one odd frame steers nothing.
const SMOOTHING: f64 = 0.2;

/// Change of speed in metres per second, from one frame to the next, that no flight makes: the
/// focus was moved by hand, or hit something.
const JUMP: f64 = 300.0;

/// Speeds in metres per second between which the lead time grows from nothing to its longest.
/// A robot on foot stays under the first, so walking streams around where it stands.
pub const LEAD_SPEEDS: (f64, f64) = (12.0, 40.0);

/// Longest lead time in seconds.
pub const LEAD_SECONDS: f64 = 2.0;

/// Longest lead in metres: past it the detail to build ahead costs more than it shows.
pub const LEAD_METRES: f64 = 1200.0;

/// Returns the seconds the streamer looks ahead of a focus that moves at `speed`.
pub fn lead_seconds(speed: f64) -> f64 {
    let x = ((speed - LEAD_SPEEDS.0) / (LEAD_SPEEDS.1 - LEAD_SPEEDS.0)).clamp(0.0, 1.0);
    let seconds = LEAD_SECONDS * x * x * (3.0 - 2.0 * x);
    seconds.min(LEAD_METRES / speed.max(1.0))
}

/// The movement of the focus in the frame of the planet.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Motion {
    last: Option<DVec3>,
    raw: DVec3,
    /// Smoothed velocity in metres per second.
    pub velocity: DVec3,
    /// How much detail the speed of the focus allows, with hysteresis.
    pub pace: Pace,
}

impl Motion {
    /// Notes that the focus is at planet position `at`, `dt` seconds after the last call. A jump
    /// forgets the velocity: nothing lies ahead of a focus that was moved by hand. Without a
    /// focus on the planet everything is forgotten.
    pub fn advance(&mut self, at: Option<DVec3>, dt: f64) {
        let Some(at) = at else {
            *self = Self::default();
            return;
        };
        let raw = self.last.filter(|_| dt > 0.0).map(|last| (at - last) / dt);
        self.last = Some(at);
        let Some(raw) = raw else {
            return;
        };
        let steady = (raw - self.raw).length() < JUMP;
        self.raw = raw;
        self.velocity = match steady {
            true => self.velocity + (raw - self.velocity) * (1.0 - (-dt / SMOOTHING).exp()),
            false => DVec3::ZERO,
        };
        self.pace = self.pace.after(self.speed());
    }

    /// Returns the speed of the focus in metres per second.
    pub fn speed(&self) -> f64 {
        self.velocity.length()
    }

    /// Returns where a focus at planet position `at` will be once the lead time has passed, kept
    /// at the height it will have then and never under the sphere of `radius` metres.
    pub fn ahead(&self, at: DVec3, radius: f64) -> DVec3 {
        let seconds = lead_seconds(self.speed());
        if seconds <= 0.0 {
            return at;
        }
        let up = at.normalize_or(DVec3::Y);
        let climb = self.velocity.dot(up) * seconds;
        let over = at + (self.velocity - up * self.velocity.dot(up)) * seconds;
        over.normalize_or(up) * (at.length() + climb).max(radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns the motion of a focus that has moved steadily at `velocity` for three seconds.
    fn steady(velocity: DVec3) -> Motion {
        let mut motion = Motion::default();
        let start = DVec3::Y * 6000.0;
        let at = |i: i32| Some(start + velocity * f64::from(i) / 60.0);
        (0..=180).for_each(|i| motion.advance(at(i), 1.0 / 60.0));
        motion
    }

    /// The lead is nothing on foot, grows with speed and never passes its longest reach.
    #[test]
    fn lead_grows_with_speed_and_is_clamped() {
        assert_eq!(lead_seconds(0.0), 0.0);
        assert_eq!(lead_seconds(8.5), 0.0);
        assert!(lead_seconds(20.0) > 0.2 && lead_seconds(20.0) < lead_seconds(30.0));
        assert!(lead_seconds(30.0) > 1.0 && lead_seconds(40.0) == LEAD_SECONDS);
        assert_eq!(lead_seconds(100.0), LEAD_SECONDS);
        assert_eq!(lead_seconds(300.0) * 300.0, 600.0);
        assert!((lead_seconds(1000.0) * 1000.0 - LEAD_METRES).abs() < 1.0e-9);
        let leads = (0..400).map(|i| lead_seconds(f64::from(i) * 5.0) * f64::from(i) * 5.0);
        assert!(
            leads
                .collect::<Vec<_>>()
                .windows(2)
                .all(|w| w[0] <= w[1] + 1.0e-9)
        );
    }

    /// The velocity follows a steady flight, the point ahead lies along it at the same height,
    /// and a standing or walking focus looks nowhere but where it is.
    #[test]
    fn point_ahead_follows_the_velocity() {
        let at = DVec3::Y * 6100.0;
        let motion = steady(DVec3::X * 300.0);
        assert!((motion.velocity - DVec3::X * 300.0).length() < 0.1);
        let ahead = motion.ahead(at, 6000.0);
        assert!((ahead.length() - 6100.0).abs() < 1.0e-6);
        assert!((at.angle_between(ahead) * 6100.0 - 599.0).abs() < 2.0);
        assert!(ahead.x > 0.0 && ahead.z.abs() < 1.0e-6);
        assert_eq!(steady(DVec3::X * 8.0).ahead(at, 6000.0), at);
        assert_eq!(Motion::default().ahead(at, 6000.0), at);
    }

    /// A dive leads to a lower point, never under the ground sphere, and a jump of the focus
    /// forgets the velocity.
    #[test]
    fn dives_stop_at_the_sphere_and_jumps_reset() {
        let mut motion = steady(DVec3::new(100.0, -200.0, 0.0));
        let ahead = motion.ahead(DVec3::Y * 6100.0, 6000.0);
        assert!((ahead.length() - 6000.0).abs() < 1.0e-6);
        motion.advance(Some(DVec3::X * 6000.0), 1.0 / 60.0);
        assert_eq!(motion.velocity, DVec3::ZERO);
        motion.advance(Some(DVec3::X * 6000.0), 0.0);
        assert_eq!(motion.velocity, DVec3::ZERO);
        assert_eq!(steady(DVec3::X * 300.0).pace, Pace::Fast);
        assert_eq!(steady(DVec3::X * 30.0).pace, Pace::Calm);
        let mut gone = steady(DVec3::X * 300.0);
        gone.advance(None, 1.0 / 60.0);
        assert_eq!(gone, Motion::default());
    }
}
