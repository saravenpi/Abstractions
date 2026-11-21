//! The legacy plane of the tests: the flat terrain of a small planet that the frozen tests guard.

use std::sync::Arc;

use super::Terrain;
use crate::world::globe::{Chart, Globe};

impl Terrain {
    /// Builds the flat terrain of a 6 km planet of standard gravity: the legacy plane that the frozen
    /// tests guard bit for bit, which is no longer the home planet of any universe.
    pub(crate) fn legacy(seed: u64) -> Self {
        let globe = Globe::new(seed, 6000.0, crate::universe::STANDARD_GRAVITY);
        Self::on(Arc::new(globe), Chart::Flat)
    }
}
