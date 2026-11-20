//! The plates of the body: the box each part wears on each segment, in the frame of that segment.
//!
//! A blow lands on the part whose plate its contact point is nearest. Marks of damage sit on the
//! faces of these boxes.

use bevy::prelude::*;

use super::super::rig::{SEGS, Seg};
use super::super::tune::{BOOT, CHEST, HEAD, HIPS};
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

/// Every plate of the body. The wings lie on the back of the torso, behind its plate.
pub const ALL: [Plate; PLATES] = [
    plate(Part::Head, Seg::Head, [0.0, HEAD.0, 0.0], HEAD.1),
    plate(Part::Torso, Seg::Torso, [0.0, CHEST.0, 0.0], CHEST.1),
    plate(Part::Torso, Seg::Pelvis, [0.0, 0.0, 0.0], HIPS),
    plate(
        Part::WingL,
        Seg::Torso,
        [-0.12, 0.2, 0.14],
        [0.08, 0.17, 0.03],
    ),
    plate(
        Part::WingR,
        Seg::Torso,
        [0.12, 0.2, 0.14],
        [0.08, 0.17, 0.03],
    ),
    plate(
        Part::ArmL,
        Seg::UpperArmL,
        [0.0, -0.115, 0.0],
        [0.05, 0.115, 0.05],
    ),
    plate(
        Part::ArmL,
        Seg::ForearmL,
        [0.0, -0.105, 0.0],
        [0.045, 0.105, 0.045],
    ),
    plate(
        Part::ArmL,
        Seg::HandL,
        [0.0, -0.05, 0.0],
        [0.04, 0.05, 0.04],
    ),
    plate(
        Part::ArmR,
        Seg::UpperArmR,
        [0.0, -0.115, 0.0],
        [0.05, 0.115, 0.05],
    ),
    plate(
        Part::ArmR,
        Seg::ForearmR,
        [0.0, -0.105, 0.0],
        [0.045, 0.105, 0.045],
    ),
    plate(
        Part::ArmR,
        Seg::HandR,
        [0.0, -0.05, 0.0],
        [0.04, 0.05, 0.04],
    ),
    plate(
        Part::LegL,
        Seg::ThighL,
        [0.0, -0.18, 0.0],
        [0.06, 0.18, 0.06],
    ),
    plate(
        Part::LegL,
        Seg::ShinL,
        [0.0, -0.18, 0.0],
        [0.052, 0.18, 0.052],
    ),
    plate(
        Part::LegR,
        Seg::ThighR,
        [0.0, -0.18, 0.0],
        [0.06, 0.18, 0.06],
    ),
    plate(
        Part::LegR,
        Seg::ShinR,
        [0.0, -0.18, 0.0],
        [0.052, 0.18, 0.052],
    ),
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
