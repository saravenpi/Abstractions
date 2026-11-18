//! The pace of the focus: how much detail its speed leaves time to build and to see, with
//! hysteresis so a speed that hovers around a limit changes nothing.

use crate::world::mesh::Lod;

/// Speeds in metres per second above which the focus is fast, and under which it is no longer.
/// A wingsuit stays under the first; a ship low over the ground is over it.
pub const FAST: (f64, f64) = (120.0, 90.0);

/// Speeds in metres per second above which the focus rushes, and under which it no longer does.
pub const RUSH: (f64, f64) = (500.0, 400.0);

/// Detail radius in chunks left to a focus that rushes: the chunks that carry colliders.
const RUSH_RINGS: i32 = 1;

/// How fast the focus goes, as the streamer sees it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Pace {
    /// On foot or gliding: every level of detail is built.
    #[default]
    Calm,
    /// Flying fast: new chunks get no ground cover and no terrain finer than the far level.
    Fast,
    /// Faster than anything shows: only the chunks under the focus get their detail.
    Rush,
}

impl Pace {
    /// Returns the pace after a frame at `speed` metres per second: it rises above the first
    /// speed of a limit and only falls back under the second.
    pub fn after(self, speed: f64) -> Self {
        let over = |limit: (f64, f64), held: bool| speed > [limit.0, limit.1][usize::from(held)];
        if over(RUSH, self == Pace::Rush) {
            Pace::Rush
        } else if over(FAST, self >= Pace::Fast) {
            Pace::Fast
        } else {
            Pace::Calm
        }
    }

    /// Returns the finest terrain level a chunk is built or rebuilt at.
    pub fn finest(self) -> Lod {
        match self {
            Pace::Calm => Lod::Near,
            Pace::Fast | Pace::Rush => Lod::Far,
        }
    }

    /// Returns true when new ground cover is built.
    pub fn covers(self) -> bool {
        self == Pace::Calm
    }

    /// Returns the detail radius in chunks out of the `rings` of the render distance.
    pub fn rings(self, rings: i32) -> i32 {
        match self {
            Pace::Rush => RUSH_RINGS.min(rings),
            Pace::Calm | Pace::Fast => rings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pace rises above a limit and only falls back well under it, so a speed that hovers
    /// around the limit changes nothing.
    #[test]
    fn pace_has_hysteresis() {
        assert_eq!(Pace::Calm.after(100.0), Pace::Calm);
        assert_eq!(Pace::Calm.after(121.0), Pace::Fast);
        assert_eq!(Pace::Fast.after(100.0), Pace::Fast);
        assert_eq!(Pace::Fast.after(89.0), Pace::Calm);
        assert_eq!(Pace::Fast.after(501.0), Pace::Rush);
        assert_eq!(Pace::Rush.after(450.0), Pace::Rush);
        assert_eq!(Pace::Rush.after(399.0), Pace::Fast);
        assert_eq!(Pace::Rush.after(50.0), Pace::Calm);
        assert_eq!(Pace::Calm.after(1000.0), Pace::Rush);
        let hover = [119.0, 121.0, 119.0, 121.0, 95.0, 121.0];
        let paces = hover.iter().scan(Pace::Calm, |pace, speed| {
            *pace = pace.after(*speed);
            Some(*pace)
        });
        assert_eq!(paces.filter(|pace| *pace == Pace::Calm).count(), 1);
    }

    /// Speed only ever takes detail away: the finest level, then the cover, then the radius.
    #[test]
    fn speed_caps_the_detail() {
        assert_eq!(Pace::Calm.finest(), Lod::Near);
        assert!(Pace::Calm.covers() && !Pace::Fast.covers() && !Pace::Rush.covers());
        assert_eq!(Pace::Fast.finest(), Lod::Far);
        assert_eq!([Pace::Calm.rings(8), Pace::Fast.rings(8)], [8, 8]);
        assert_eq!(Pace::Rush.rings(8), 1);
    }
}
