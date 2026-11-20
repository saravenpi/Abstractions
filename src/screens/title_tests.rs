//! Tests of the title picture: it covers windows of every shape without stretching.

use bevy::math::Vec2;

use super::title::cover;

/// A wide, a tall and a matching window are each covered whole, the picture keeps its shape,
/// it is centred across, and a tall window crops the sides while a wide one crops the top more
/// than the bottom.
#[test]
fn the_picture_covers_every_window() {
    let aspect = 2.0;
    for size in [
        Vec2::new(1920.0, 600.0),
        Vec2::new(800.0, 1000.0),
        Vec2::new(1000.0, 500.0),
    ] {
        let (shown, corner) = cover(size, aspect);
        assert!((shown.x / shown.y - aspect).abs() < 1.0e-4);
        assert!(corner.x <= 0.0 && corner.y <= 0.0);
        assert!(corner.x + shown.x >= size.x - 1.0e-3 && corner.y + shown.y >= size.y - 1.0e-3);
        assert!((corner.x * 2.0 + shown.x - size.x).abs() < 1.0e-3);
    }
    let (shown, corner) = cover(Vec2::new(1920.0, 600.0), aspect);
    let below = shown.y + corner.y - 600.0;
    assert!(-corner.y > below, "{corner} {shown}");
}
