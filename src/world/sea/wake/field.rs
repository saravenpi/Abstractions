//! The height of the waves of one packet, the CPU twin of `field.wesl`.
//!
//! A point is given in the frame of the packet: `s` metres along the direction the source moved
//! and `c` metres across it, from where the source laid the packet.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::prelude::*;

use super::packet::Packet;
use super::tune::{ARC, LIFE, MOVING, RESOLVE, SPAN};
use crate::render::smoothstep;

/// Returns how much of a wave of wave number `k` the footprint `foot` in metres resolves.
fn keep(k: f32, foot: f32) -> f32 {
    1.0 - smoothstep(RESOLVE.0, RESOLVE.1, foot * k / TAU)
}

impl Packet {
    /// Returns the height of the packet in metres at `(s, c)` at clock `now`, and its slope
    /// along `s` and `c`, as seen at a footprint of `foot` metres.
    ///
    /// The slopes follow the phase of the waves and leave out the slow change of their envelope.
    pub fn bit(&self, (s, c): (f32, f32), now: f32, foot: f32) -> Vec3 {
        let age = now - self.born;
        if !(0.0..LIFE).contains(&age) {
            return Vec3::ZERO;
        }
        let fade = (1.0 - age / LIFE).powi(2);
        (self.ring((s, c), age, foot) + self.transverse((s, c), age, foot)) * fade
    }

    /// Returns the part of the arc of the ring that is kept at `(s, c)`, `r` metres from the packet.
    pub(super) fn arc(&self, s: f32, r: f32) -> f32 {
        if self.speed <= MOVING {
            return 1.0;
        }
        let rear = (-s / r).clamp(-1.0, 1.0).acos();
        let meet = FRAC_PI_2 + (self.front / self.speed).clamp(0.0, 1.0).asin();
        (-((rear - meet) / ARC).powi(2)).exp()
    }

    /// Returns the ring waves at `(s, c)`, `age` seconds after they were laid.
    fn ring(&self, (s, c): (f32, f32), age: f32, foot: f32) -> Vec3 {
        let r = s.hypot(c);
        let front = self.front * age;
        let off = r - front;
        if r < 1e-3 || off.abs() > 3.0 * self.width {
            return Vec3::ZERO;
        }
        let spread = (1.0 + front / self.size.max(1.0)).sqrt().recip();
        let envelope = (-(off / self.width).powi(2)).exp() * spread;
        let height = self.amp * envelope * self.arc(s, r) * keep(self.k, foot);
        let phase = self.k * (r - self.crest * age);
        let slope = -height * self.k * phase.sin() / r;
        Vec3::new(height * phase.cos(), slope * s, slope * c)
    }

    /// Returns the transverse waves at `(s, c)`, `age` seconds after they were laid.
    fn transverse(&self, (s, c): (f32, f32), age: f32, foot: f32) -> Vec3 {
        if self.amp_t <= 0.0 {
            return Vec3::ZERO;
        }
        let behind = self.speed * age - s;
        let along = s - 0.5 * self.speed * age;
        let span = SPAN * self.speed;
        if along.abs() > 3.0 * span {
            return Vec3::ZERO;
        }
        let size = self.size.max(1.0);
        let reach = behind.max(0.0) * self.front / self.speed + 0.5 * self.size;
        let window = (-(along / span).powi(2)).exp() * (-(c / (0.6 * reach)).powi(2)).exp();
        let thin = smoothstep(0.0, size, behind) / (1.0 + behind.max(0.0) / (8.0 * size));
        let height = self.amp_t * window * thin * keep(self.k_t, foot);
        let phase = self.k_t * (s - self.speed * age);
        Vec3::new(height * phase.cos(), -height * self.k_t * phase.sin(), 0.0)
    }
}
