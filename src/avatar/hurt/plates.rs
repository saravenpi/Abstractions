//! The plates of the body: the box each part wears on each segment, in the frame of that segment.
//!
//! A blow lands on the part whose plate its contact point is nearest. Marks of damage sit on the
//! faces of these boxes.

use bevy::prelude::*;

use super::super::rig::{SEGS, Seg};
use super::super::tune::{BOOT, HEAD, HIPS};
use crate::condition::Part;

/// A box on a segment: its centre and half extents in the frame of the segment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plate {
    /// The part that wears it.
    pub part: Part,
    /// The segment it sits on.
    pub seg: Seg,
    /// The centre of the box.
    pub centre: Vec3,
    /// Half its extents along each axis.
    pub half: Vec3,
}

/// Returns the plate of `part` on `seg`, a box of half extents `half` centred at `centre`.
const fn plate(part: Part, seg: Seg, centre: [f32; 3], half: [f32; 3]) -> Plate {
    Plate {
        part,
        seg,
        centre: Vec3::from_array(centre),
        half: Vec3::from_array(half),
    }
}

/// Number of plates on the body.
pub const PLATES: usize = 17;

/// The box of a shoulder block, a gauntlet, a palm, the tassets round a thigh and a greave: its
/// centre and its half extents, written for the right side.
const LIMBS: [([f32; 3], [f32; 3]); 5] = [
    ([0.046, -0.05, 0.0], [0.042, 0.088, 0.08]),
    ([0.0, -0.14, 0.014], [0.05, 0.1, 0.074]),
    ([0.0, -0.045, 0.0], [0.024, 0.055, 0.034]),
    ([0.01, -0.14, 0.0], [0.07, 0.13, 0.07]),
    ([0.0, -0.195, 0.012], [0.075, 0.155, 0.09]),
];

/// Returns the plate of `part` on `seg` from the box `limb` of [`LIMBS`], mirrored for a
/// segment of the left side: the limbs go left then right, from an odd place in [`Seg::ALL`].
const fn limb(part: Part, seg: Seg, limb: usize) -> Plate {
    let (centre, half) = LIMBS[limb];
    let side = if seg as usize % 2 == 1 { -1.0 } else { 1.0 };
    plate(part, seg, [centre[0] * side, centre[1], centre[2]], half)
}

/// Every plate of the body: the helmet, the chest out to its breast plates, the pelvis, the two
/// halves of the pack the wings unfold from, and the armour of each limb.
pub const ALL: [Plate; PLATES] = [
    plate(Part::Head, Seg::Head, [0.0, HEAD.0, 0.0], HEAD.1),
    plate(
        Part::Torso,
        Seg::Torso,
        [0.0, 0.2, -0.008],
        [0.185, 0.2, 0.118],
    ),
    plate(Part::Torso, Seg::Pelvis, [0.0, 0.0, 0.0], HIPS),
    plate(
        Part::WingL,
        Seg::Torso,
        [-0.064, 0.225, 0.094],
        [0.064, 0.105, 0.03],
    ),
    plate(
        Part::WingR,
        Seg::Torso,
        [0.064, 0.225, 0.094],
        [0.064, 0.105, 0.03],
    ),
    limb(Part::ArmL, Seg::UpperArmL, 0),
    limb(Part::ArmL, Seg::ForearmL, 1),
    limb(Part::ArmL, Seg::HandL, 2),
    limb(Part::ArmR, Seg::UpperArmR, 0),
    limb(Part::ArmR, Seg::ForearmR, 1),
    limb(Part::ArmR, Seg::HandR, 2),
    limb(Part::LegL, Seg::ThighL, 3),
    limb(Part::LegL, Seg::ShinL, 4),
    limb(Part::LegR, Seg::ThighR, 3),
    limb(Part::LegR, Seg::ShinR, 4),
    plate(Part::BootL, Seg::BootL, BOOT.0, BOOT.1),
    plate(Part::BootR, Seg::BootR, BOOT.0, BOOT.1),
];

impl Plate {
    /// Returns the distance from `point`, in the frame of the segment, to the box: 0 inside it.
    pub fn distance(&self, point: Vec3) -> f32 {
        ((point - self.centre).abs() - self.half)
            .max(Vec3::ZERO)
            .length()
    }
}

/// Returns the plates of `part`.
pub fn of(part: Part) -> impl Iterator<Item = &'static Plate> {
    ALL.iter().filter(move |plate| plate.part == part)
}

/// Returns the part whose plate lies nearest to `point`, given in the frame of the body root, and
/// the distance to it, for a body whose segments sit at `joints` in that frame.
pub fn nearest(point: Vec3, joints: &[Transform; SEGS]) -> (Part, f32) {
    let gap = |plate: &Plate| {
        let inverse = joints[plate.seg as usize].compute_affine().inverse();
        plate.distance(inverse.transform_point3(point))
    };
    ALL.iter()
        .map(|plate| (plate.part, gap(plate)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap_or((Part::Torso, f32::INFINITY))
}
