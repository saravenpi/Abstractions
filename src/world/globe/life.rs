//! The life a globe carries: one source of truth for the plants, the creatures and the field guide.

use std::sync::Arc;

use super::Globe;
use crate::life::Life;

impl Globe {
    /// Returns this globe carrying `life`, the life of its planet, or none on a dead world.
    pub fn with_life(mut self, life: Option<Life>) -> Self {
        self.life = life.map(Arc::new);
        self
    }

    /// Returns the life of the planet, or none on a dead world.
    pub fn life(&self) -> Option<&Life> {
        self.life.as_deref()
    }
}
