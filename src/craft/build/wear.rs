//! Wear: how many blows each joint of a build takes, and which one gives first.
//!
//! Each joint has a safety margin, what it holds over what a blow asks of it. Its life in blows is
//! margin^6, the steep curve of fatigue (Basquin's law, https://en.wikipedia.org/wiki/Fatigue_(material)):
//! a joint at margin 1 breaks on the first blow, one at margin 2 lasts about 64.

use std::f32::consts::PI;

use serde::{Deserialize, Serialize};

use super::Build;
use super::swing::Swing;
use super::tune::{BIND_SHARE, BIND_SHEAR, CONTACT, FIT_SHEAR, SOCKET, SWING_ARC, WEAR_POWER};
use crate::craft::matter::Material;

/// A place where a build can fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Joint {
    /// The binding that holds the head on the shaft.
    Head,
    /// The binding that holds the counterweight on the butt.
    Counterweight,
    /// The edge or point itself, chipping.
    Edge,
    /// The shaft, bending at the hand.
    Shaft,
}

impl Joint {
    /// Every joint, in the order of [`lives`] and [`Wear`].
    pub const ALL: [Joint; 4] = [Joint::Head, Joint::Counterweight, Joint::Edge, Joint::Shaft];
}

/// Returns the shear strength of a socket bound with `binding`, in megapascals: a share of the binding's
/// strength within 5 to 20 MPa, or a bare friction fit.
fn shear(binding: Option<Material>) -> f32 {
    binding.map_or(FIT_SHEAR, |m| {
        (m.strength * BIND_SHARE).clamp(BIND_SHEAR[0], BIND_SHEAR[1])
    })
}

/// Returns the safety margins of each joint of `build` under `swing`, in [`Joint::ALL`] order.
///
/// A blow stops the head in [`CONTACT`] seconds, so the head socket takes momentum / time against its
/// shear area π·d·depth. The counterweight socket takes the pull m·ω²·r of the spin. An edge holds
/// while its K_IC stays over the brittle line. The shaft bends at the hand under the moment I·α that
/// spins it up, α = ω² / 2·arc, against its strength over the section modulus π·d³/32.
fn margins(build: &Build, swing: &Swing) -> [f32; 4] {
    let d = build.shaft.diameter;
    let socket = shear(build.binding) * 1.0e6 * PI * d * SOCKET;
    let pull = build
        .counterweight
        .map_or(0.0, |w| w.mass * swing.spin * swing.spin * build.grip);
    let bending = swing.inertia * swing.spin * swing.spin / (2.0 * SWING_ARC);
    let stress = 32.0 * bending / (PI * d * d * d);
    [
        socket / (swing.momentum / CONTACT).max(f32::EPSILON),
        socket / pull.max(f32::EPSILON),
        build.head.material.toughness / Material::BRITTLE,
        build.shaft.material.strength * 1.0e6 / stress.max(f32::EPSILON),
    ]
}

/// Returns how many blows each joint of `build` lasts under `swing`, in [`Joint::ALL`] order.
pub fn lives(build: &Build, swing: &Swing) -> [f32; 4] {
    margins(build, swing).map(|m| m.powf(WEAR_POWER))
}

/// Blows each joint has taken since it was last made or replaced, in [`Joint::ALL`] order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Wear(pub [u16; 4]);

impl Wear {
    /// Counts one blow on every joint.
    pub fn hit(&mut self) {
        self.0 = self.0.map(|n| n.saturating_add(1));
    }

    /// Replaces the piece at `joint`: its count starts again.
    pub fn repair(&mut self, joint: Joint) {
        self.0[joint as usize] = 0;
    }

    /// Returns the joint that has broken given each joint's `lives`, the one furthest past its life.
    pub fn broken(&self, lives: &[f32; 4]) -> Option<Joint> {
        Joint::ALL
            .into_iter()
            .filter(|&j| f32::from(self.0[j as usize]) >= lives[j as usize])
            .max_by(|&a, &b| {
                let over =
                    |j: Joint| f32::from(self.0[j as usize]) / lives[j as usize].max(f32::EPSILON);
                over(a).total_cmp(&over(b))
            })
    }

    /// Returns the joint that will break first given `lives`: the least blows left.
    pub fn weakest(&self, lives: &[f32; 4]) -> Joint {
        Joint::ALL
            .into_iter()
            .min_by(|&a, &b| {
                let left = |j: Joint| lives[j as usize] - f32::from(self.0[j as usize]);
                left(a).total_cmp(&left(b))
            })
            .unwrap_or(Joint::Head)
    }
}
