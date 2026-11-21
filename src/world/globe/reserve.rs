//! Reserved ground: a disc of a planet that props keep out of.

use bevy::math::DVec3;

use super::Globe;

/// A square of ground kept free of every plant: the landing pad of a base.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pad {
    /// Centre of the square in metres from the planet centre.
    pub centre: DVec3,
    /// The vertical at the centre, a unit vector.
    pub up: DVec3,
    /// Horizontal unit direction of one pair of sides.
    pub heading: DVec3,
    /// Half the side of the square in metres.
    pub half: f64,
}

impl Pad {
    /// Returns true when planet position `pos` lies over the square grown by `margin` metres.
    pub fn covers(&self, pos: DVec3, margin: f64) -> bool {
        let from = pos - self.centre;
        let side = self.up.cross(self.heading);
        from.dot(self.heading).abs().max(from.dot(side).abs()) <= self.half + margin
    }
}

/// A disc of ground kept free of props, such as the footprint of the home base.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reserve {
    /// Centre of the disc in metres from the planet centre.
    pub centre: DVec3,
    /// The vertical at the centre, a unit vector.
    pub up: DVec3,
    /// Radius of the disc in metres, measured level around the centre.
    pub radius: f64,
    /// The landing pad inside the disc, where no plant of any size stands.
    pub pad: Option<Pad>,
}

impl Reserve {
    /// Returns true when planet position `pos` lies over the disc grown by `margin` metres.
    pub fn covers(&self, pos: DVec3, margin: f64) -> bool {
        let from = pos - self.centre;
        (from - self.up * from.dot(self.up)).length() <= self.radius + margin
    }
}

impl Globe {
    /// Returns this globe with `reserve` kept free of props.
    pub fn reserving(mut self, reserve: Reserve) -> Self {
        self.reserve = Some(reserve);
        self
    }

    /// Returns the reserved disc of the globe, when it has one.
    pub fn reserve(&self) -> Option<&Reserve> {
        self.reserve.as_ref()
    }

    /// Returns true when planet position `pos` lies within `margin` metres of the landing pad of the reserve.
    pub fn on_pad(&self, pos: DVec3, margin: f64) -> bool {
        self.reserve
            .and_then(|r| r.pad)
            .is_some_and(|pad| pad.covers(pos, margin))
    }

    /// Returns true when planet position `pos` lies within `margin` metres of reserved ground.
    pub fn reserved(&self, pos: DVec3, margin: f64) -> bool {
        self.reserve.is_some_and(|r| r.covers(pos, margin))
    }
}
