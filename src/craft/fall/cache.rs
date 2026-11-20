//! The cache: the gear of a destroyed robot, waiting where it fell, kept in the world file.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::craft::Pack;
use crate::saves::ActiveWorld;
use crate::universe::BodyId;

/// What a destroyed robot left behind and where, in metres in the frame of the planet it fell on.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Cache {
    /// The planet it lies on.
    pub body: BodyId,
    /// Where it lies, in metres from the centre of the planet in its turning frame.
    pub at: [f64; 3],
    /// The tools and materials that were carried.
    pub pack: Pack,
}

/// The cache of the world being played, if the robot left one.
#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct Left(pub Option<Cache>);

/// Reads the cache of the world that was just made active.
pub fn load(active: Option<Res<ActiveWorld>>, mut left: ResMut<Left>) {
    let Some(active) = active.filter(|active| active.is_added()) else {
        return;
    };
    left.0 = active.0.save.cache.clone();
}

/// Writes the cache into the active world whenever it changes, so the next save keeps it.
pub fn store(left: Res<Left>, active: Option<ResMut<ActiveWorld>>) {
    let Some(mut active) = active.filter(|_| left.is_changed()) else {
        return;
    };
    if active.0.save.cache != left.0 {
        active.0.save.cache = left.0.clone();
    }
}
