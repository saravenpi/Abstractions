//! Looking at the rig: the ship, the player and the ground, in the frames the game uses.

use avian3d::prelude::*;
use bevy::ecs::system::RunSystemOnce;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::Game;
use crate::player::{BODY_LENGTH, BODY_RADIUS, Motion, Player};
use crate::ship::Ship;
use crate::space::{Anchor, Frame};
use crate::universe::BodyId;
use crate::world::Planet;

/// The player body as the tests see it.
#[derive(Debug, Clone, Copy)]
pub struct Walker {
    /// Planet the anchor is fixed to, when it is fixed to one.
    pub body: Option<BodyId>,
    /// Position of the body centre in the anchor frame.
    pub at: DVec3,
    /// Render position of the feet.
    pub feet: Vec3,
    /// Height of the feet above the ground the planet generates, in metres.
    pub gap: f64,
    /// Whether the controller stands on walkable ground.
    pub grounded: bool,
    /// Whether the body is switched off, as in the seat.
    pub seated: bool,
}

impl Game {
    /// Returns a copy of resource `T`.
    pub fn get<T: Resource + Clone>(&self) -> T {
        self.app.world().resource::<T>().clone()
    }

    /// Returns a copy of the ship.
    pub fn ship(&mut self) -> Ship {
        let world = self.app.world_mut();
        let mut ships = world.query::<&Ship>();
        ships.single(world).expect("one ship").clone()
    }

    /// Changes the ship in place.
    pub fn with_ship(&mut self, change: impl FnOnce(&mut Ship)) {
        let world = self.app.world_mut();
        let mut ships = world.query::<&mut Ship>();
        change(&mut ships.single_mut(world).expect("one ship"));
    }

    /// Returns the player body measured against the streamed planet.
    pub fn walker(&mut self) -> Walker {
        let (anchor, planet) = (self.get::<Anchor>(), self.get::<Planet>());
        let world = self.app.world_mut();
        let mut bodies =
            world.query_filtered::<(&Transform, &Motion, Has<ColliderDisabled>), With<Player>>();
        let (transform, motion, seated) = bodies.single(world).expect("one player");
        let at = anchor.to_frame(transform.translation);
        let half = f64::from(0.5 * BODY_LENGTH + BODY_RADIUS);
        let ground = planet.globe.radius() + f64::from(planet.globe.height(at.normalize()));
        Walker {
            body: match anchor.frame {
                Frame::Surface(body) => Some(body),
                _ => None,
            },
            at,
            feet: transform.translation - transform.up() * half as f32,
            gap: at.length() - half - ground,
            grounded: motion.grounded,
            seated,
        }
    }

    /// Metres above a point the probe for the ground starts, and metres it reaches down from there: hills and
    /// valleys of 60 m around the point still show their ground, where a short probe starts inside a rise or
    /// ends above a drop.
    const PROBE: (f32, f32) = (60.0, 160.0);

    /// Returns true when a collider of the world lies within [`PROBE`] of the height of render position `at`.
    pub fn has_ground(&mut self, at: Vec3) -> bool {
        let anchor = self.get::<Anchor>();
        let up = crate::space::up(&anchor, at);
        type Excluded = Or<(With<Ship>, With<Player>)>;
        let probe = move |query: SpatialQuery, excluded: Query<Entity, Excluded>| {
            let filter = SpatialQueryFilter::from_excluded_entities(excluded.iter());
            let down = Dir3::new(-up).expect("a vertical");
            query
                .cast_ray(at + up * Self::PROBE.0, down, Self::PROBE.1, true, &filter)
                .is_some()
        };
        let found = self.app.world_mut().run_system_once(probe);
        found.expect("the probe runs")
    }
}
