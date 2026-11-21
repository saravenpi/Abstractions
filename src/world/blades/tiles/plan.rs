//! Which bands of which tiles the camera wants: every band within its reach of the eye, nearest first, and the
//! bands past their reach dropped.

use bevy::prelude::*;

use super::{Meadow, Spot};
use crate::world::blades::fade::reach;
use crate::world::blades::tune::{AHEAD, BANDS, END, LINGER, RELIEF, TILE};

/// Where the camera stands for the bands: its chart position, and how high above the ground it counts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Eye {
    /// Chart position of the camera.
    pub at: Vec2,
    /// Metres the camera counts above the ground: its height over the ground under it less [`RELIEF`].
    pub lift: f32,
}

impl Eye {
    /// Returns the eye of a camera at chart position `at`, `above` metres over the ground under it.
    pub fn new(at: Vec2, above: f32) -> Self {
        Self {
            at,
            lift: (above - RELIEF).max(0.0),
        }
    }

    /// Returns the distance in metres from the eye to the nearest point of tile `key`.
    pub fn away(&self, key: IVec2) -> f32 {
        let low = key.as_vec2() * TILE;
        let flat = self.at.clamp(low, low + TILE).distance(self.at);
        Vec2::new(flat, self.lift).length()
    }
}

/// Returns the distance in metres from the eye within which `band` is built: its reach, and [`AHEAD`] more for
/// every band but the farthest, whose last blades shrink away before the end anyway.
pub fn within(band: usize) -> f32 {
    match band {
        0 => reach(0),
        _ => reach(band) + AHEAD,
    }
}

impl Meadow {
    /// Drops every band and flight farther from `eye` than [`within`] and [`LINGER`], with the entities of the
    /// bands dropped into `gone`, and forgets the turf no band holds any more.
    fn prune(&mut self, eye: Eye, gone: &mut Vec<Entity>) {
        let before = self.tiles.len() + self.flights.len();
        let near = |(key, band): &Spot| eye.away(*key) < within(*band) + LINGER;
        self.flights.retain(|spot, _| near(spot));
        self.tiles.retain(|spot, tile| {
            let keep = near(spot);
            if let (false, Some(entity)) = (keep, tile.entity) {
                gone.push(entity);
            }
            keep
        });
        if before != self.tiles.len() + self.flights.len() {
            self.evict();
        }
    }

    /// Drops the bands past their reach of `eye`, with their entities into `gone`, and returns into `wanted`
    /// the bands within their reach that want building, missing or stale, nearest first and the farthest band
    /// of a tile before its nearer ones.
    pub(in crate::world::blades) fn plan(
        &mut self,
        eye: Eye,
        wanted: &mut Vec<(f32, Spot)>,
        gone: &mut Vec<Entity>,
    ) {
        self.prune(eye, gone);
        wanted.clear();
        let span = ((END + AHEAD) / TILE).ceil() as i32 + 1;
        let home = (eye.at / TILE).floor().as_ivec2();
        for (x, y) in (-span..=span).flat_map(|x| (-span..=span).map(move |y| (x, y))) {
            let key = home + IVec2::new(x, y);
            let away = eye.away(key);
            for band in (0..BANDS.len()).take_while(|band| away < within(*band)) {
                if self.tiles.get(&(key, band)).is_none_or(|tile| tile.stale) {
                    wanted.push((away, (key, band)));
                }
            }
        }
        wanted.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.1.cmp(&b.1.1)));
    }
}
