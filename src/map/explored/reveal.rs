//! Uncovering the ground around the player twice a second, and around the home base once.

use std::time::Duration;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use super::Explored;
use crate::home::Base;
use crate::player::{Player, local};
use crate::screens::Screen;
use crate::sim::PlayerId;
use crate::space::{Anchor, Frame};
use crate::world::Terrain;
use crate::world::globe::Grain;

/// Metres uncovered around a player on the ground.
pub const REACH_FOOT: f64 = 300.0;

/// Most metres uncovered around a player, however high.
pub const REACH_CAP: f64 = 2000.0;

/// Seconds between two looks around the player.
const EVERY: f32 = 0.5;

/// Returns the metres uncovered around a player `height` metres above the ground of a body of
/// sea level `radius`: the distance to the horizon, at least [`REACH_FOOT`] and at most
/// [`REACH_CAP`].
pub fn reach(height: f64, radius: f64) -> f64 {
    let height = height.max(0.0);
    (2.0 * radius * height + height * height)
        .sqrt()
        .clamp(REACH_FOOT, REACH_CAP)
}

/// Where the local player is on the body under it.
#[derive(SystemParam)]
struct Eyes<'w, 's> {
    /// The frame poses are read in.
    anchor: Option<Res<'w, Anchor>>,
    /// The ground of the body under the player.
    terrain: Option<Res<'w, Terrain>>,
    /// The player bodies.
    bodies: Query<'w, 's, (&'static PlayerId, &'static GlobalTransform), With<Player>>,
}

/// Uncovers the cells around the local player while it is near a body.
fn walk(eyes: Eyes, mut explored: ResMut<Explored>) {
    let (Some(anchor), Some(terrain)) = (eyes.anchor.as_deref(), eyes.terrain.as_deref()) else {
        return;
    };
    let Frame::Surface(body) = anchor.frame else {
        return;
    };
    let at = local(
        eyes.bodies
            .iter()
            .map(|(id, body)| (*id, body.translation())),
    );
    let pos = at.filter(|_| !anchor.flat).map(|at| anchor.to_frame(at));
    let Some((pos, dir)) = pos.and_then(|pos| Some((pos, pos.try_normalize()?))) else {
        return;
    };
    let globe = terrain.globe();
    let ground = globe.height_seen(dir, Grain::Far).max(globe.sea_level());
    let height = pos.length() - globe.radius() - f64::from(ground);
    explored.reveal(body, dir, reach(height, globe.radius()) / globe.radius());
}

/// Uncovers the ground around the home base once it is found, so that an old world shows it.
fn home(base: Res<Base>, mut explored: ResMut<Explored>) {
    let radius = base.centre.length().max(1.0);
    explored.reveal(base.body, base.centre / radius, REACH_CAP / radius);
}

/// Forgets the ground of the world that ended.
fn forget(mut explored: ResMut<Explored>) {
    *explored = Explored::default();
}

/// Registers the uncovering of the ground.
pub fn plugin(app: &mut App) {
    app.init_resource::<Explored>()
        .add_systems(
            Update,
            (
                walk.run_if(resource_exists::<Anchor>)
                    .run_if(on_timer(Duration::from_secs_f32(EVERY))),
                home.run_if(resource_added::<Base>),
            ),
        )
        .add_systems(OnEnter(Screen::Title), forget);
}
