//! Frame upkeep: copies `Placed` poses into transforms and rebases the anchor on foot.

use avian3d::interpolation::TransformEasingSystems;
use bevy::prelude::*;
use bevy::transform::TransformSystems;

use super::rebase::rebase_on_foot;
use super::{Anchor, Location, Placed, Travel};

/// When the frame systems run.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameSet {
    /// In `FixedFirst`, before the physics step of the same tick: the on-foot rebase, then the
    /// copy of every [`Placed`] pose, so that colliders and free bodies change frame together.
    Rebase,
    /// In `PostUpdate`, before transform propagation: the copy of [`Placed`] poses that changed
    /// and of all of them when the [`Anchor`] changed during the frame.
    Sync,
}

/// Run condition: a world exists and the player walks or swims.
pub fn on_foot(location: Option<Res<Location>>) -> bool {
    location.is_some_and(|location| location.travel == Travel::OnFoot)
}

/// Copies [`Placed`] poses into transforms through the anchor: all of them when the anchor
/// differs from the one this system last saw, else only the poses that changed.
pub(super) fn sync_placed(
    anchor: Option<Res<Anchor>>,
    mut seen: Local<Option<Anchor>>,
    mut placed: Query<(Ref<Placed>, &mut Transform)>,
) {
    let Some(anchor) = anchor else {
        return;
    };
    let moved = *seen != Some(*anchor);
    *seen = Some(*anchor);
    for (pose, mut transform) in &mut placed {
        if moved || pose.is_changed() {
            let next = Transform {
                translation: anchor.to_render(pose.pos),
                rotation: anchor.rotation_to_render(pose.rot),
                scale: transform.scale,
            };
            transform.set_if_neq(next);
        }
    }
}

/// Registers the frame systems.
pub(super) fn plugin(app: &mut App) {
    let rebase = FrameSet::Rebase
        .after(TransformEasingSystems::Reset)
        .before(TransformEasingSystems::UpdateStart);
    app.configure_sets(FixedFirst, rebase)
        .configure_sets(
            PostUpdate,
            FrameSet::Sync.before(TransformSystems::Propagate),
        )
        .add_systems(
            FixedFirst,
            (rebase_on_foot.run_if(on_foot), sync_placed)
                .chain()
                .in_set(FrameSet::Rebase),
        )
        .add_systems(PostUpdate, sync_placed.in_set(FrameSet::Sync));
}
