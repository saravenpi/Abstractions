//! A sea floor map on the test planet, filled and read the way the game does.

use std::f64::consts::TAU;
use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::paths::{along, mesh_height};
use crate::world::Globe;
use crate::world::globe::Face;
use crate::world::sea::depth::{self, DEEP, LAND, TEXEL};
use crate::world::sea::floor::SeaFloor;
use crate::world::sea::seat::Seat;

/// A sea floor map on a test planet, filled the way the refresh system fills it.
pub struct Rig {
    globe: Arc<Globe>,
    /// Seat of the sea.
    pub seat: Seat,
    floor: SeaFloor,
    images: Assets<Image>,
}

impl Rig {
    /// Seats a sea 300 m from `focus` on `globe`, so that sea coordinates are not trivial.
    pub fn new(globe: &Arc<Globe>, focus: DVec3) -> Self {
        let mut images = Assets::<Image>::default();
        Self {
            globe: globe.clone(),
            seat: Seat::at(globe, along(focus.normalize(), 2.0, 300.0)),
            floor: SeaFloor::new(&mut images),
            images,
        }
    }

    /// Centers the map on planet position `at` and lets it fill completely.
    pub fn fill(&mut self, at: DVec3) -> IVec2 {
        let p = self.seat.coords(at);
        let centre = depth::centre_texel(Vec3::new(p.x, 0.0, p.y));
        for _ in 0..40 {
            self.floor.follow(centre, &self.seat, &mut self.images);
        }
        centre
    }

    /// Returns the bytes of the map.
    pub fn bytes(&self) -> Vec<u8> {
        let image = self.images.get(&self.floor.image());
        image
            .and_then(|image| image.data.clone())
            .unwrap_or_default()
    }

    /// Returns the depth in meters stored in `texel`.
    pub fn stored(&self, data: &[u8], texel: IVec2) -> f32 {
        let at = depth::offset(texel);
        f32::from(u16::from_be_bytes([data[at], data[at + 1]])) / 65535.0 * (DEEP + LAND) - LAND
    }

    /// Returns the depth the shader reads at planet position `at`: the linear filter of the map.
    pub fn depth(&self, data: &[u8], at: DVec3) -> f32 {
        let p = self.seat.coords(at) / TEXEL;
        let (cell, f) = (p.floor().as_ivec2(), p - p.floor());
        let tap = |dx: i32, dz: i32| self.stored(data, cell + IVec2::new(dx, dz));
        let near = tap(0, 0) + (tap(1, 0) - tap(0, 0)) * f.x;
        let far = tap(0, 1) + (tap(1, 1) - tap(0, 1)) * f.x;
        near + (far - near) * f.y
    }

    /// Returns true when the map waterline passes within one texel of planet position `at`.
    pub fn waterline_near(&self, data: &[u8], at: DVec3) -> bool {
        let ring = (0..16).map(|i| along(at.normalize(), f64::from(i) * TAU / 16.0, 2.0));
        let depths: Vec<f32> = ring.map(|point| self.depth(data, point)).collect();
        depths.iter().any(|d| *d <= 0.0) && depths.iter().any(|d| *d >= 0.0)
    }

    /// Walks rays from `focus` and returns the cube face of every clean crossing of the terrain
    /// mesh waterline, after checking that the map waterline lies within one texel of each.
    pub fn check_waterline(&self, focus: DVec3) -> Vec<Face> {
        let data = self.bytes();
        let mut faces = Vec::new();
        for (ray, step) in (0..24).flat_map(|ray| (8..360).map(move |step| (ray, step))) {
            let point = |reach: f64| along(focus.normalize(), f64::from(ray) * TAU / 24.0, reach);
            let height = |reach: f64| mesh_height(&self.globe, point(reach).normalize());
            let reach = f64::from(step) * 0.25;
            let crossing = height(reach) * height(reach + 0.25) < 0.0;
            let (before, after) = (height(reach - 2.0), height(reach + 2.25));
            if crossing && before * after < 0.0 && before.abs().min(after.abs()) > 0.05 {
                assert!(
                    self.waterline_near(&data, point(reach)),
                    "ray {ray} at {reach} m"
                );
                faces.push(Face::locate(point(reach).normalize()).0);
            }
        }
        faces
    }
}
