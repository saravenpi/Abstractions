//! The companion casts its shadow: every part of its shell is drawn into the shadow maps.

use bevy::ecs::system::RunSystemOnce;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::shapes::Shapes;
use super::spawn::{Stock, look};

/// Spawns one companion in a headless app and returns the app.
fn spawned() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin))
        .init_asset::<Mesh>()
        .init_asset::<Image>()
        .init_asset::<StandardMaterial>();
    let at = Transform::from_xyz(3.0, 1.0, -2.0);
    let spawn = move |mut commands: Commands, mut stock: Stock| look(&mut commands, &mut stock, at);
    app.world_mut().run_system_once(spawn).expect("a companion");
    app.update();
    app
}

/// The head and both hands, near and far, wear the opaque shell and none of them is kept out of
/// the shadow maps.
#[test]
fn the_companion_casts_its_shadow() {
    let mut app = spawned();
    let shell = app.world().resource::<Shapes>().shell.clone();
    let alpha = app
        .world()
        .resource::<Assets<StandardMaterial>>()
        .get(&shell)
        .map(|own| own.alpha_mode);
    assert_eq!(alpha, Some(AlphaMode::Opaque));
    let mut parts = app
        .world_mut()
        .query::<(&MeshMaterial3d<StandardMaterial>, Has<NotShadowCaster>)>();
    let worn: Vec<bool> = parts
        .iter(app.world())
        .filter(|(material, _)| material.0.id() == shell.id())
        .map(|(_, hidden)| hidden)
        .collect();
    assert!(worn.len() >= 6, "{} shell parts", worn.len());
    assert!(worn.iter().all(|hidden| !hidden), "{worn:?}");
}
