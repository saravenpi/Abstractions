//! The feet of every robot on the ground, heard: each time a foot of the gait plants, the body
//! lands from a jump, a fall or a glide, or slides along the ground, a footstep cue goes to the
//! audio with what the foot meets and how hard and how heavy, as a sound from the foot. The body of
//! the player at this machine holds its own steps, which it also carries in a vacuum; the bodies of
//! the other players are heard from where they are.

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
use crate::audio::spatial::tune::STEP;
use crate::audio::{AudioCue, Origin, Sound, Step};
use crate::player::{BODY_LENGTH, BODY_RADIUS};
use crate::screens::shown;
use crate::space::{Anchor, Location, up};
use crate::world::Terrain;
use ground::Site;

/// The ships a robot can stand on: the centre of each hull in frame metres and the reach of its
/// airframe from that centre. The ship writes it every frame.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct Hulls(pub Vec<(DVec3, f32)>);

/// Returns `step` made by a body whose root is at render position `body`, on the ground under
/// it, as a sound from the foot: held by the listener's own body when `own`. Nothing when the
/// foot is on no ground.
fn heard(site: &Site, step: Step, body: Vec3, own: bool) -> Option<Sound> {
    let feet = body - up(&site.anchor, body) * (0.5 * BODY_LENGTH + BODY_RADIUS);
    let (surface, depth) = ground::footing(site, feet)?;
    let from = match own {
        true => Origin::Held(feet, STEP.0, STEP.1),
        false => Origin::At(feet, STEP.0),
    };
    let step = Step {
        surface,
        depth,
        ..step
    };
    Some(Sound {
        cue: AudioCue::Footstep(step),
        from,
    })
}

/// Sends a footstep sound for every contact of every body with the ground this frame.
fn fire(
    site: Site,
    mut bodies: Query<(Entity, &Anim, &GlobalTransform, &mut Footfall)>,
    own: Own,
    mut sounds: MessageWriter<Sound>,
) {
    let (dt, gravity) = (
        site.time.delta_secs(),
        site.terrain.globe().gravity() as f32,
    );
    let mine = own.body().map(|(_, body)| body);
    for (entity, anim, body, mut feet) in &mut bodies {
        for step in contact::contacts(&mut feet, anim, gravity, dt)
            .into_iter()
            .flatten()
        {
            if let Some(sound) = heard(&site, step, body.translation(), mine == Some(entity)) {
                sounds.write(sound);
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
        .and_then(resource_exists::<Messages<Sound>>);
    app.init_resource::<Hulls>().add_systems(
        Update,
        fire.after(AvatarSet::Animate).run_if(shown).run_if(world),
    );
}
