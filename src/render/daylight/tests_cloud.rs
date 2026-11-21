//! Tests of the shadow of the clouds at the camera: only the cloud above the camera dims its sun,
//! so climbing through a cloud and over it brightens the view and never darkens it.

use super::track::sun_share;

/// Under the band the camera takes the whole shadow of the cloud, over it none, and the share
/// of the sun only grows on the way up.
#[test]
fn only_the_cloud_above_shades_the_camera() {
    let band = (1100.0, 3000.0);
    assert_eq!(sun_share(0.2, 500.0, band), 0.2);
    assert_eq!(sun_share(0.2, 3500.0, band), 1.0);
    let mut last = 0.0;
    for metre in (0..4000).step_by(50) {
        let now = sun_share(0.2, metre as f32, band);
        assert!(now >= last && now >= 0.2, "{metre} m: {now}");
        last = now;
    }
    assert_eq!(sun_share(0.3, 2000.0, (0.0, 0.0)), 0.3);
}
