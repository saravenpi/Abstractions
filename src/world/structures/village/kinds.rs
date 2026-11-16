//! The house kinds of a village, their sizes and the weighted choice among them.

use bevy::prelude::*;
use rand::RngExt;
use rand_chacha::ChaCha8Rng;

/// Every kind of building a village can hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum House {
    /// Small one-room house.
    Cottage,
    /// Long house with many windows.
    Long,
    /// Two storeys with an outside stair.
    TwoStory,
    /// House with a covered porch.
    Porch,
    /// Workshop with an open front and a big chimney.
    Smithy,
    /// Big barn with hay.
    Barn,
    /// Narrow tower house.
    Tower,
    /// Open market stall.
    Stall,
    /// Small storage shed.
    Shed,
    /// The village hall with a bell turret, one per village.
    Hall,
}

/// Houses a lot can get, with their weights. The hall is placed separately.
const POOL: [(House, f32); 9] = [
    (House::Cottage, 5.0),
    (House::Long, 3.0),
    (House::TwoStory, 2.0),
    (House::Porch, 3.0),
    (House::Smithy, 1.5),
    (House::Barn, 1.5),
    (House::Tower, 1.0),
    (House::Stall, 2.5),
    (House::Shed, 3.0),
];

impl House {
    /// Returns half length, half depth and the extra depth of a porch in front of the door.
    pub fn dims(self) -> Vec3 {
        match self {
            House::Cottage => Vec3::new(2.8, 2.2, 0.0),
            House::Long => Vec3::new(4.6, 2.3, 0.0),
            House::TwoStory => Vec3::new(3.0, 2.6, 0.0),
            House::Porch => Vec3::new(3.2, 2.4, 1.6),
            House::Smithy => Vec3::new(3.4, 2.6, 0.0),
            House::Barn => Vec3::new(4.2, 3.2, 0.0),
            House::Tower => Vec3::new(2.3, 2.3, 0.0),
            House::Stall => Vec3::new(2.4, 1.7, 0.0),
            House::Shed => Vec3::new(1.9, 1.6, 0.0),
            House::Hall => Vec3::new(5.0, 3.6, 0.0),
        }
    }

    /// Returns the radius of the circle used to keep lots apart.
    pub fn radius(self) -> f32 {
        let d = self.dims();
        (d.x + d.y + d.z) * 0.5 + 0.9
    }
}

/// Picks a house by weight, favoring kinds the village does not hold yet.
pub fn pick(rng: &mut ChaCha8Rng, used: &[House]) -> House {
    let weight =
        |h: House, w: f32| w * 0.4f32.powi(used.iter().filter(|u| **u == h).count() as i32);
    let total: f32 = POOL.iter().map(|(h, w)| weight(*h, *w)).sum();
    let mut roll = rng.random_range(0.0..total);
    for (h, w) in POOL {
        roll -= weight(h, w);
        if roll <= 0.0 {
            return h;
        }
    }
    House::Cottage
}
