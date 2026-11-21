//! The compass away from the home planet: its home mark is not shown in space nor on another
//! planet.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::rig::app;
use crate::menus::MarkerKind;
use crate::menus::hud::compass::Pin;
use crate::render::GameCamera;
use crate::space::{Anchor, Frame};
use crate::universe::{BodyId, LY, Universe};

/// Half the width in pixels of the room of a mark, which is centred on its place on the bar.
const HALF_ROOM: f32 = 16.0;

/// Turns the camera to look along `toward` with +Y up, runs three frames, and returns where
/// along the bar the home mark is, in pixels, when it is shown, with the words under the bar.
fn home_mark(app: &mut App, toward: Vec3) -> (Option<f32>, Vec<String>) {
    let pose = Transform::default().looking_to(toward, Vec3::Y);
    let mut cameras = app
        .world_mut()
        .query_filtered::<&mut Transform, With<GameCamera>>();
    for mut camera in cameras.iter_mut(app.world_mut()) {
        *camera = pose;
    }
    for _ in 0..3 {
        app.update();
    }
    let mut marks = app.world_mut().query::<(&Pin, &Node, &Children)>();
    let home = marks
        .iter(app.world())
        .find(|(pin, ..)| pin.kind == Some(MarkerKind::Home));
    let Some((_, node, children)) = home else {
        return (None, Vec::new());
    };
    let left = match (node.display, node.left) {
        (Display::Flex, Val::Px(left)) => Some(left + HALF_ROOM),
        _ => None,
    };
    let words = children
        .iter()
        .filter_map(|child| app.world().get::<Text>(child));
    (left, words.map(|text| text.0.clone()).collect())
}

/// Looks every way and returns whether the home mark showed in any direction.
fn ever_shown(app: &mut App) -> bool {
    [Vec3::NEG_X, Vec3::X, Vec3::NEG_Z, Vec3::Z, Vec3::Y]
        .into_iter()
        .any(|toward| home_mark(app, toward).0.is_some())
}

/// One light-year east of the home star, in the frame of the home galaxy, the compass has no
/// home mark in any direction, even looking straight at the star.
#[test]
fn the_compass_hides_home_in_space() {
    let mut app = app("home");
    let universe = Universe::new(7);
    let star = universe.star(universe.home().star).unwrap();
    app.insert_resource(Anchor {
        frame: Frame::Galaxy(star.id.galaxy),
        origin: star.pos + DVec3::X * LY,
        rotation: DQuat::IDENTITY,
        flat: false,
    });
    assert!(!ever_shown(&mut app));
}

/// On the surface of another planet of the same star the compass has no home mark either.
#[test]
fn the_compass_hides_home_on_another_planet() {
    let mut app = app("home");
    let home = Universe::new(7).home();
    let other = BodyId {
        index: home.index + 1,
        ..home
    };
    app.insert_resource(Anchor {
        frame: Frame::Surface(other),
        origin: DVec3::X * 6000.0,
        rotation: DQuat::IDENTITY,
        flat: false,
    });
    assert!(!ever_shown(&mut app));
}
