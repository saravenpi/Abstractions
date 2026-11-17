//! The view from the ship: a cockpit camera with a slight lag, and a chase camera behind it.

use bevy::ecs::system::SystemParam;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::body::Ship;
use super::helm::Helm;
use crate::player::Player;
use crate::render::GameCamera;
use crate::space::{Anchor, Frame, Location, Placed, Travel};

/// Eye point of the pilot in body axes, under the canopy.
const SEAT: DVec3 = DVec3::new(0.0, 0.9, -1.3);

/// Place of the chase camera in the axes it lags behind with: above and behind the ship.
const CHASE: DVec3 = DVec3::new(0.0, 3.4, 13.0);

/// Rates per second at which the cockpit and the chase camera catch up with the ship.
const CATCH_UP: (f64, f64) = (14.0, 4.5);

/// The orientation the camera had last frame, and the frame it was expressed in.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct View(Option<(Frame, DQuat)>);

/// Run condition: a world exists and the player is seated in the ship.
pub fn seated(location: Option<Res<Location>>) -> bool {
    location.is_some_and(|location| location.travel == Travel::InShip)
}

/// Filter of the player body, which is not the camera.
type OnlyBody = (With<Player>, Without<GameCamera>);

/// Filter of the camera, a child of the player body.
type OnlyEyes = (With<GameCamera>, Without<Player>);

/// What the camera reads.
#[derive(SystemParam)]
pub struct Sight<'w> {
    /// Frame time.
    time: Res<'w, Time>,
    /// Which view is on.
    helm: Res<'w, Helm>,
    /// The render anchor.
    anchor: Res<'w, Anchor>,
}

/// Puts the player body and its camera in the ship: the body takes the pose of the camera and
/// the camera sits on it without offset.
pub fn look(
    sight: Sight,
    mut view: ResMut<View>,
    ship: Single<&Placed, With<Ship>>,
    mut body: Single<&mut Transform, OnlyBody>,
    mut eyes: Single<&mut Transform, OnlyEyes>,
) {
    let (anchor, chase) = (*sight.anchor, sight.helm.chase);
    let rate = if chase { CATCH_UP.1 } else { CATCH_UP.0 };
    let kept = view.0.filter(|(frame, _)| *frame == anchor.frame);
    let from = kept.map_or(ship.rot, |(_, rot)| rot);
    let rot = from.slerp(ship.rot, 1.0 - (-rate * sight.time.delta_secs_f64()).exp());
    view.0 = Some((anchor.frame, rot));
    let offset = if chase { rot * CHASE } else { ship.rot * SEAT };
    body.translation = anchor.to_render(ship.pos + offset);
    body.rotation = anchor.rotation_to_render(rot);
    eyes.set_if_neq(Transform::IDENTITY);
}
