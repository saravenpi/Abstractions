//! The fall of the robot: a body whose head or torso is gone is destroyed. The robot wakes in a
//! spare body at the home base, whole again, and the gear it carried waits where it fell until it
//! comes back for it.

mod cache;
mod health;
mod show;
#[cfg(test)]
mod tests;

pub use cache::{Cache, Left};

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Pack;
use crate::avatar::hurt::Scars;
use crate::condition::Condition;
use crate::home::Base;
use crate::player::{Player, place};
use crate::screens::playing;
use crate::space::{Anchor, Frame};

/// Run condition: the head or the torso of the robot is gone.
fn destroyed(condition: Res<Condition>) -> bool {
    condition.ruined()
}

/// Puts everything `pack` holds into the cache `left` at `at`, in the frame of `body`, with what it
/// already held. The tools that are out in the world stay where they are, and stay the robot's.
pub fn leave(left: &mut Left, pack: &mut Pack, (body, at): (crate::universe::BodyId, DVec3)) {
    let mut cache = left.0.take().unwrap_or(Cache {
        body,
        at: at.to_array(),
        pack: Pack::default(),
    });
    let mut kit = std::mem::take(pack);
    (pack.out, pack.next) = (std::mem::take(&mut kit.out), kit.next);
    for stack in kit.stacks {
        cache.pack.stow(stack, f32::MAX);
    }
    for item in kit.items {
        cache.pack.keep(item.recipe, item.materials);
    }
    left.0 = Some(cache);
}

/// Replaces a destroyed robot with a spare one: the gear it carried goes into the cache where it
/// fell, every part is whole again, and the body stands at the wake point of the base when the
/// planet it fell on is the one the base stands on.
pub(crate) fn wake(
    mut commands: Commands,
    (mut condition, mut pack, mut left): (ResMut<Condition>, ResMut<Pack>, ResMut<Left>),
    (anchor, base): (Res<Anchor>, Option<Res<Base>>),
    player: Query<&Transform, With<Player>>,
) {
    if let (Ok(body), Frame::Surface(planet)) = (player.single(), anchor.frame) {
        leave(
            &mut left,
            &mut pack,
            (planet, anchor.to_frame(body.translation)),
        );
    }
    *condition = Condition::default();
    let Some(base) = base.filter(|base| anchor.frame == Frame::Surface(base.body)) else {
        return;
    };
    let feet = anchor.to_render(base.wake);
    let toward = (anchor.rotation * base.look()).as_vec3();
    commands.queue(move |world: &mut World| place(world, feet, toward));
}

/// Registers the fall, the cache and the way back to it.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Left>().add_systems(
        Update,
        (
            (cache::load, health::load),
            cache::store.run_if(resource_changed::<Left>),
            health::store.run_if(resource_changed::<Condition>),
            wake.run_if(destroyed).run_if(playing),
            show::show.run_if(show::wanted.and_then(resource_exists::<Scars>)),
            show::recover.run_if(any_with_component::<show::Waiting>),
        )
            .chain(),
    );
}
