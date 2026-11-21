//! Where the strip of an animal shows: the top of its head projected to the screen as a marker over
//! the world is placed, nothing when it is off screen or behind the camera, and the strip hung over
//! it clear of the crosshair.

use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::onscreen::place::{self, Frustum};
use super::{BAR, STRIP};
use crate::fauna::Part;
use crate::render::GameCamera;
use crate::settings::{Persistent, Settings};

/// Pixels between the top of the head on screen and the bottom of the strip.
const LIFT: f32 = 8.0;

/// Metres above the top of the animal the strip is anchored at.
const HEAD: f32 = 0.15;

/// Half the side of the square around the crosshair the strip keeps out of, in pixels: past the
/// ticks at their widest.
const CLEAR: f32 = 28.0;

/// Returns where the render position `head` shows on a `screen` of interface pixels seen by
/// `camera` through `lens`, or nothing when it is behind the camera or off the screen.
pub fn over(head: Vec3, camera: &GlobalTransform, (lens, screen): (Frustum, Vec2)) -> Option<Vec2> {
    let dir = camera.affine().inverse().transform_point3(head);
    let whole = Rect::from_corners(Vec2::ZERO, screen);
    let placed = place::place(dir, lens, whole);
    placed.arrow.is_none().then_some(placed.pos)
}

/// Returns the left and the bottom of the strip from the centre of a `screen` for a head at `pos`:
/// centred over it, [`LIFT`] above it, lifted clear of the crosshair, on whole steps of 4 pixels.
pub fn hung(pos: Vec2, screen: Vec2) -> (f32, f32) {
    let left = pos.x - screen.x / 2.0 - BAR.0;
    let bottom = screen.y / 2.0 - pos.y + LIFT;
    let across = left < CLEAR && left + 2.0 * BAR.0 > -CLEAR;
    let crossed = across && bottom < CLEAR && bottom + STRIP > -CLEAR;
    let bottom = if crossed { CLEAR } else { bottom };
    let step = |v: f32| (v / 4.0).round() * 4.0;
    (step(left), step(bottom))
}

/// The placed bounds of the parts of the animals.
pub(super) type Bounds<'w, 's> =
    Query<'w, 's, (&'static GlobalTransform, &'static Aabb), With<Part>>;

/// Returns the top of an animal placed at `placed` whose parts are `limbs`, from their bounds in
/// `parts`, raised by [`HEAD`] along its up. Only its own parts are read.
pub(super) fn top(placed: &GlobalTransform, limbs: &[Option<Entity>], parts: &Bounds) -> Vec3 {
    let (at, up) = (placed.translation(), placed.up().as_vec3());
    let height = limbs
        .iter()
        .flatten()
        .filter_map(|limb| parts.get(*limb).ok())
        .map(|(part, aabb)| {
            let centre = part.transform_point(Vec3::from(aabb.center));
            let reach = (Vec3::from(aabb.half_extents) * part.scale().abs()).max_element();
            up.dot(centre - at) + reach
        })
        .fold(0.0_f32, f32::max);
    at + up * (height + HEAD)
}

/// The game camera as the rig placed it this frame: its own transform, the body that carries it
/// and its projection.
type Camera<'a> = (&'a Transform, Option<&'a ChildOf>, &'a Projection);

/// What the strip reads of the screen: the camera and what carries it, the window, the scale of
/// the interface and the field of view in the settings.
pub(super) type Lens<'w, 's> = (
    (
        Query<'w, 's, Camera<'static>, With<GameCamera>>,
        Query<'w, 's, &'static Transform, Without<GameCamera>>,
    ),
    Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    Option<Res<'w, UiScale>>,
    Res<'w, Persistent<Settings>>,
);

/// Returns the camera where the rig put it this frame, the screen in interface pixels and what the
/// camera sees on it, with the field of view it has now.
pub(super) fn sight(lens: &Lens) -> Option<(GlobalTransform, Vec2, Frustum)> {
    let ((cameras, holders), window, scale, settings) = lens;
    let ((local, parent, projection), window) = (cameras.single().ok()?, window.single().ok()?);
    let holder = parent.and_then(|parent| holders.get(parent.parent()).ok());
    let camera =
        GlobalTransform::from(holder.map_or(*local, |holder| holder.mul_transform(*local)));
    let scale = scale.as_ref().map_or(1.0, |scale| scale.0).max(0.1);
    let screen = Vec2::new(window.width(), window.height()) / scale;
    let fov = (settings.graphics.fov as f32).to_radians();
    Some((
        camera,
        screen,
        Frustum::of(Some(projection), fov, window, scale),
    ))
}
