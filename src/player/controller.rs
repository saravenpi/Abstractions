//! Kinematic character controller built on avian's move and slide.
//!
//! The movement rules run in the upright frame of the body, where +Y is the local up. The
//! flight gear takes the body over in the air; walking and swimming are the rules of `walk`.

use std::time::Duration;

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::asks::{Intents, ask, gather};
use super::flight::{Air, clearance, density, settle};
use super::ground::probe;
use super::keys::send;
use super::lead::lead;
use super::sea::above_sea;
use super::steer::{Steer, Turn};
use super::upright::{facing, tilt};
use super::walk::{Step, velocity};
use super::{Player, PlayerSet};
use crate::gear::Gear;
use crate::screens::playing;
use crate::sim::IntentOf;
use crate::space::{Anchor, altitude, local_gravity, on_foot, up};
use crate::view::ViewSet;
use crate::wind::{Thickness, Wind};
use crate::world::Terrain;

/// Longest simulated frame, to survive hitches.
const MAX_DT: f32 = 0.05;

/// Movement state owned by the controller.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Motion {
    /// Current velocity in the upright frame of the body: +Y is the local up.
    pub velocity: Vec3,
    /// Heading of the body in radians around the local up. The body orientation follows it. In
    /// first person it is the heading of the view; in third person the body turns to where it goes.
    pub yaw: f32,
    /// Whether walkable ground was touched on the last step.
    pub grounded: bool,
    /// Remaining seconds in which a jump is still allowed.
    pub coyote: f32,
    /// Whether the body is swimming instead of walking.
    pub swimming: bool,
    /// Whether the wingsuit is asked open. Only the air keeps it: landing and water fold it.
    pub wings: bool,
    /// Whether the boots are fired: the run key is held in the air, pressed there, on the wings
    /// or in a fast fall. A run that only carries on through a jump never fires them.
    pub boots: bool,
    /// Whether the held jump key already worked the gear in the air, and so starts no jump
    /// until let go.
    pub spent: bool,
    /// The heading the movement keys are read against.
    pub steer: Steer,
    /// How the body turns to its heading.
    pub turn: Turn,
}

/// Services the controller needs besides the player query.
#[derive(SystemParam)]
pub(super) struct Env<'w, 's> {
    time: Res<'w, Time>,
    anchor: Res<'w, Anchor>,
    terrain: Res<'w, Terrain>,
    wind: Option<Res<'w, Wind>>,
    air: Option<Res<'w, Thickness>>,
    slide: MoveAndSlide<'w, 's>,
    query: SpatialQuery<'w, 's>,
}

impl Env<'_, '_> {
    /// Reads the per-frame values of a body at render position `at` that is asked `asks`.
    fn step(&self, at: Vec3, asks: &Intents) -> Step {
        let globe = self.terrain.globe();
        Step {
            dt: self.time.delta_secs().min(MAX_DT),
            time: self.time.elapsed_secs(),
            view: asks.view(),
            height: altitude(&self.anchor, globe, at),
            afloat: above_sea(&self.anchor, globe, at),
            gravity: local_gravity(&self.anchor, globe, at).length(),
            air: self.air.as_deref().map_or(1.0, |air| air.0),
        }
    }

    /// Reads the air around a body with `free` metres of height under its soles.
    ///
    /// The wind blows along the tangent plane of the anchor in render axes; carried to the
    /// body by the tilt of its vertical, it has the same components in the upright frame.
    fn air(&self, free: f32, step: &Step) -> Air {
        let wind = self.wind.as_deref().map_or(Vec3::ZERO, Wind::velocity);
        Air {
            density: density(self.terrain.globe(), step.height),
            wind: wind.with_y(0.0),
            clearance: free,
            look: step.view.direction(),
            aim: Vec2::new(step.view.yaw, step.view.pitch),
            gravity: step.gravity,
        }
    }

    /// Moves a body at `from` by `velocity`, given in render axes, for `dt` seconds, sliding
    /// along what it meets.
    fn moved(
        &self,
        (collider, from): (&Collider, &Transform),
        velocity: Vec3,
        dt: f32,
        filter: &SpatialQueryFilter,
    ) -> MoveAndSlideOutput {
        self.slide.move_and_slide(
            collider,
            from.translation,
            from.rotation,
            velocity,
            Duration::from_secs_f32(dt),
            &MoveAndSlideConfig::default(),
            filter,
            |_| MoveAndSlideHitResponse::Accept,
        )
    }
}

/// The parts of a player body the controller moves.
type Body = (
    Entity,
    &'static mut Transform,
    &'static mut Motion,
    &'static Collider,
    &'static mut Gear,
    &'static Intents,
);

/// Moves every player body by the intents of its player.
pub(super) fn drive(env: Env, mut players: Query<Body, With<Player>>) {
    for body in &mut players {
        step_body(&env, body);
    }
}

/// Runs one step of intents, ground detection, flight gear and collision-aware movement.
///
/// The free height under the body is only measured off the ground, where the gear reads it.
fn step_body(
    env: &Env,
    (entity, mut transform, mut motion, collider, mut gear, asks): (
        Entity,
        Mut<Transform>,
        Mut<Motion>,
        &Collider,
        Mut<Gear>,
        &Intents,
    ),
) {
    let step = env.step(transform.translation, asks);
    let to_render = tilt(up(&env.anchor, transform.translation));
    let intent = ask(&mut motion, asks, &step);
    let filter = SpatialQueryFilter::from_excluded_entities([entity]);
    let floor = probe(&env.query, collider, &transform, to_render, &filter);
    let down = to_render * Vec3::NEG_Y;
    let free = match motion.grounded {
        true => 0.0,
        false => clearance(&env.query, transform.translation, down, &filter),
    };
    let air = env.air(free, &step);
    let wanted = velocity((&mut motion, &mut gear), &intent, floor, &air, &step);
    let out = env.moved((collider, &transform), to_render * wanted, step.dt, &filter);
    let moved = to_render.inverse() * out.projected_velocity;
    motion.velocity = match (gear.state.mode.drives(), motion.grounded) {
        (true, _) => settle(&mut gear, moved),
        (false, true) => moved.with_y(0.0),
        (false, false) => moved,
    };
    let toward = lead(&motion, &gear, &intent, &step.view);
    let body = &mut *motion;
    body.yaw = body.turn.step(body.yaw, toward, step.dt);
    transform.translation = out.position;
    transform.rotation = facing(up(&env.anchor, out.position), motion.yaw);
}

/// Registers the controller: the local intents are sent after the mouse is read, then every
/// body takes in its intents and moves, before the camera rig runs. The intents are sent and
/// gathered on every frame, so the step before the body takes over is never stale.
pub(super) fn plugin(app: &mut App) {
    app.add_message::<IntentOf>()
        .configure_sets(
            Update,
            (PlayerSet::Intend, PlayerSet::Move)
                .chain()
                .after(ViewSet::Input)
                .before(ViewSet::Rig),
        )
        .add_systems(Update, send.in_set(PlayerSet::Intend))
        .add_systems(
            Update,
            (gather, drive.run_if(playing).run_if(on_foot))
                .chain()
                .in_set(PlayerSet::Move),
        );
}
