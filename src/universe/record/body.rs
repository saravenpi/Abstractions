//! The body a record is derived for: what it is made of and where it orbits.

/// What a body is made of and where, as its record needs it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::universe) struct Body {
    /// Surface gravity in metres per second squared.
    pub gravity: f64,
    /// Radius in metres.
    pub radius: f64,
    /// Radius in metres of its orbit around the star, its planet's for a moon.
    pub orbit: f64,
    /// Heat of the tides, in units of flux.
    pub tide: f64,
}

impl Body {
    /// Returns a body that no tide heats.
    pub fn new(gravity: f64, radius: f64, orbit: f64) -> Self {
        Self {
            gravity,
            radius,
            orbit,
            tide: 0.0,
        }
    }
}
