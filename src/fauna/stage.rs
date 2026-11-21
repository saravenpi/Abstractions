//! The stage of the fauna tests: an app with the frame systems and the fauna systems on one spot of a
//! globe, stepped by hand, and what the tests read of it.

use std::sync::Arc;
use std::time::Duration;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::beast::Beast;
use super::gate::Spec;
use super::herds::Herds;
use super::rig::{SEED, view};
use super::{fade, gate, looks, scan, senses, walk};
use crate::player::Player;
use crate::space::{Anchor, Clock, Location, Travel};
use crate::universe::{BodyId, Universe};
use crate::world::Globe;

/// Returns an app on `globe`, the planet `body`, with the robot standing at unit direction `dir`
/// at universe `time`.
pub fn stage(globe: Arc<Globe>, body: BodyId, dir: DVec3, time: f64) -> App {
    let (terrain, anchor, ground) = view(&globe, body, dir);
    let mut app = App::new();
    crate::space::plugin(&mut app);
    app.insert_resource(terrain)
        .insert_resource(anchor)
        .insert_resource(Universe::new(SEED))
        .insert_resource(Location {
            body,
            travel: Travel::OnFoot,
        })
        .insert_resource(Clock { seconds: time })
        .init_resource::<Time>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>();
    fauna(&mut app);
    app.world_mut()
        .spawn((Player, Transform::from_translation(ground + Vec3::Y)));
    app
}

/// Adds the resources and the systems of the fauna to `app`.
fn fauna(app: &mut App) {
    app.init_resource::<Herds>()
        .init_resource::<super::Nearby>()
        .init_resource::<Spec>()
        .init_resource::<looks::Looks>()
        .init_resource::<senses::Blood>()
        .add_systems(
            Update,
            (
                gate::forget,
                scan::scan,
                senses::sense,
                walk::walk,
                super::near::mark,
                fade::fade,
            )
                .chain(),
        );
}

/// Moves the rig `seconds` on in frames of `dt` seconds: game time, universe time, then one update.
pub fn run(app: &mut App, seconds: f32, dt: f32) {
    let steps = (seconds / dt).round() as u32;
    for _ in 0..steps {
        let world = app.world_mut();
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(dt));
        world.resource_mut::<Clock>().seconds += f64::from(dt);
        world.run_schedule(Update);
        world.run_schedule(PostUpdate);
    }
}

/// Returns the drawn animals that are not leaving.
pub fn staying(app: &mut App) -> Vec<Beast> {
    let world = app.world_mut();
    let mut beasts = world.query::<&Beast>();
    beasts
        .iter(world)
        .filter(|b| !b.fade.leaving())
        .cloned()
        .collect()
}

/// Returns the planet position of the local robot.
pub fn robot(app: &mut App) -> DVec3 {
    let world = app.world_mut();
    let anchor = *world.resource::<Anchor>();
    let mut robot = world.query_filtered::<&Transform, With<Player>>();
    robot
        .single(world)
        .map_or(DVec3::ZERO, |t| anchor.to_frame(t.translation))
}

/// Puts the local robot at planet position `pos`.
pub fn put(app: &mut App, pos: DVec3) {
    let world = app.world_mut();
    let anchor = *world.resource::<Anchor>();
    let mut robot = world.query_filtered::<&mut Transform, With<Player>>();
    if let Ok(mut t) = robot.single_mut(world) {
        t.translation = anchor.to_render(pos);
    }
}
