//! The body casts its shadow: no segment of its shell is kept out of the shadow maps and the shell
//! is opaque, so the shadow pass draws it, standing, walking and seated.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::app::{Forced, world};
use super::scene;
use crate::avatar::Mode;
use crate::avatar::animate::Bone;

/// Asserts that every segment of the body casts a shadow: none carries [`NotShadowCaster`] and
/// each wears an opaque material, which the shadow pass draws.
fn assert_casts(app: &mut App, what: &str) {
    let mut bones = app.world_mut().query_filtered::<(
        Entity,
        &MeshMaterial3d<StandardMaterial>,
        Has<NotShadowCaster>,
    ), With<Bone>>();
    let shells: Vec<_> = bones
        .iter(app.world())
        .map(|(entity, material, hidden)| (entity, material.0.clone(), hidden))
        .collect();
    assert!(!shells.is_empty(), "{what}: no segment");
    let materials = app.world().resource::<Assets<StandardMaterial>>();
    for (entity, material, hidden) in shells {
        assert!(!hidden, "{what}: {entity} casts no shadow");
        let alpha = materials.get(&material).map(|own| own.alpha_mode);
        assert!(
            matches!(alpha, Some(AlphaMode::Opaque | AlphaMode::Mask(_))),
            "{what}: {entity} is drawn {alpha:?}, which casts no shadow"
        );
    }
}

/// Standing, walking and seated, every segment of the body casts its shadow.
#[test]
fn the_body_casts_its_shadow_in_every_mode() {
    let (mut app, _) = world();
    for mode in [Mode::Idle, Mode::Walk, Mode::Seated, Mode::Idle] {
        app.insert_resource(Forced(scene(mode, 0.5)));
        app.update();
        app.update();
        assert_casts(&mut app, &format!("{mode:?}"));
    }
}
