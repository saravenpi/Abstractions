//! Tests of the backdrop of the menus: the blur follows the game image when it is made anew, so
//! a menu over the game never shows a still picture of another moment.

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;

use super::BlurMaterial;
use crate::render::LowRes;

/// How many times the blur material was built again.
#[derive(Resource, Default)]
struct Rebuilt(usize);

/// Counts the blur materials changed this frame.
fn count(mut events: MessageReader<AssetEvent<BlurMaterial>>, mut rebuilt: ResMut<Rebuilt>) {
    let changed = events
        .read()
        .filter(|event| matches!(event, AssetEvent::Modified { .. }));
    rebuilt.0 += changed.count();
}

/// Returns an app with the game image, the blur material and the backdrop systems, and no
/// renderer.
fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .init_asset::<BlurMaterial>()
        .init_resource::<LowRes>()
        .init_resource::<Rebuilt>()
        .add_plugins(super::plugin)
        .add_systems(Last, count);
    for _ in 0..3 {
        app.update();
    }
    app
}

/// The blur is built again once the game image is resized, as when the window changes size,
/// and not while it stays as it is.
#[test]
fn the_blur_follows_a_resized_game_image() {
    let mut app = app();
    assert_eq!(app.world().resource::<Rebuilt>().0, 0);
    let low = app.world().resource::<LowRes>().0.clone();
    let mut images = app.world_mut().resource_mut::<Assets<Image>>();
    let size = Extent3d {
        width: 640,
        height: 360,
        depth_or_array_layers: 1,
    };
    images.get_mut(&low).expect("the game image").resize(size);
    for _ in 0..4 {
        app.update();
    }
    assert!(app.world().resource::<Rebuilt>().0 >= 1);
}
