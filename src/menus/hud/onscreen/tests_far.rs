//! Tests of the far limit of one marker over the world: a pin nobody follows fades out past two
//! minutes of travel and comes back nearer than it left, while the course, the followed pin,
//! friends, home and the ship show at any distance.

use super::rule::{Sticky, look, reach};
use super::tests_rule::marker;
use super::tune::{BAND, FAR_FOOT, FAR_GLIDE, FAR_SHIP, STICK};
use crate::menus::MarkerKind;
use crate::menus::hud::context::Context;

/// The limit of a pin nobody follows grows with how the player travels: 1 km on foot and
/// swimming, 5 km gliding, 15 km in the ship, landed or flying.
#[test]
fn the_far_limit_follows_how_the_player_travels() {
    let pin = marker(MarkerKind::Favourite, 0);
    assert_eq!(reach(&pin, Context::OnFoot), Some(FAR_FOOT));
    assert_eq!(reach(&pin, Context::Swimming), Some(FAR_FOOT));
    assert_eq!(reach(&pin, Context::Gliding), Some(FAR_GLIDE));
    assert_eq!(reach(&pin, Context::ShipLanded), Some(FAR_SHIP));
    assert_eq!(reach(&pin, Context::ShipAir), Some(FAR_SHIP));
    assert!(FAR_FOOT < FAR_GLIDE && FAR_GLIDE < FAR_SHIP);
    let theirs = marker(MarkerKind::Waypoint, 7);
    assert_eq!(reach(&theirs, Context::OnFoot), Some(FAR_FOOT));
}

/// What the player follows, friends, home and the ship have no far limit, and the followed pin
/// is whole 50 km away on foot.
#[test]
fn a_tracked_pin_far_away_keeps_its_marker() {
    let kept = [
        marker(MarkerKind::Course, 0),
        marker(MarkerKind::Waypoint, 0),
        marker(MarkerKind::Friend, 3),
        marker(MarkerKind::Home, 0),
        marker(MarkerKind::Ship, 0),
    ];
    for context in Context::ALL {
        assert!(kept.iter().all(|mark| reach(mark, context).is_none()));
    }
    let followed = look(&kept[1], 5.0e4, Context::OnFoot, &mut Sticky::default());
    assert_eq!((followed.alpha, followed.detail), (1.0, 1.0));
}

/// Walking away from a pin on foot, its marker is whole up to 1 km, fades over the next
/// quarter without a jump and is gone past 1.25 km.
#[test]
fn a_pin_fades_out_past_its_far_limit() {
    let (pin, mut sticky) = (marker(MarkerKind::Favourite, 0), Sticky::default());
    let mut last = 1.0;
    for step in 0..=400 {
        let gap = 900.0 + f64::from(step);
        let seen = look(&pin, gap, Context::OnFoot, &mut sticky);
        assert_eq!(seen.alpha >= 1.0, gap <= FAR_FOOT, "{gap}");
        assert_eq!(seen.alpha <= 0.0, gap >= FAR_FOOT * BAND, "{gap}");
        assert!(
            seen.alpha <= last && last - seen.alpha < 0.05,
            "a jump at {gap} m"
        );
        last = seen.alpha;
    }
    assert!(sticky.far);
}

/// Walking back, the marker comes back only a fifth nearer than it left, so a player on the
/// limit sees no flicker.
#[test]
fn a_pin_comes_back_nearer_than_it_left() {
    let (pin, mut sticky) = (marker(MarkerKind::Favourite, 0), Sticky::default());
    look(&pin, 2000.0, Context::OnFoot, &mut sticky);
    let back = FAR_FOOT * BAND / STICK;
    for gap in [1300.0, 1249.0, back + 1.0, 1100.0] {
        assert_eq!(
            look(&pin, gap, Context::OnFoot, &mut sticky).alpha,
            0.0,
            "{gap}"
        );
    }
    let between = look(&pin, back - 50.0, Context::OnFoot, &mut sticky).alpha;
    assert!(between > 0.0 && between < 1.0, "{between}");
    let whole = look(&pin, FAR_FOOT / STICK - 1.0, Context::OnFoot, &mut sticky).alpha;
    assert_eq!(whole, 1.0);
    assert!(!sticky.far);
}
