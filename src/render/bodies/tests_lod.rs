//! Detail tests: the finest surface gives a texel to every game pixel wherever the sky first draws
//! a body, and a body shows its surface from a few pixels on.

use super::light::{DETAIL, detail};
use super::lod::{AHEAD, KEEP, level, need, pixels_per_radian};
use crate::world::globe::far::{LEAVE, LIMIT, SIDES, handover, sky_from};

/// At the default field of view the finest surface gives a texel or more to every game pixel of a
/// body at [`LEAVE`] radii, the nearest the sky draws it, and every level is asked for before the
/// body needs it and kept a while after.
#[test]
fn the_finest_surface_gives_a_texel_to_every_pixel() {
    let ppr = pixels_per_radian(75f32.to_radians());
    let at_leave = need(ppr, 1.0, LEAVE as f32);
    assert!(at_leave <= SIDES[SIDES.len() - 1] as f32, "{at_leave}");
    assert!(at_leave > SIDES[1] as f32, "{at_leave}");
    assert_eq!(level(at_leave, 1.0), SIDES.len() - 1);
    for distance in [1.5, 2.0, 5.0, 20.0, 200.0, 2000.0] {
        let needed = need(ppr, 1.0, distance);
        let asked = level(needed, AHEAD);
        assert!(
            SIDES[asked] as f32 >= needed || asked == SIDES.len() - 1,
            "{distance}"
        );
        assert!(level(needed, KEEP) >= asked, "{distance}");
    }
    assert_eq!(level(need(ppr, 1.0, 2000.0), AHEAD), 0);
}

/// A body is a dot in its mean colour up to about a pixel, shows all of its surface from three
/// pixels on, never less as it grows, and all of it wherever the sky draws it.
#[test]
fn a_body_shows_its_surface_from_the_hand_over_inward() {
    assert_eq!((detail(0.0), detail(DETAIL.0)), (0.0, 0.0));
    assert_eq!((detail(DETAIL.1), detail(1.0)), (1.0, 1.0));
    let steps: Vec<f32> = (0..=40).map(|k| detail(k as f32 * 0.0005)).collect();
    assert!(steps.windows(2).all(|pair| pair[1] >= pair[0]));
    for radius in [3000.0, 6000.0, 40_000.0, 150_000.0] {
        let at = handover(radius);
        assert_eq!(detail((radius / at) as f32), 1.0, "{radius}");
        assert!((radius / at - 1.0 / LIMIT).abs() < 1.0e-12);
        assert_eq!(detail((radius / sky_from(radius)) as f32), 1.0, "{radius}");
        assert!((sky_from(radius) - LEAVE * radius).abs() < 1.0e-6 * radius);
    }
}
