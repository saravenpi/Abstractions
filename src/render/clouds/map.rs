//! The cover map of one planet: the broad cover of its cloud field at every texel of the
//! cover grid, at two snapshots, as the bytes of a cube texture with two channels.
//!
//! Snapshots are [`SNAPSHOT`] seconds apart. The even ones live in the first channel and the
//! odd ones in the second, and the shaders read between the two with [`toward_second`]. A
//! channel is written over only at the instant its weight is zero, so the picture never steps,
//! and the snapshot that comes next is worked out a few texels per frame ahead of time.

use crate::weather::{Field, SNAPSHOT, TEXELS, moment, texel};

/// Channels of the texture: one per snapshot it holds.
pub const CHANNELS: usize = 2;

/// Fewest texels of the next snapshot worked out in one frame.
const SLICE: usize = 256;

/// Texels of the next snapshot worked out per second of frame time, on top of [`SLICE`]: a
/// slow frame does more, so that a snapshot is always ready within its [`SNAPSHOT`] seconds.
const PACE: f32 = 8192.0;

/// Returns how far the shaders read from the first channel toward the second at `seconds`.
pub fn toward_second(seconds: f64) -> f32 {
    let (epoch, share) = moment(seconds);
    match epoch.rem_euclid(2) {
        0 => share,
        _ => 1.0 - share,
    }
}

/// Returns the texels of the next snapshot to work out in a frame of `seconds`.
pub fn budget(seconds: f32) -> usize {
    SLICE + (seconds.clamp(0.0, 1.0) * PACE) as usize
}

/// The cover map of one planet.
#[derive(Debug, Clone, PartialEq)]
pub struct CoverMap {
    /// The cloud field the cover is read from.
    field: Field,
    /// Wetness of the ground under every texel.
    wet: Vec<f32>,
    /// The bytes of the texture: for each texel, the cover at the even and at the odd snapshot.
    bytes: Vec<u8>,
    /// Number of the earlier of the two snapshots the bytes hold, once they hold any.
    epoch: Option<i64>,
    /// The snapshot after those two, worked out ahead of time.
    next: Vec<u8>,
    /// Texels of it worked out so far.
    done: usize,
}

impl CoverMap {
    /// Returns the empty map of `field` over ground of wetness `wet` at every texel.
    pub fn new(field: Field, wet: Vec<f32>) -> Self {
        Self {
            field,
            wet,
            bytes: vec![0; TEXELS * CHANNELS],
            epoch: None,
            next: vec![0; TEXELS],
            done: 0,
        }
    }

    /// Returns the bytes of the texture.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the cover of texel number `index` at snapshot `epoch`, as a byte.
    fn cover(&self, index: usize, epoch: i64) -> u8 {
        let second = epoch as f64 * SNAPSHOT;
        let cover = self.field.raw(self.wet[index], texel(index), second);
        (cover * 255.0).round() as u8
    }

    /// Works out up to `texels` more texels of the snapshot after the two held, which are
    /// `epoch` and the one after it.
    fn stage(&mut self, epoch: i64, texels: usize) {
        let end = self.done.saturating_add(texels).min(TEXELS);
        for texel in self.done..end {
            self.next[texel] = self.cover(texel, epoch + 2);
        }
        self.done = end;
    }

    /// Writes snapshot `epoch` into its channel of the bytes, from the staged snapshot when
    /// `staged` and from the field otherwise.
    fn write(&mut self, epoch: i64, staged: bool) {
        let channel = epoch.rem_euclid(2) as usize;
        for texel in 0..TEXELS {
            self.bytes[texel * CHANNELS + channel] = match staged {
                true => self.next[texel],
                false => self.cover(texel, epoch),
            };
        }
    }

    /// Brings the map to `seconds` of universe time, working out up to `texels` texels of the
    /// next snapshot, and returns whether the bytes changed.
    ///
    /// Within one snapshot period nothing changes. When the next period starts, the snapshot
    /// worked out ahead takes the channel whose weight has just reached zero. After a jump in
    /// time both snapshots are worked out at once: a cut, but the right sky.
    pub fn advance(&mut self, seconds: f64, texels: usize) -> bool {
        let now = moment(seconds).0;
        match self.epoch {
            Some(held) if held == now => {
                self.stage(held, texels);
                false
            }
            Some(held) if held + 1 == now => {
                self.stage(held, TEXELS);
                self.write(now + 1, true);
                (self.epoch, self.done) = (Some(now), 0);
                true
            }
            _ => {
                self.write(now, false);
                self.write(now + 1, false);
                (self.epoch, self.done) = (Some(now), 0);
                true
            }
        }
    }
}
