//! The mouse on a rolled screen: it turns the view toward the edge of the screen it moves to,
//! however far the camera is rolled.

use bevy::math::Vec2;

use crate::view::on_screen;

/// Unrolled, the mouse turns the view as it always did. Rolled a quarter turn to the right, a
/// move to the right edge of the screen turns the view down and a move up turns it right, and
/// rolled over, every move turns the other way.
#[test]
fn the_mouse_follows_the_rolled_screen() {
    let close = |a: Vec2, b: Vec2| a.distance(b) < 1.0e-5;
    let right = Vec2::new(-1.0, 0.0);
    let up = Vec2::new(0.0, 1.0);
    assert!(close(on_screen(right, 0.0, 0.0), right) && close(on_screen(up, 0.0, 0.0), up));
    let quarter = std::f32::consts::FRAC_PI_2;
    assert!(close(on_screen(right, quarter, 0.0), Vec2::new(0.0, -1.0)));
    assert!(close(on_screen(up, quarter, 0.0), Vec2::new(-1.0, 0.0)));
    let over = std::f32::consts::PI;
    assert!(close(on_screen(right, over, 0.0), -right) && close(on_screen(up, over, 0.0), -up));
}

/// A view that went over the top of a loop is pitched past the vertical and lies on its back:
/// its roof points against the up its heading turns round. There a move of the mouse to the
/// right edge of the screen still turns the picture to the right, which takes the opposite turn
/// of the heading, and a move up still pitches the same way. Rolled over on top of that, as
/// behind a body that came the right way up, both turns are reversed again.
#[test]
fn the_mouse_follows_the_screen_of_a_view_on_its_back() {
    let close = |a: Vec2, b: Vec2| a.distance(b) < 1.0e-5;
    let right = Vec2::new(-1.0, 0.0);
    let up = Vec2::new(0.0, 1.0);
    for pitch in [2.0, 3.1, -2.0, -3.1] {
        assert!(close(on_screen(right, 0.0, pitch), -right), "{pitch}");
        assert!(close(on_screen(up, 0.0, pitch), up), "{pitch}");
        let over = std::f32::consts::PI;
        assert!(close(on_screen(right, over, pitch), right), "{pitch}");
        assert!(close(on_screen(up, over, pitch), -up), "{pitch}");
    }
    for pitch in [0.0, 1.5, -1.5] {
        assert!(close(on_screen(right, 0.0, pitch), right), "{pitch}");
    }
}

/// Returns the direction a view at `heading`, a heading and a pitch, looks along.
fn ahead(heading: Vec2) -> bevy::math::Vec3 {
    let turn =
        bevy::math::Quat::from_rotation_y(heading.x) * bevy::math::Quat::from_rotation_x(heading.y);
    turn * bevy::math::Vec3::NEG_Z
}

/// Whatever the pitch of the view and the roll of the camera, a move of the mouse to the right
/// edge of the screen moves where the view looks toward the right of the camera, and a move up
/// toward its roof: the picture never turns against the hand.
#[test]
fn the_view_turns_toward_the_edge_the_mouse_moves_to() {
    use bevy::math::{Quat, Vec3};
    for pitch in [-3.0, -2.0, -1.0, 0.0, 0.4, 1.0, 2.0, 3.0] {
        for roll in [-3.0, -1.6, -0.5, 0.0, 0.5, 1.6, 3.0] {
            let at = Vec2::new(0.7, pitch);
            let camera = Quat::from_rotation_y(at.x)
                * Quat::from_rotation_x(at.y)
                * Quat::from_rotation_z(-roll);
            let step = 1.0e-3;
            let moved =
                |mouse: Vec2| (ahead(at + on_screen(mouse * step, roll, pitch)) - ahead(at)) / step;
            let (aside, above) = (moved(Vec2::new(-1.0, 0.0)), moved(Vec2::new(0.0, 1.0)));
            assert!(
                aside.dot(camera * Vec3::X) > 0.05,
                "pitch {pitch}, roll {roll}: {aside}"
            );
            assert!(
                above.dot(camera * Vec3::Y) > 0.05,
                "pitch {pitch}, roll {roll}: {above}"
            );
        }
    }
}
