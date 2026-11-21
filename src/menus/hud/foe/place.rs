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
use crate::space::{Anchor, Placed};

/// Pixels between the top of the head on screen and the bottom of the strip.
const LIFT: f32 = 8.0;

/// Metres above the top of the animal the strip is anchored at.
pub(super) const HEAD: f32 = 0.15;

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
/// centred over it, [`LIFT`] above it, lifted clear of the crosshair, on whole pixels so it moves
/// with its animal without steps.
pub fn hung(pos: Vec2, screen: Vec2) -> (f32, f32) {
    let left = pos.x - screen.x / 2.0 - BAR.0;
    let bottom = screen.y / 2.0 - pos.y + LIFT;
    let across = left < CLEAR && left + 2.0 * BAR.0 > -CLEAR;
    let crossed = across && bottom < CLEAR && bottom + STRIP > -CLEAR;
    let bottom = if crossed { CLEAR } else { bottom };
    let step = |v: f32| v.round();
    (step(left), step(bottom))
}

/// The parts of the animals: their places on their animal, which the walk wrote this frame, and
/// their bounds.
pub(super) type Bounds<'w, 's> = Query<'w, 's, (&'static Transform, &'static Aabb), With<Part>>;

/// Returns the render place of an animal this frame: from its pose in the frame, `placed`, through
/// `anchor`, as the frame upkeep will copy it before the transforms propagate, or `transform`
/// with no pose or no anchor. Last frame's global transform would trail a running animal.
pub fn rooted(
    anchor: Option<&Anchor>,
    transform: &Transform,
    placed: Option<&Placed>,
) -> Transform {
    let (Some(anchor), Some(placed)) = (anchor, placed) else {
        return *transform;
    };
    Transform {
        translation: anchor.to_render(placed.pos),
        rotation: anchor.rotation_to_render(placed.rot),
        scale: transform.scale,
    }
}

/// Returns the top of an animal whose root is at `root` this frame and whose body sits at `base`
/// in that root's frame, from the places on it and the bounds of its parts `limbs`, raised by
/// [`HEAD`] along its up: the top of a floating body as much as the head of a walker.
pub fn crown(root: &Transform, base: Vec3, limbs: impl Iterator<Item = (Transform, Aabb)>) -> Vec3 {
    let up = root.rotation * Vec3::Y;
    let foot = root.transform_point(base);
    let height = limbs
        .map(|(local, aabb)| {
            let centre = root.transform_point(local.transform_point(Vec3::from(aabb.center)));
            let scale = (root.scale * local.scale).abs();
            up.dot(centre - foot) + (Vec3::from(aabb.half_extents) * scale).max_element()
        })
        .fold(0.0_f32, f32::max);
    foot + up * (height + HEAD)
}

/// Returns the foot and the top of an animal at `root` this frame whose body sits at `base` in
/// that root's frame and whose drawn parts are `limbs`, read from `parts`: only the parts of its
/// own body.
pub(super) fn top(
    root: &Transform,
    base: Vec3,
    limbs: &[Option<Entity>],
    parts: &Bounds,
) -> (Vec3, Vec3) {
    let found = limbs
        .iter()
        .flatten()
        .filter_map(|limb| parts.get(*limb).ok());
    let foot = root.transform_point(base);
    let head = crown(root, base, found.map(|(local, aabb)| (*local, *aabb)));
    (foot, head)
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
