//! Placing the mirror camera under the water each frame, and letting it draw only when it shows
//! something.

use bevy::camera::Exposure;
use bevy::ecs::query::QueryItem;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::super::Sea;
use super::super::floor::SeaFloor;
use super::super::material::WaterMaterial;
use super::super::uniform::WaterUniform;
use super::gate::{Scene, nearest};
use super::plane::{Plane, sees_water};
use super::tune::{CLEAR, FADE, REACH, WET, WOBBLE};
use super::{MirrorCamera, Reflected};
use crate::render::GameCamera;
use crate::settings::{Persistent, Reflections, Settings};

/// What the mirror reads of the game camera: its pose, its lens and its exposure.
type Eye = (
    &'static GlobalTransform,
    &'static Projection,
    Option<&'static Exposure>,
);

/// What the mirror writes on its own camera.
type Mirror = (
    &'static mut Camera,
    &'static mut Transform,
    &'static mut GlobalTransform,
    &'static mut Projection,
    &'static mut Exposure,
);

/// What the mirror reads of the sea: its material, its floor map and whether it is shown.
type SeaParts = (
    &'static MeshMaterial3d<WaterMaterial>,
    &'static SeaFloor,
    &'static Visibility,
);

/// The game camera and the mirror camera.
#[derive(SystemParam)]
pub struct Cameras<'w, 's> {
    /// The game camera, read.
    game: Query<'w, 's, Eye, (With<GameCamera>, Without<MirrorCamera>)>,
    /// The mirror camera, written.
    mirror: Query<'w, 's, Mirror, (With<MirrorCamera>, Without<GameCamera>)>,
    /// The images of the mirror camera.
    images: Query<'w, 's, &'static MirrorCamera>,
}

/// Returns whether a camera with the lens `lens` posed by `pose` sees water under `up`.
fn looks_down(pose: &GlobalTransform, lens: &Projection, up: Vec3) -> bool {
    match lens {
        Projection::Perspective(lens) => {
            sees_water(pose.forward().as_vec3(), up, lens.fov, lens.aspect_ratio)
        }
        _ => false,
    }
}

/// Poses the mirror camera of the game camera `eye` about `plane` and switches it on when `pass`
/// holds: the mirrored pose, the lens of the game camera clipped at the water and cut at
/// [`REACH`], and the same exposure, so that a mirrored robot is as bright as the robot.
fn place(
    mirror: QueryItem<'_, '_, Mirror>,
    eye: QueryItem<'_, '_, Eye>,
    plane: &Plane,
    pass: bool,
) {
    let (mut camera, mut transform, mut global, mut projection, mut exposure) = mirror;
    if camera.is_active != pass {
        camera.is_active = pass;
    }
    let (pose, lens, light) = eye;
    let Projection::Perspective(lens) = lens else {
        return;
    };
    if !pass {
        return;
    }
    let world = plane.eye(pose);
    *global = GlobalTransform::from(world);
    *transform = Transform::from_matrix(world);
    *projection = Projection::Perspective(PerspectiveProjection {
        near_clip_plane: plane.clip(pose),
        far: REACH,
        ..lens.clone()
    });
    if let Some(light) = light {
        *exposure = *light;
    }
}

/// Returns what decides the pass for the game camera posed by `eye` with the lens `lens`, over
/// the sea of floor map `floor`, shown or not, drawn with the values `water`, with the mirrored
/// `actors`, and the water plane.
fn scene(
    want: bool,
    (eye, lens): (&GlobalTransform, &Projection),
    (floor, shown, water): (&SeaFloor, &Visibility, &WaterUniform),
    actors: impl Iterator<Item = Vec3>,
) -> (Scene, Plane) {
    let plane = Plane::under(water);
    let at = eye.translation();
    let scene = Scene {
        want,
        water: *shown != Visibility::Hidden && water.deep.w >= CLEAR && floor.wet >= WET,
        height: plane.height(at),
        looking: looks_down(eye, lens, plane.up),
        nearest: nearest(plane.mirror(at), actors),
    };
    (scene, plane)
}

/// Decides each frame whether the mirror draws, poses its camera under the water and tells the
/// water shader whether to read the mirror image, how it fades and how much the waves bend it.
pub fn aim(
    mut cameras: Cameras,
    actors: Query<&GlobalTransform, (With<Reflected>, Without<MirrorCamera>)>,
    seas: Query<SeaParts, With<Sea>>,
    settings: Option<Res<Persistent<Settings>>>,
    mut materials: ResMut<Assets<WaterMaterial>>,
) {
    let want = settings.is_some_and(|s| s.graphics.lighting.reflections == Reflections::Actors);
    let sea = seas.iter().next();
    let water = sea.and_then(|(handle, ..)| materials.get(&handle.0).map(|m| m.water));
    let (Some((handle, floor, shown)), Some(water), Some(eye)) =
        (sea, water, cameras.game.iter().next())
    else {
        cameras.mirror.iter_mut().for_each(place_off);
        let image = cameras.images.iter().next().map(|m| m.shown(false).clone());
        if let Some((handle, ..)) = sea {
            tell(&mut materials, &handle.0, Vec4::ZERO, image);
        }
        return;
    };
    let actors = actors.iter().map(GlobalTransform::translation);
    let (scene, plane) = scene(want, (eye.0, eye.1), (floor, shown, &water), actors);
    let pass = scene.pass();
    for mirror in &mut cameras.mirror {
        place(mirror, eye, &plane, pass);
    }
    let mirror = Vec4::new(f32::from(u8::from(pass)), FADE, REACH, WOBBLE);
    let image = cameras.images.iter().next().map(|m| m.shown(pass).clone());
    tell(&mut materials, &handle.0, mirror, image);
}

/// Hands the water material `handle` the mirror values `mirror` and the image `image` it reads,
/// keeping its image when none is given, and touches the material only when one of them changed.
fn tell(
    materials: &mut Assets<WaterMaterial>,
    handle: &Handle<WaterMaterial>,
    mirror: Vec4,
    image: Option<Handle<Image>>,
) {
    let Some(material) = materials.get(handle) else {
        return;
    };
    let image = image.unwrap_or_else(|| material.mirror.clone());
    if material.water.mirror == mirror && material.mirror == image {
        return;
    }
    if let Some(mut material) = materials.get_mut(handle) {
        material.water.mirror = mirror;
        material.mirror = image;
    }
}

/// Switches the mirror camera off.
fn place_off(mirror: QueryItem<'_, '_, Mirror>) {
    let mut camera = mirror.0;
    if camera.is_active {
        camera.is_active = false;
    }
}
