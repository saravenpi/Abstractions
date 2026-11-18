//! The compass in space: its home mark points at the home star from a ship elsewhere in the
//! home galaxy, slides to the middle of the bar when the view turns to it and waits at an end
//! of the bar when it is behind.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::rig::app;
use crate::menus::hud::compass::{Bar, Mark};
use crate::menus::hud::needle::WIDTH;
use crate::render::GameCamera;
use crate::space::{Anchor, Frame};
use crate::universe::{GLY, Universe};

/// Half the width in pixels of the room of a mark, which is centred on its place on the bar.
const HALF_ROOM: f32 = 48.0;

/// Turns the camera to look along `toward` with +Y up, runs three frames, and returns where
/// along the bar the home mark is, in pixels, when it is shown, with the words under the bar.
fn home_mark(app: &mut App, toward: Vec3) -> (Option<f32>, Vec<String>) {
    let pose = Transform::default().looking_to(toward, Vec3::Y);
    let mut cameras = app
        .world_mut()
        .query_filtered::<&mut GlobalTransform, With<GameCamera>>();
    for mut camera in cameras.iter_mut(app.world_mut()) {
        *camera = GlobalTransform::from(pose);
    }
    for _ in 0..3 {
        app.update();
    }
    let mut marks = app.world_mut().query::<(&Mark, &Node, &Children)>();
    let (_, node, children) = marks
        .iter(app.world())
        .find(|(mark, ..)| **mark == Mark::Home)
        .unwrap();
    let left = match (node.display, node.left) {
        (Display::Flex, Val::Px(left)) => Some(left + HALF_ROOM),
        _ => None,
    };
    let words = children
        .iter()
        .filter_map(|child| app.world().get::<Text>(child));
    (left, words.map(|text| text.0.clone()).collect())
}

/// One light-year east of the home star, in the frame of the home galaxy, the bar is shown
/// and the home mark gives one light-year: at the left end looking north, in the middle
/// looking west at the star, and at an end of the bar looking away from it.
#[test]
fn the_compass_points_home_from_space() {
    let mut app = app("home");
    let universe = Universe::new(7);
    let star = universe.star(universe.home().star).unwrap();
    app.insert_resource(Anchor {
        frame: Frame::Galaxy(star.id.galaxy),
        origin: star.pos + DVec3::X * GLY,
        rotation: DQuat::IDENTITY,
        flat: false,
    });
    let (north, words) = home_mark(&mut app, Vec3::NEG_Z);
    assert_eq!(north, Some(0.0));
    assert!(words.contains(&"1.00 gly".to_string()), "{words:?}");
    let mut bars = app.world_mut().query_filtered::<&Visibility, With<Bar>>();
    assert_eq!(bars.single(app.world()).unwrap(), Visibility::Inherited);
    let (west, _) = home_mark(&mut app, Vec3::NEG_X);
    assert!((west.unwrap() - 0.5 * WIDTH).abs() < 0.01, "{west:?}");
    let (east, _) = home_mark(&mut app, Vec3::X);
    assert!(
        matches!(east, Some(at) if at == 0.0 || at == WIDTH),
        "{east:?}"
    );
}
