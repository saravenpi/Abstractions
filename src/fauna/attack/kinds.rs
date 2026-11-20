//! The attacks a body has, read from its parts: horns charge, a tail sweeps behind, a long neck
//! lunges, clawed forelimbs swipe, jaws bite and a heavy body tramples.

use bevy::math::Vec2;

use super::anatomy::{claws, mouth, neck, tail};
use super::limbs::Broken;
use super::tune::{
    CHARGE_CONE, CHARGE_RANGE, CHARGE_ROOM, LUNGE_NECK, MOUTH, SHARE, SLACK, TRAMPLE_KG,
};
use crate::craft::matter::Class;
use crate::life::Species;
use crate::life::body::{Body, FeatureKind, Locomotion, PartKind};

/// One way an animal hurts what is near it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// A run at the target with the horns first.
    Charge,
    /// The tail swept at what is behind.
    Sweep,
    /// The head thrown out on a long neck.
    Lunge,
    /// The forelimb swung at the target.
    Swipe,
    /// A bite.
    Bite,
    /// The body brought down on the target.
    Trample,
}

impl Kind {
    /// The kinds in the order an animal tries them, the closest first.
    pub const ORDER: [Kind; 6] = [
        Kind::Trample,
        Kind::Sweep,
        Kind::Swipe,
        Kind::Lunge,
        Kind::Bite,
        Kind::Charge,
    ];

    /// Returns true when the body of `species` has this attack, its parts broken as `broken`.
    pub fn can(self, species: &Species, broken: Broken) -> bool {
        let body = &species.body;
        let has = |kind: FeatureKind| body.features.iter().any(|f| f.kind == kind);
        let walks = body.locomotion == Locomotion::Walk && broken.lame(body) == 0.0;
        match self {
            Kind::Charge => walks && has(FeatureKind::Horn) && broken.whole(body, PartKind::Horn),
            Kind::Sweep => has(FeatureKind::Tail) && broken.whole(body, PartKind::Tail),
            Kind::Lunge => neck(body) > LUNGE_NECK * body.length && mouth(body, broken),
            Kind::Swipe => claws(species) && broken.lame(body) == 0.0,
            Kind::Bite => mouth(body, broken),
            Kind::Trample => walks && species.history.mass > TRAMPLE_KG,
        }
    }

    /// Returns the metres from the middle of the body of `body` the attack reaches.
    pub fn reach(self, body: &Body) -> f32 {
        let half = 0.5 * body.length;
        match self {
            Kind::Bite => half + MOUTH,
            Kind::Lunge => half + neck(body) + MOUTH,
            Kind::Swipe => half + body.height + MOUTH,
            Kind::Sweep => half + tail(body),
            Kind::Charge | Kind::Trample => 0.6 * body.length + 0.8,
        }
    }

    /// Returns true when it carries the blow on an edge or a point: teeth, claws, horns.
    pub fn cuts(self) -> bool {
        matches!(self, Kind::Charge | Kind::Lunge | Kind::Swipe | Kind::Bite)
    }

    /// Returns true for the attacks that run: the body itself goes for the target.
    pub fn dashes(self) -> bool {
        matches!(self, Kind::Charge | Kind::Trample)
    }

    /// Returns the class of the material of the part that strikes: teeth and claws are bone, horns
    /// keratin, a hoof keratin, a tail hide.
    pub fn striker(self) -> Class {
        match self {
            Kind::Charge | Kind::Trample => Class::Keratin,
            Kind::Lunge | Kind::Swipe | Kind::Bite => Class::Bone,
            Kind::Sweep => Class::Hide,
        }
    }

    /// Returns the share of the mass of the body that carries the blow.
    pub fn share(self) -> f32 {
        SHARE[self as usize]
    }

    /// Returns true when the attack can take on a target `gap` metres off in the frame of the body
    /// whose way is the unit `facing`, in `body`.
    pub fn fits(self, body: &Body, (gap, facing): (Vec2, Vec2)) -> bool {
        let (dist, cos) = (gap.length(), facing.dot(gap.normalize_or_zero()));
        match self {
            Kind::Sweep => cos < -0.5 && dist <= self.reach(body) * SLACK,
            Kind::Trample => dist <= self.reach(body) * SLACK,
            Kind::Charge => {
                let near = CHARGE_ROOM * body.length;
                dist >= near && dist <= CHARGE_RANGE + body.length && cos > CHARGE_CONE.cos()
            }
            _ => cos > 0.5 && dist <= self.reach(body) * SLACK,
        }
    }
}
