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
    assert!(close(on_screen(right, 0.0), right) && close(on_screen(up, 0.0), up));
    let quarter = std::f32::consts::FRAC_PI_2;
    assert!(close(on_screen(right, quarter), Vec2::new(0.0, -1.0)));
    assert!(close(on_screen(up, quarter), Vec2::new(-1.0, 0.0)));
    let over = std::f32::consts::PI;
    assert!(close(on_screen(right, over), -right) && close(on_screen(up, over), -up));
}
