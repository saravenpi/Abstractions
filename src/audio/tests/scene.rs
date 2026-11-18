//! The scene helpers: how altitude, space and clouds turn into levels.

use crate::audio::scene::{AudioScene, altitude_fade};

/// The altitude fade is 1 on the ground, 0 at the top of the air, and falls all the way up.
#[test]
fn altitude_fade_follows_the_air() {
    assert_eq!(altitude_fade(0.0, 4000.0), 1.0);
    assert_eq!(altitude_fade(-30.0, 4000.0), 1.0);
    assert_eq!(altitude_fade(4000.0, 4000.0), 0.0);
    assert_eq!(altitude_fade(9000.0, 4000.0), 0.0);
    let steps: Vec<f32> = (0..=40)
        .map(|i| altitude_fade(i as f32 * 100.0, 4000.0))
        .collect();
    assert!(steps.windows(2).all(|pair| pair[1] < pair[0]));
}

/// The music follows the altitude fade and is silent in space and in warp.
#[test]
fn music_level_follows_the_scene() {
    let scene = AudioScene {
        altitude_fade: 0.4,
        ..AudioScene::default()
    };
    assert_eq!(AudioScene::default().music_level(), 1.0);
    assert_eq!(scene.music_level(), 0.4);
    let space = AudioScene {
        in_space: true,
        ..scene
    };
    let warp = AudioScene {
        warping: true,
        ..scene
    };
    assert_eq!(space.music_level(), 0.0);
    assert_eq!(warp.music_level(), 0.0);
}

