//! The life a globe carries: one source of truth for the plants, the creatures and the field guide.

use std::sync::Arc;

use super::Globe;
use crate::life::Life;

impl Globe {
    /// Returns this globe carrying `life`, the life of its planet, or none on a dead world.
    pub fn with_life(mut self, life: Option<Life>) -> Self {
        self.life = Some(life.map(Arc::new));
        self
    }

    /// Returns the life of the planet, or none on a dead world or before its life was given.
    pub fn life(&self) -> Option<&Life> {
        self.life.as_ref()?.as_deref()
    }

    /// Returns whether the planet lives, or none when its life was never given, as for test globes
    /// and far views: those keep the rules of home.
    pub fn lives(&self) -> Option<bool> {
        self.life.as_ref().map(Option::is_some)
    }
}
