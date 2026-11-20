//! The cover map built one face at a time: each face of the cube grid is worked out by its own
//! task, with the wetness of its ground, both snapshots and the one after them, and the six
//! parts are joined into the map without any texel being worked out twice.

use bevy::math::DVec3;

use super::{CHANNELS, CoverMap};
use crate::weather::{Field, SNAPSHOT, TEXELS, moment, texel, wet};
use crate::world::Globe;

/// Texels of one face of the grid.
const FACE_TEXELS: usize = TEXELS / 6;

/// Everything a map holds for one face of the grid, built from the same snapshots as a map
/// built at once.
pub struct Slab {
    /// Wetness of the ground under each texel of the face.
    wet: Vec<f32>,
    /// The bytes of the texture for the face, two channels per texel.
    bytes: Vec<u8>,
    /// The snapshot after the two the bytes hold.
    next: Vec<u8>,
}

/// Returns the cover of the texel in unit direction `dir` over ground of wetness `wet` at
/// `second` seconds of universe time, as a byte.
pub(super) fn level(field: &Field, wet: f32, dir: DVec3, second: f64) -> u8 {
    (field.raw(wet, dir, second) * 255.0).round() as u8
}

/// Works out face number `face` of the map of `field` over `globe` at `seconds`: the snapshot
/// that holds `seconds`, the one after it and the one after that. It reads only its arguments
/// and is safe on any thread.
pub fn slab(field: &Field, globe: &Globe, face: usize, seconds: f64) -> Slab {
    let now = moment(seconds).0;
    let at = |epoch: i64| epoch as f64 * SNAPSHOT;
    let mut slab = Slab {
        wet: Vec::with_capacity(FACE_TEXELS),
        bytes: vec![0; FACE_TEXELS * CHANNELS],
        next: Vec::with_capacity(FACE_TEXELS),
    };
    for local in 0..FACE_TEXELS {
        let dir = texel(face * FACE_TEXELS + local);
        let ground = wet(globe, dir);
        for epoch in [now, now + 1] {
            let channel = epoch.rem_euclid(2) as usize;
            slab.bytes[local * CHANNELS + channel] = level(field, ground, dir, at(epoch));
        }
        slab.next.push(level(field, ground, dir, at(now + 2)));
        slab.wet.push(ground);
    }
    slab
}

impl CoverMap {
    /// Joins the six faces of the map of `field` at `seconds`, given in the order of the
    /// faces, into a map that holds that snapshot and the next and has the one after worked
    /// out already.
    pub fn from_slabs(field: Field, seconds: f64, slabs: Vec<Slab>) -> Self {
        let mut map = Self {
            field,
            wet: Vec::with_capacity(TEXELS),
            bytes: Vec::with_capacity(TEXELS * CHANNELS),
            epoch: Some(moment(seconds).0),
            next: Vec::with_capacity(TEXELS),
            done: TEXELS,
        };
        for slab in slabs {
            map.wet.extend(slab.wet);
            map.bytes.extend(slab.bytes);
            map.next.extend(slab.next);
        }
        map
    }
}
