//! Interface scale tests: the scale follows the window height from the 720 pixel reference.

use super::camera::ui_scale_for;

/// The scale is 0.75, 1, 1.5 and 3 at 540, 720, 1080 and 2160 rows, and never leaves 0.75 to 3.
#[test]
fn the_scale_follows_the_window_height() {
    assert_eq!(ui_scale_for(540.0), 0.75);
    assert_eq!(ui_scale_for(720.0), 1.0);
    assert_eq!(ui_scale_for(1080.0), 1.5);
    assert_eq!(ui_scale_for(2160.0), 3.0);
    assert_eq!(ui_scale_for(200.0), 0.75);
    assert_eq!(ui_scale_for(9000.0), 3.0);
}
