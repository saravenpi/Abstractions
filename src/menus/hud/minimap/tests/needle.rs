//! The arrow of the minimap: on the ground it turns by what lies between the view and the
//! body, so in third person it shows where the body faces, not where the camera looks.

use std::f32::consts::FRAC_PI_2;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::rig::{app, ground, look};
use crate::menus::hud::minimap::pose::needle;
use crate::menus::hud::minimap::widget::Needle;
use crate::player::Player;
use crate::space::Anchor;

/// Returns the clockwise turn of every arrow of the minimap, in radians.
fn turns(app: &mut App) -> Vec<f32> {
    let mut needles = app
        .world_mut()
        .query_filtered::<&UiTransform, With<Needle>>();
    let found = needles.iter(app.world());
    found.map(|turn| turn.rotation.as_radians()).collect()
}

/// A body that faces east under a view that looks north turns the arrow a quarter right, one
/// that faces the view leaves it up, and without a body the arrow follows the view.
#[test]
fn the_arrow_shows_the_body_not_the_view() {
    let (north, east, up) = (DVec3::Y, DVec3::NEG_Z, DVec3::X);
    assert!((needle(north, east, up) - FRAC_PI_2).abs() < 1e-6);
    assert!((needle(north, -east, up) + FRAC_PI_2).abs() < 1e-6);
    assert_eq!(needle(north, north, up), 0.0);
    let mut app = app("needle");
    let at = ground(0.0, 0.0);
    look(&mut app, at, north);
    assert_eq!(turns(&mut app), vec![0.0]);
    let anchor = *app.world().resource::<Anchor>();
    let turned = |along: DVec3| (anchor.rotation * along).as_vec3();
    let body = Transform::from_translation(anchor.to_render(at));
    let body = body.looking_to(turned(east), turned(at.normalize()));
    app.world_mut().spawn((Player, body));
    look(&mut app, at, north);
    let shown = turns(&mut app);
    assert!(
        shown.len() == 1 && (shown[0] - FRAC_PI_2).abs() < 1e-3,
        "{shown:?}"
    );
}
