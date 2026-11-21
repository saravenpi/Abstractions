//! Baking the far bodies: each solid planet and moon of a star system gets the texture of its real
//! surface, baked off the main thread, coarse when the camera enters the system and finer as the
//! body grows on the screen. Until the first one is ready the body shows the flat colour of its
//! class.

use std::sync::Arc;

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::material::BodyMaterial;
use super::surface::image;
use crate::render::air::Air;
use crate::universe::{PlanetClass, PlanetSpec};
use crate::world::Globe;
use crate::world::globe::Face;
use crate::world::globe::far::{Ball, SIDES, ball, face};

/// Texture coordinates that mark the cloud shell; the ground has none.
pub const SHELL_UV: [f32; 2] = [2.0, 2.0];

/// A bake of one level of [`SIDES`] being done: the level and its task.
type Pending = (usize, Task<Baked>);

/// The baked surface of a solid body: the textures of the levels baked so far, the bake under
/// way, and the globe they are read from once the first bake has built it.
#[derive(Component)]
pub(super) struct Surface {
    /// The body.
    spec: PlanetSpec,
    /// Its globe, once built.
    globe: Option<Arc<Globe>>,
    /// The texture of each level of [`SIDES`], once baked.
    pub(super) held: [Option<Handle<Image>>; SIDES.len()],
    /// The bake under way.
    task: Option<Pending>,
}

/// A baked level: the globe it was read from, its texture, and what the body shows as a dot.
pub(super) struct Baked {
    /// The globe.
    globe: Arc<Globe>,
    /// The texture of the six faces with its mipmaps.
    image: Image,
    /// The mean colour, sea and glow of the surface.
    pub(super) ball: Ball,
}

/// Returns the radius of the cloud shell of `body` in metres, halfway up its cloud band, or
/// nothing for a body whose class or whose band has no clouds.
pub fn shell_radius(body: &PlanetSpec) -> Option<f64> {
    let cloudy = Air::of(&body.record).clouds > 0.0 && body.cloud_top > body.cloud_base;
    cloudy.then_some(body.radius + 0.5 * (body.cloud_base + body.cloud_top))
}

/// Returns true when `body` is solid and has a cloud shell, drawn as a body of its own.
pub fn cloudy(body: &PlanetSpec) -> bool {
    body.record.class != PlanetClass::GasGiant && shell_radius(body).is_some()
}

/// Bakes level `level` of [`SIDES`] of `globe`, or of the globe of `spec` built first, the six
/// faces on tasks of their own.
pub(super) async fn bake(spec: PlanetSpec, globe: Option<Arc<Globe>>, level: usize) -> Baked {
    let globe = globe.unwrap_or_else(|| Arc::new(Globe::of(&spec)));
    let pool = AsyncComputeTaskPool::get();
    let side = SIDES[level];
    let faces = Face::ALL.map(|at| {
        let globe = globe.clone();
        pool.spawn(async move { face(&globe, at, side) })
    });
    let mut maps = Vec::with_capacity(faces.len());
    for task in faces {
        maps.push(task.await);
    }
    Baked {
        ball: ball(&globe, &maps),
        image: image(&maps),
        globe,
    }
}

impl Surface {
    /// Starts baking the coarsest level of `body` off the main thread, or nothing for a gas giant,
    /// which has no ground and keeps its bands.
    pub(super) fn start(body: &PlanetSpec) -> Option<Self> {
        if body.record.class == PlanetClass::GasGiant {
            return None;
        }
        let mut surface = Self {
            spec: body.clone(),
            globe: None,
            held: default(),
            task: None,
        };
        surface.ask(0);
        Some(surface)
    }

    /// Starts baking `level` when neither it nor a finer one is held, nothing else bakes and, past
    /// the first level, the globe is built.
    pub(super) fn ask(&mut self, level: usize) {
        let ready = level == 0 || self.globe.is_some();
        let held = self.held[level..].iter().any(Option::is_some);
        if held || self.task.is_some() || !ready {
            return;
        }
        let task = bake(self.spec.clone(), self.globe.clone(), level);
        self.task = Some((level, AsyncComputeTaskPool::get().spawn(task)));
    }

    /// Returns the finest texture held, if any.
    pub(super) fn finest(&self) -> Option<Handle<Image>> {
        self.held.iter().rev().flatten().next().cloned()
    }

    /// Lets go of every level over `level`.
    pub(super) fn keep(&mut self, level: usize) {
        for held in self.held.iter_mut().skip(level + 1) {
            *held = None;
        }
    }
}

/// Gives every body whose bake is done its texture and its material values, a frame after the
/// task is done. Nothing else happens on the main thread.
pub(super) fn finish(
    mut baking: Query<(&mut Surface, &MeshMaterial3d<BodyMaterial>)>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<BodyMaterial>>,
) {
    for (mut surface, handle) in &mut baking {
        let Some((level, task)) = surface.task.as_mut() else {
            continue;
        };
        let level = *level;
        let Some(baked) = block_on(poll_once(task)) else {
            continue;
        };
        let texture = images.add(baked.image);
        surface.held[level] = Some(texture.clone());
        surface.globe = Some(baked.globe);
        surface.task = None;
        if surface.finest() == Some(texture.clone())
            && let Some(mut material) = materials.get_mut(&handle.0)
        {
            material.baked(&surface.spec, &baked.ball, texture);
        }
    }
}
