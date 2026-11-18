//! Tests of the creatures on a small globe: heights along the local vertical while the camera
//! walks and the anchor is rebased under it.

use std::sync::Arc;

use bevy::prelude::*;

use super::carry::moved;
use super::env::{Ctx, Sky};
use super::flock::{CAPACITY, Flocks, formation};
use super::wander::{Wander, dormant, ground};
use crate::effects::eye::Eye;
use crate::effects::ground::Ground;
use crate::effects::kind::Ambient;
use crate::effects::rng::Rng;
use crate::space::{Anchor, Shift, altitude, rebased, up};
use crate::universe::Universe;
use crate::world::globe::Face;
use crate::world::{Biome, Globe, Surface, Terrain};

/// Sea level radius of the test planet in meters.
const RADIUS: f64 = 3000.0;

/// Seconds per simulated frame.
const DT: f32 = 1.0 / 30.0;

/// Walking speed of the camera in meters per second.
const PACE: f32 = 5.0;

/// A camera walking over the test planet, rebased like the frame systems rebase the player.
pub(super) struct Walk {
    pub(super) terrain: Terrain,
    pub(super) anchor: Anchor,
    pub(super) camera: Vec3,
    pub(super) rebases: usize,
    pub(super) walked: f32,
}

impl Walk {
    /// Starts at the spawn point of the test planet, seen through the face that holds it.
    pub(super) fn new() -> Self {
        let globe = Arc::new(Globe::new(1, RADIUS, 22.0));
        let north = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
        let spawn = north.find_spawn();
        Self {
            anchor: Anchor::surface(Universe::new(1).home(), &north, spawn),
            terrain: north,
            camera: spawn,
            rebases: 0,
            walked: 0.0,
        }
    }

    /// Returns the eye of the camera.
    pub(super) fn eye(&self) -> Eye {
        Eye::new(
            &self.anchor,
            self.terrain.globe(),
            self.camera,
            Quat::IDENTITY,
        )
    }

    /// Walks one frame along the ground, eye 1.7 m over the ground or the sea, and returns the
    /// motion of render space when that took the anchor past its rebase limit.
    pub(super) fn step(&mut self) -> Option<Shift> {
        let eye = self.eye();
        let ground = Ground::new(&self.anchor, &self.terrain, &eye);
        let ahead = self.camera + ground.turn(Vec3::X * PACE * DT);
        self.camera = ground.raised(ahead, ground.height(ahead).max(0.0) + 1.7);
        self.walked += PACE * DT;
        let next = rebased(&self.anchor, RADIUS, self.camera)?;
        let shift = moved(Some(self.anchor), &next)?;
        self.camera = shift.point(self.camera);
        self.anchor = next;
        self.rebases += 1;
        Some(shift)
    }
}

/// Returns the frame view of the sky around `eye` at `frame`, at night with a west wind.
pub(super) fn sky(eye: &Eye, frame: usize) -> Sky {
    Sky {
        dt: DT,
        now: frame as f32 * DT,
        camera: eye.at,
        tilt: eye.tilt,
        wind: Vec2::X,
        ambient: Ambient {
            biome: Biome::Meadow,
            surface: Surface::Grass,
            height: eye.altitude,
            daylight: 0.0,
            wind: 0.4,
        },
    }
}

/// Far from the render origin of a small globe, where render `y` is meters off, a point raised
/// to a level 12 m over the camera reads that altitude, and the ground is the same through any cube
/// face view.
#[test]
fn heights_follow_the_local_vertical() {
    let walk = Walk::new();
    let eye = walk.eye();
    let ground = Ground::new(&walk.anchor, &walk.terrain, &eye);
    let globe = walk.terrain.globe();
    let far = walk.camera + Vec3::new(150.0, 0.0, -40.0);
    let level = walk.camera.y + 12.0;
    let raised = ground.raised(far, level);
    assert!((altitude(&walk.anchor, globe, raised) - level).abs() < 1e-3);
    assert!((raised.y - walk.camera.y - 12.0).abs() > 2.0);
    assert!(ground.span(far, raised).y > 0.0 && up(&walk.anchor, far).dot(raised - far) > 0.0);
    let east = Terrain::on(globe.clone(), globe.face_chart(Face::PosX));
    let other = Ground::new(&walk.anchor, &east, &eye);
    assert!((ground.height(far) - other.height(far)).abs() < 5e-3);
}

/// While the camera walks 500 m over a 3 km globe, through many rebases, both flocks and every
/// bird place stay above the ground and above the sea.
#[test]
fn birds_stay_above_ground_and_sea_on_a_small_globe() {
    let (mut walk, mut flocks, mut rng) = (Walk::new(), Flocks::default(), Rng::default());
    let mut lowest = f32::MAX;
    for frame in 0..3100 {
        if let Some(shift) = walk.step() {
            flocks
                .0
                .iter_mut()
                .for_each(|flock| flock.center = shift.point(flock.center));
        }
        let eye = walk.eye();
        let sky = sky(&eye, frame);
        let mut ctx = Ctx {
            ground: Ground::new(&walk.anchor, &walk.terrain, &eye),
            rng: &mut rng,
        };
        for (index, flock) in flocks.0.iter_mut().enumerate() {
            flock.advance(index, &sky, &mut ctx);
            for slot in 0..CAPACITY[index] {
                let bird = flock.center + sky.tilt * formation(slot, flock.heading);
                lowest = lowest.min(ctx.ground.altitude(bird) - ground(&ctx, bird));
            }
        }
    }
    assert!(
        walk.walked >= 500.0 && walk.rebases >= 10,
        "{}",
        walk.rebases
    );
    assert!(lowest > 5.0, "a bird flew {lowest} m over the ground");
}

/// A flyer that is neither shown nor wanted is left alone, so it reads no terrain.
#[test]
fn hidden_flyers_rest() {
    let mut wander = Wander::new(3);
    assert!(dormant(&wander, 3.0) && dormant(&wander, 0.0) && !dormant(&wander, 3.5));
    wander.shown = 0.2;
    assert!(!dormant(&wander, 0.0));
}
