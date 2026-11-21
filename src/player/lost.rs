//! The safety net of a lost body. A body that fell out of the world or sank under the ground is
//! put back once, onto a collider of the world. Where no collider is loaded yet it is held still
//! at the ground the planet gives until one is, so it never falls through and comes back again
//! and again.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::controller::Motion;
use super::floor::Floor;
use super::spawn::{Home, rescue};
use super::{BODY_LENGTH, BODY_RADIUS, Player, PlayerSet};
use crate::gear::Gear;
use crate::sim::PlayerId;
use crate::space::{Anchor, on_foot, up};
use crate::view::{Cut, ViewSet};
use crate::world::Terrain;

/// Gap in metres left between the feet and the collider a rescued body is stood on.
const LIFT: f32 = 0.05;

/// Seconds after a rescue within which another one counts as a repeat.
const AGAIN: f64 = 10.0;

/// A body held still until a collider is loaded under it: where it is held, in the frame of the
/// anchor.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Stranded(pub DVec3);

/// How often a body was rescued: every rescue, and those that came within [`AGAIN`] seconds of
/// the one before, which should never happen.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Rescues {
    /// Rescues so far.
    pub count: u32,
    /// Rescues that came within [`AGAIN`] seconds of the one before.
    pub repeats: u32,
    /// Seconds of game time at the last rescue.
    last: Option<f64>,
}

impl Rescues {
    /// Counts a rescue at `now` seconds, and a repeat when the last one was less than [`AGAIN`]
    /// seconds before. A dev build logs a repeat.
    pub fn note(&mut self, now: f64) {
        let repeat = self.last.is_some_and(|last| now - last < AGAIN);
        self.count += 1;
        self.repeats += u32::from(repeat);
        #[cfg(feature = "dev")]
        if repeat {
            warn!(
                "the body was rescued again within {AGAIN} s: {} repeats",
                self.repeats
            );
        }
        self.last = Some(now);
    }
}

/// The parts of a player body that are put back when lost.
type Lost = (
    Entity,
    &'static PlayerId,
    &'static mut Transform,
    &'static mut Motion,
    &'static mut Gear,
    &'static Home,
    Option<&'static Stranded>,
);

/// What a rescue reads of the world.
type Ground<'w> = (Res<'w, Anchor>, Res<'w, Terrain>, Res<'w, Time>);

/// Returns the render position of the centre of a body stood on the collider under render
/// position `back`, or nothing when no collider is loaded there.
fn stand(floor: &Floor, anchor: &Anchor, back: Vec3) -> Option<Vec3> {
    let vertical = up(anchor, back);
    let soles = BODY_LENGTH * 0.5 + BODY_RADIUS + LIFT;
    floor
        .under(back, vertical)
        .map(|feet| feet + vertical * soles)
}

/// Marks `entity` as held at `left`, or no longer held when there is nothing.
fn strand(commands: &mut Commands, entity: Entity, left: Option<Stranded>) {
    match left {
        Some(held) => {
            commands.entity(entity).insert(held);
        }
        None => {
            commands.entity(entity).remove::<Stranded>();
        }
    }
}

/// Puts every lost player back: once, at rest, the flight gear folded, on the collider under the
/// place [`rescue`] gives, or held there until that collider loads. The camera of the local body
/// cuts when it is moved there and when it stands.
pub(super) fn recover(
    (anchor, terrain, time): Ground,
    floor: Floor,
    mut rescues: ResMut<Rescues>,
    mut player: Query<Lost, With<Player>>,
    (mut cuts, mut commands): (MessageWriter<Cut>, Commands),
) {
    for (entity, id, mut transform, mut motion, mut gear, home, stranded) in &mut player {
        let held = stranded.map(|held| anchor.to_render(held.0));
        let Some(back) = held.or_else(|| rescue(&anchor, &terrain, transform.translation, home.0))
        else {
            continue;
        };
        if stranded.is_none() {
            rescues.note(time.elapsed_secs_f64());
        }
        let stood = stand(&floor, &anchor, back);
        transform.translation = stood.unwrap_or(back);
        *motion = Motion {
            yaw: motion.yaw,
            ..default()
        };
        *gear = Gear::default();
        let left = stood.is_none().then(|| Stranded(anchor.to_frame(back)));
        if left.is_some() != stranded.is_some() {
            strand(&mut commands, entity, left);
        }
        if *id == PlayerId::LOCAL && (stood.is_some() || stranded.is_none()) {
            cuts.write(Cut::default());
        }
    }
}

/// Registers the rescue, after the controller moved the bodies and before the camera rig.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Rescues>().add_systems(
        Update,
        recover
            .run_if(on_foot)
            .after(PlayerSet::Move)
            .before(ViewSet::Rig),
    );
}
