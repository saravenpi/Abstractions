//! Tests of the season the texture takes for a camera: in full near the ground, the mean
//! colours of the planet from the air and from orbit, and the snow of the ground, not of the air.

use std::f32::consts::FRAC_PI_4;

use super::season::SeasonLook;
use super::seasons::look_over;
use super::tests_season::{home, spot};
use crate::climate::Setting;

/// From orbit over a frozen pole the planet keeps its mean colours: the snow of the one place
/// under the camera shows in full near the ground there and not at all from a few hundred
/// metres up, so it never whitens the whole disc.
#[test]
fn the_season_under_the_camera_fades_with_height() {
    let mut setting = Setting::of(&home());
    setting.temperature = 250.0;
    let pole = spot(1.55, 40.0);
    assert_eq!(SeasonLook::at(&setting, &pole, 0.0).snow, 1.0);
    assert_eq!(look_over(&setting, &pole, 42.0, 0.0).snow, 1.0);
    assert_eq!(look_over(&setting, &pole, 150.0, 0.0).snow, 1.0);
    for altitude in [640.0, 5000.0, 2.0e5, 4.0e7] {
        let look = look_over(&setting, &pole, altitude, 0.0);
        assert_eq!(look, SeasonLook::NONE, "{altitude}");
    }
    let fall = spot(FRAC_PI_4, 0.0);
    let heights = (0..=70).map(|i| i as f32 * 10.0);
    let looks: Vec<SeasonLook> = heights
        .map(|h| look_over(&setting, &fall, h, 0.4))
        .collect();
    assert!(looks[0].autumn > 0.9 && looks[70] == SeasonLook::NONE);
    assert!(
        looks
            .windows(2)
            .all(|pair| pair[1].autumn <= pair[0].autumn)
    );
    assert!(
        looks
            .windows(2)
            .all(|pair| pair[0].autumn - pair[1].autumn < 0.05)
    );
}

/// The snow is that of the ground under the camera, not of the cold air the camera is in: low
/// over warm ground nothing whitens, and low over a frozen plateau the snow is the plateau's.
#[test]
fn snow_is_read_at_the_ground_not_at_the_camera() {
    let setting = Setting::of(&home());
    let (warm, high) = (spot(0.5, 5.0), spot(0.5, 4000.0));
    assert_eq!(SeasonLook::at(&setting, &high, 0.75).snow, 1.0);
    for above in [0.0, 60.0, 149.0] {
        assert_eq!(look_over(&setting, &warm, 5.0 + above, 0.75).snow, 0.0);
        assert_eq!(look_over(&setting, &high, 4000.0 + above, 0.75).snow, 1.0);
    }
    assert_eq!(look_over(&setting, &warm, 4000.0, 0.75), SeasonLook::NONE);
}
