//! The compass reads the markers: marks fade toward the ends of the bar and hide past them,
//! every mark sits at its bearing with its distance under it, the one in front whole and the
//! others dimmed behind, and a rebase in the frame keeps both the compass and the minimap on
//! the true place of the player.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::read::{at, furnish};
use super::rig::{app, ground, look, settle};
use crate::menus::hud::compass::{BEHIND, Pin};
use crate::menus::hud::minimap::scope::Scope;
use crate::menus::hud::minimap::widget::Part;
use crate::menus::hud::needle::{SPAN, fade};
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};
use crate::render::GameCamera;
use crate::space::Anchor;
use crate::ui::theme;

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

/// What a shown mark of the compass draws: its place in pixels along the bar, the opacity of
/// its icon and the distance written under it.
type Drawn = (f32, f32, String);

/// Returns what every shown mark of `kind` draws, read through the nodes of the mark: its row
/// holds the icon then the count, and the line under it the distance.
fn drawn(app: &mut App, kind: MarkerKind) -> Vec<Drawn> {
    let mut pins = app.world_mut().query::<(&Pin, &Node, &Children)>();
    let world = app.world();
    pins.iter(world)
        .filter(|(pin, node, _)| pin.kind == Some(kind) && node.display == Display::Flex)
        .map(|(_, node, children)| {
            let Val::Px(left) = node.left else {
                panic!("placed with {:?}", node.left);
            };
            let head = world
                .get::<Children>(children[0])
                .expect("the row of the mark");
            let icon = world.get::<TextColor>(head[0]).expect("the icon").0.alpha();
            let far = world
                .get::<Text>(children[1])
                .expect("the distance")
                .0
                .clone();
            (left, icon, far)
        })
        .collect()
}

/// Eight friends 300 m ahead show as five icons at their one bearing, none moved aside: the
/// one in front whole with "300 m" under it, the four behind it at the reduced opacity and
/// without a distance, which would print over the front one's. The three past the limit fold
/// into the front one as "+3". A ninth one behind the player shows nowhere.
#[test]
fn the_compass_keeps_friends_ahead_at_their_bearing() {
    let mut app = app("friends");
    let ahead = ground(300.0, 0.0);
    let mut marks: Vec<Marker> = (0..8)
        .map(|i| friend(&app, &format!("f{i}"), ahead))
        .collect();
    marks.push(friend(&app, "behind", ground(-300.0, 0.0)));
    app.world_mut().resource_mut::<Markers>().0.extend(marks);
    for _ in 0..4 {
        app.update();
    }
    let found = drawn(&mut app, MarkerKind::Friend);
    assert_eq!(found.len(), 5, "{found:?}");
    assert!(found.iter().all(|mark| mark.0 == found[0].0), "{found:?}");
    let front: Vec<&Drawn> = found.iter().filter(|mark| mark.1 > 0.99).collect();
    assert_eq!(front.len(), 1, "{found:?}");
    assert_eq!(front[0].2, "300 m");
    let back: Vec<&Drawn> = found
        .iter()
        .filter(|mark| (mark.1 - BEHIND).abs() < 1e-3)
        .collect();
    assert_eq!(back.len(), 4, "{found:?}");
    assert!(back.iter().all(|mark| mark.2.is_empty()), "{found:?}");
    let mut words = app.world_mut().query::<&Text>();
    let spoken: Vec<&str> = words
        .iter(app.world())
        .map(|text| text.0.as_str())
        .filter(|words| words.starts_with('+'))
        .collect();
    assert_eq!(spoken, ["+3"]);
}

/// A mark is an icon with its distance on the line under it, in the secondary tone, and no
/// name anywhere: the count beside the icon stays empty unless friends fold into it.
#[test]
fn a_mark_holds_an_icon_and_its_distance_and_no_name() {
    let mut app = app("icons");
    let marks = vec![friend(&app, "Ada", ground(300.0, 0.0))];
    app.world_mut().resource_mut::<Markers>().0.extend(marks);
    for _ in 0..4 {
        app.update();
    }
    let found = drawn(&mut app, MarkerKind::Friend);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].2, "300 m");
    let mut pins = app.world_mut().query::<(&Pin, &Children)>();
    let world = app.world();
    let (_, children) = pins
        .iter(world)
        .find(|(pin, _)| pin.kind == Some(MarkerKind::Friend))
        .unwrap();
    let head = world.get::<Children>(children[0]).unwrap();
    let count = world.get::<Text>(head[1]).unwrap();
    assert!(count.0.is_empty(), "{count:?}");
    let tone = world.get::<TextColor>(children[1]).unwrap().0;
    assert!(
        (tone.alpha() - theme::TEXT_2.alpha()).abs() < 1e-4,
        "{tone:?}"
    );
    let mut texts = app.world_mut().query::<&Text>();
    assert!(texts.iter(app.world()).all(|text| !text.0.contains("Ada")));
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
    drawn(app, MarkerKind::Home)[0].0
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
