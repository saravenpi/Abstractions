//! The compass reads the markers: marks fade toward the ends of the bar and hide past them,
//! marks that would cover each other stack in rows, and a rebase in the frame keeps both the
//! compass and the minimap on the true place of the player.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::read::{at, furnish};
use super::rig::{app, ground, look, settle};
use crate::menus::hud::compass::Pin;
use crate::menus::hud::minimap::scope::Scope;
use crate::menus::hud::minimap::widget::Part;
use crate::menus::hud::needle::{ROWS, SPAN, fade, row};
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};
use crate::render::GameCamera;
use crate::space::Anchor;

/// A mark is fully opaque over the middle 70% of the span, fades to nothing at its ends and
/// is hidden past them, behind the player included.
#[test]
fn marks_fade_toward_the_ends_and_hide_behind() {
    assert_eq!(fade(0.0), Some(1.0));
    assert_eq!(fade(-0.7 * SPAN), Some(1.0));
    assert!((fade(0.85 * SPAN).unwrap() - 0.5).abs() < 1e-5);
    assert!(fade(SPAN).unwrap().abs() < 1e-6);
    assert_eq!(fade(1.01 * SPAN), None);
    assert_eq!(fade(std::f32::consts::PI), None);
}

/// Five marks at one place take five rows; a mark far enough to the right goes back to the
/// first row.
#[test]
fn five_overlapping_marks_stack_in_rows() {
    let mut ends = [f32::NEG_INFINITY; ROWS];
    let rows: Vec<usize> = (0..5).map(|_| row(100.0, &mut ends)).collect();
    assert_eq!(rows, vec![0, 1, 2, 3, 4]);
    assert_eq!(row(300.0, &mut ends), 0);
}

/// Returns a friend called `name` at the ground point `at` of the home planet.
fn friend(app: &App, name: &str, at: DVec3) -> Marker {
    let body = app.world().resource::<crate::universe::Universe>().home();
    Marker {
        kind: MarkerKind::Friend,
        place: MarkerPlace::Body(body, at),
        color: Color::srgb(0.9, 0.3, 0.3),
        label: name.to_string(),
        owner: 7,
    }
}

/// Returns the place and row of every shown mark of `kind`, sorted by row.
fn shown(app: &mut App, kind: MarkerKind) -> Vec<(f32, f32)> {
    let mut pins = app.world_mut().query::<(&Pin, &Node)>();
    let mut found: Vec<(f32, f32)> = pins
        .iter(app.world())
        .filter(|(pin, node)| pin.kind == Some(kind) && node.display == Display::Flex)
        .map(|(_, node)| match (node.left, node.top) {
            (Val::Px(left), Val::Px(top)) => (left, top),
            other => panic!("placed with {other:?}"),
        })
        .collect();
    found.sort_by(|a, b| a.1.total_cmp(&b.1));
    found
}

/// Five friends 300 m ahead show as five marks stacked in five rows over the same place, and
/// a sixth one behind the player shows nowhere.
#[test]
fn the_compass_shows_every_friend_ahead() {
    let mut app = app("friends");
    let ahead = ground(300.0, 0.0);
    let mut marks: Vec<Marker> = (0..5)
        .map(|i| friend(&app, &format!("f{i}"), ahead))
        .collect();
    marks.push(friend(&app, "behind", ground(-300.0, 0.0)));
    app.world_mut().resource_mut::<Markers>().0.extend(marks);
    for _ in 0..4 {
        app.update();
    }
    let found = shown(&mut app, MarkerKind::Friend);
    assert_eq!(found.len(), 5, "{found:?}");
    assert!(
        found
            .iter()
            .all(|(left, _)| (left - found[0].0).abs() < 0.01)
    );
    let tops: Vec<f32> = found.iter().map(|(_, top)| *top).collect();
    assert!(tops.windows(2).all(|pair| pair[1] > pair[0]), "{tops:?}");
}

/// Moves the anchor 500 m along the planet and the camera with it in one frame, leaving a
/// stale `GlobalTransform` from the anchor before, then runs that one frame.
fn rebase(app: &mut App, at: DVec3, toward: DVec3) {
    let old = *app.world().resource::<Anchor>();
    let new = Anchor {
        origin: old.origin + DVec3::NEG_Z * 500.0,
        ..old
    };
    let turned = |along: DVec3| (new.rotation * along).as_vec3();
    let pose = Transform::from_translation(new.to_render(at));
    let pose = pose.looking_to(turned(toward), turned(at.normalize()));
    let stale = GlobalTransform::from(Transform::from_translation(old.to_render(at)));
    app.insert_resource(new);
    let mut cameras = app.world_mut().query_filtered::<Entity, With<GameCamera>>();
    let camera = cameras.single(app.world()).unwrap();
    app.world_mut().entity_mut(camera).insert((pose, stale));
    app.update();
}

/// Returns the place in pixels along the bar of the home mark.
fn home_left(app: &mut App) -> f32 {
    shown(app, MarkerKind::Home)[0].0
}

/// A rebase in the frame keeps the centre of the minimap on the true place of the player
/// within 0.1 m, and the home marks of the compass and the minimap where they were.
#[test]
fn a_rebase_in_the_frame_moves_no_mark() {
    let mut app = app("rebase");
    furnish(&mut app);
    let (start, toward) = (ground(0.0, 0.0), DVec3::new(0.0, 1.0, -1.0));
    look(&mut app, start, toward);
    settle(&mut app);
    let (compass, minimap) = (home_left(&mut app), at(&mut app, Part::Home));
    rebase(&mut app, start, toward);
    let centre = app.world().resource::<Scope>().last.unwrap().1;
    assert!(
        (centre - start).length() < 0.1,
        "{}",
        (centre - start).length()
    );
    assert!((home_left(&mut app) - compass).abs() < 0.5);
    assert_eq!(at(&mut app, Part::Home), minimap);
}
