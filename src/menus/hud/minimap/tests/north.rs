//! The north mark of the compass: on a planet an N sits at the bearing of the north pole, in
//! the middle of the bar while the player looks north and to the left while they look east of
//! north, and it hides while the player looks away from it.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::rig::{app, ground, look, settle};
use crate::menus::hud::compass::North;

/// Returns the place in pixels along the bar of the north mark, or nothing while it is hidden.
fn north(app: &mut App) -> Option<f32> {
    let mut marks = app.world_mut().query_filtered::<&Node, With<North>>();
    let node = marks.single(app.world()).expect("one north mark");
    match (node.display, node.left) {
        (Display::None, _) => None,
        (_, Val::Px(left)) => Some(left),
        other => panic!("placed with {other:?}"),
    }
}

/// Looking north puts the N in the middle of the bar, looking north-east puts it to the left,
/// and looking south hides it.
#[test]
fn the_compass_marks_north() {
    let mut app = app("north");
    let start = ground(0.0, 0.0);
    look(&mut app, start, DVec3::Y);
    settle(&mut app);
    let ahead = north(&mut app).expect("north shows ahead");
    look(&mut app, start, DVec3::new(0.0, 1.0, -1.0));
    settle(&mut app);
    let left = north(&mut app).expect("north shows to the left");
    assert!(left < ahead - 50.0, "{left} {ahead}");
    look(&mut app, start, DVec3::NEG_Y);
    settle(&mut app);
    assert_eq!(north(&mut app), None);
}
