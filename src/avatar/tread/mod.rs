//! The feet of every robot on the ground, heard: each time a foot of the gait plants, the body
//! lands from a jump, a fall or a glide, or slides along the ground, a footstep cue goes to the
//! audio with what the foot meets, how hard and how heavy, and where it is for the listener. The
//! body of the player at this machine is heard as the listener's own; the bodies of the other
//! players are heard from where they are.

pub(crate) mod contact;
pub(crate) mod ground;
#[cfg(test)]
pub(crate) mod stage;
mod tune;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_ground;

pub use contact::Footfall;

use bevy::ecs::message::Messages;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::AvatarSet;
use super::animate::Anim;
use super::own::Own;
use crate::audio::{AudioCue, Step};
use crate::player::{BODY_LENGTH, BODY_RADIUS};
use crate::render::GameCamera;
use crate::screens::shown;
use crate::space::{Anchor, Location, up};
use crate::world::Terrain;
use ground::Site;

/// The ships a robot can stand on: the centre of each hull in frame metres and the reach of its
/// airframe from that centre. The ship writes it every frame.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct Hulls(pub Vec<(DVec3, f32)>);

/// Returns `step` made by a body whose root is at render position `body`, on the ground under
/// it and placed for the listener `ear`: the listener's own when `own`. Nothing when the foot is
/// on no ground, or when there is no listener to place it for.
fn heard(
    site: &Site,
    step: Step,
    body: Vec3,
    (ear, own): (Option<&GlobalTransform>, bool),
) -> Option<Step> {
    let feet = body - up(&site.anchor, body) * (0.5 * BODY_LENGTH + BODY_RADIUS);
    let (surface, depth) = ground::footing(site, feet)?;
    let air = crate::wind::thickness(site.terrain.globe().record()).min(1.0) as f32;
    let (far, side) = match (own, ear) {
        (true, _) => (0.0, 0.0),
        (false, Some(ear)) => {
            let to = feet - ear.translation();
            (
                to.length().max(0.01),
                to.normalize_or_zero().dot(ear.right().as_vec3()),
            )
        }
        (false, None) => return None,
    };
    Some(Step {
        surface,
        depth,
        air,
        far,
        side,
        ..step
    })
}

/// Sends a footstep cue for every contact of every body with the ground this frame.
fn fire(
    site: Site,
    ears: Query<&GlobalTransform, With<GameCamera>>,
    mut bodies: Query<(Entity, &Anim, &GlobalTransform, &mut Footfall)>,
    own: Own,
    mut cues: MessageWriter<AudioCue>,
) {
    let (dt, gravity) = (
        site.time.delta_secs(),
        site.terrain.globe().gravity() as f32,
    );
    let (mine, ear) = (own.body().map(|(_, body)| body), ears.single().ok());
    for (entity, anim, body, mut feet) in &mut bodies {
        for step in contact::contacts(&mut feet, anim, gravity, dt)
            .into_iter()
            .flatten()
        {
            let placed = (ear, mine == Some(entity));
            if let Some(step) = heard(&site, step, body.translation(), placed) {
                cues.write(AudioCue::Footstep(step));
            }
        }
    }
}

/// Registers the feet, which listen once the bodies are posed, wherever there is ground and an
/// audio to tell.
pub(super) fn plugin(app: &mut App) {
    let world = resource_exists::<Terrain>
        .and_then(resource_exists::<Anchor>)
        .and_then(resource_exists::<Location>)
        .and_then(resource_exists::<Messages<AudioCue>>);
    app.init_resource::<Hulls>().add_systems(
        Update,
        fire.after(AvatarSet::Animate).run_if(shown).run_if(world),
    );
}
