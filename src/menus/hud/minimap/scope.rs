//! The state of the minimap and the systems that keep its ground image: the mode the key set,
//! the speed and the range, the patch on screen and the one being painted.
//!
//! No ground is sampled on the main thread. One worker task paints one patch at a time; the
//! main thread polls it and copies the finished image.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::paint::{BYTES, render};
use super::patch::{Patch, TEXELS, next};
use super::pose::Reader;
use super::range::{Mode, eased, range, step};
use super::widget::Part;
use crate::universe::BodyId;

/// Fastest ground speed in metres per second the range follows. Anything faster is a jump to
/// another place, not a speed, and is left out.
const FASTEST: f32 = 2000.0;

/// How fast the measured speed follows the real one, per second.
const SETTLE: f32 = 2.0;

/// What the minimap remembers from one frame to the next.
#[derive(Resource, Default)]
pub struct Scope {
    /// What the key asked for.
    pub mode: Mode,
    /// The range step the speed asks for.
    pub step: usize,
    /// The range in metres the map shows now, easing toward its step. Zero before the first.
    pub radius: f32,
    /// Smoothed ground speed in metres per second.
    pub speed: f32,
    /// The planet and the place of the player on the last frame.
    pub last: Option<(BodyId, DVec3)>,
    /// The patch the image shows.
    pub patch: Option<Patch>,
    /// The patch being painted and its task.
    pub job: Option<(Patch, Task<Vec<u8>>)>,
    /// The image of the ground.
    pub image: Handle<Image>,
}

/// Returns an image of the size of a patch with nothing on it, drawn texel by texel.
fn blank() -> Image {
    let side = TEXELS as u32;
    let mut image = Image::new(
        Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![0; TEXELS * TEXELS * BYTES],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

/// Gives a new minimap its image and forgets the ground of the world before.
pub fn fresh(
    mut scope: ResMut<Scope>,
    mut images: Option<ResMut<Assets<Image>>>,
    mut grounds: Query<&mut ImageNode, (Added<ImageNode>, With<Part>)>,
) {
    for mut node in &mut grounds {
        if let Some(images) = images.as_deref_mut() {
            scope.image = images.add(blank());
        }
        node.image = scope.image.clone();
        (scope.patch, scope.job, scope.last, scope.radius) = (None, None, None, 0.0);
    }
}

/// Follows the player on a planet: measures the ground speed, picks the range step it asks
/// for and eases the shown range toward it. Another planet starts from nothing.
pub fn track(time: Res<Time>, reader: Reader, mut scope: ResMut<Scope>) {
    let Some((pose, globe)) = reader.pose().zip(reader.globe()) else {
        scope.last = None;
        return;
    };
    let seconds = time.delta_secs();
    match scope.last {
        Some((body, was)) if body == pose.body && seconds > 0.0 => {
            let raw = (pose.at - was).length() as f32 / seconds;
            let gain = f32::from(raw <= FASTEST) * (1.0 - (-SETTLE * seconds).exp());
            scope.speed += (raw.min(FASTEST) - scope.speed) * gain;
        }
        Some((body, _)) if body == pose.body => {}
        _ => (scope.patch, scope.job, scope.radius, scope.speed) = (None, None, 0.0, 0.0),
    }
    scope.last = Some((pose.body, pose.at));
    scope.step = step(scope.speed, scope.step);
    if let Some(goal) = range(scope.mode, scope.step, globe.radius()) {
        scope.radius = eased(scope.radius, goal, seconds);
    }
}

/// Starts painting a new patch on a worker thread when the player nears the edge of the one
/// shown or the range step changed, unless one is already being painted.
pub fn refill(reader: Reader, mut scope: ResMut<Scope>) {
    let Some((pose, globe)) = reader.pose().zip(reader.globe()) else {
        return;
    };
    let goal = range(scope.mode, scope.step, globe.radius()).filter(|_| scope.job.is_none());
    let wanted = goal.and_then(|goal| next(scope.patch, pose.at.normalize(), globe.radius(), goal));
    if let Some(patch) = wanted {
        let globe = globe.clone();
        let task = AsyncComputeTaskPool::get().spawn(async move { render(&globe, &patch) });
        scope.job = Some((patch, task));
    }
}

/// Takes the patch a worker finished: copies its image and shows it from now on.
pub fn collect(mut scope: ResMut<Scope>, images: Option<ResMut<Assets<Image>>>) {
    let scope = scope.bypass_change_detection();
    let done = scope
        .job
        .as_mut()
        .and_then(|job| block_on(poll_once(&mut job.1)));
    let Some(pixels) = done else {
        return;
    };
    scope.patch = scope.job.take().map(|job| job.0);
    if let Some(mut image) = images.and_then(|images| images.into_inner().get_mut(&scope.image)) {
        image.data = Some(pixels);
    }
}
