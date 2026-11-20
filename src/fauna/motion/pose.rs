//! The pose of a body: every part's place, rotation and scale in the local frame.
//!
//! Parts are placed parents first. A part hangs from where its parent is now, then the motion
//! that owns it moves it: legs reach their feet, chain parts lie along their strands, the head
//! looks, wings flap, antennae flick, and the torso breathes. A calling body lifts its head, drops
//! its jaw and pumps its chest on every note, and rises for a threat display.

use bevy::math::{Quat, Vec2, Vec3};

use super::frame::Role;
use super::ik::{three_bone, two_bone, unit};
use super::look::tilt;
use super::rest::{Place, carry};
use super::rig::Motion;
use super::song::{JAW, Song};
use crate::life::body::{Body, Knee, Locomotion};

/// A part's position, rotation and scale in the local frame.
pub type Placed = (Vec3, Quat, Vec3);

/// Legs whose solved joints a body keeps on the stack while it is posed; a body with more uses the
/// heap.
const ON_STACK: usize = 16;

/// Returns `turn` swung so that its `axis` points along `dir`.
fn aimed(turn: Quat, axis: Vec3, dir: Vec3) -> Quat {
    let from = turn * axis;
    let to = unit(dir, from);
    Quat::from_rotation_arc(from, to) * turn
}

impl Motion {
    /// Returns the place, rotation and scale of every part of `body`, in the order of its parts,
    /// all in the local frame. `body` must be the body the motion was made for.
    pub fn pose(&self, body: &Body) -> Vec<Placed> {
        let raise = self.sung().map_or(0.0, |s| s.raise(body.height));
        let root = (self.root.place + Vec3::Y * raise, self.root.turn);
        let mut placed: Vec<Placed> = Vec::with_capacity(body.parts.len());
        let mut stack = [[Vec3::ZERO; 4]; ON_STACK];
        let mut heap = Vec::new();
        let limbs = match body.legs.len() {
            n if n <= ON_STACK => &mut stack[..n],
            n => {
                heap.resize(n, [Vec3::ZERO; 4]);
                &mut heap[..]
            }
        };
        for (i, part) in body.parts.iter().enumerate() {
            let parent = part.parent.and_then(|p| placed.get(usize::from(p)));
            let base = carry(
                parent.map_or(root, |&(at, turn, _)| (at, turn)),
                (part.socket, part.rest),
            );
            placed.push(self.place(body, i, base, limbs));
        }
        placed
    }

    /// Returns part `i` of `body` moved by its role from where its parent carries it, `base`.
    fn place(&self, body: &Body, i: usize, base: Place, limbs: &mut [[Vec3; 4]]) -> Placed {
        let (at, turn) = base;
        let breath = 0.015 * self.idle.breath();
        let song = self.sung();
        match self.frame.roles[i] {
            Role::Torso => {
                let chest = song.map_or(Vec3::ONE, Song::chest);
                (at, turn, Vec3::new(1.0 + breath, 1.0 + breath, 1.0) * chest)
            }
            Role::Squash => (
                at,
                turn,
                self.beat.squash * song.map_or(Vec3::ONE, Song::squash),
            ),
            Role::Bone(leg, bone) if body.locomotion == Locomotion::Walk => {
                self.bone(body, (i, usize::from(leg), usize::from(bone)), base, limbs)
            }
            Role::Link(strand, joint) => self.link(
                (usize::from(strand), usize::from(joint)),
                base,
                self.frame.axes[i],
            ),
            role => (at, self.swing(role) * turn, Vec3::ONE),
        }
    }

    /// Returns the turn a part of `role` gets on top of its rest: the head looks and reaches as it
    /// calls, the jaw drops, wings flap and an antenna flicks when its side's timer fires.
    fn swing(&self, role: Role) -> Quat {
        let turn = self.root.turn;
        let song = self.sung();
        match role {
            Role::Head => tilt(self.look.at + Vec2::Y * song.map_or(0.0, Song::pitch), turn),
            Role::Jaw => Quat::from_axis_angle(turn * Vec3::X, -JAW * song.map_or(0.0, Song::open)),
            Role::Wing(side) => {
                Quat::from_axis_angle(turn * Vec3::Z, self.beat.wing * f32::from(side))
            }
            Role::Antenna(side) => {
                let (angle, flicked) = self.idle.flick();
                let shown = if side == 0 || side == flicked {
                    angle
                } else {
                    0.0
                };
                Quat::from_axis_angle(turn * Vec3::X, -shown)
            }
            _ => Quat::IDENTITY,
        }
    }

    /// Returns the call the body is making now, if any.
    fn sung(&self) -> Option<&Song> {
        self.song.as_ref().filter(|s| s.live())
    }

    /// Returns part `i`, bone `bone` of leg `leg` of `body`, laid between the joints its leg solver
    /// gives; the first bone of a leg solves the leg into `limbs` from where `base` puts its hip.
    fn bone(
        &self,
        body: &Body,
        (i, leg, bone): (usize, usize, usize),
        base: Place,
        limbs: &mut [[Vec3; 4]],
    ) -> Placed {
        if bone == 0 {
            limbs[leg] = self.limb(body, leg, base.0);
        }
        let (from, to) = (limbs[leg][bone.min(3)], limbs[leg][(bone + 1).min(3)]);
        (
            from,
            aimed(base.1, self.frame.axes[i], to - from),
            Vec3::ONE,
        )
    }

    /// Returns the hip, knee, ankle and foot of leg `i` of `body` from `hip` to its foot, the last
    /// joints repeated for a shorter leg.
    fn limb(&self, body: &Body, i: usize, hip: Vec3) -> [Vec3; 4] {
        let leg = &body.legs[i];
        let foot = self.feet.get(i).map_or(hip, |f| f.at);
        let turn = self.root.turn;
        let reach: f32 = leg.lengths.iter().sum();
        let bend = match leg.knee {
            Knee::Forward => turn * Vec3::NEG_Z,
            Knee::Back => turn * Vec3::Z,
            Knee::Out => turn * Vec3::new(f32::from(leg.side), 1.0, 0.0),
        };
        let pole = hip + bend * reach;
        match *leg.lengths.as_slice() {
            [a, b] => {
                let [knee, end] = two_bone(hip, foot, (a, b), pole);
                [hip, knee, end, end]
            }
            [a, b, c, ..] => {
                let last = leg.bones.get(2).map_or(0, |&b| usize::from(b));
                let held = turn * (self.frame.rest[last].1 * self.frame.axes[last]);
                let [knee, ankle, end] = three_bone(hip, foot, [a, b, c], held, pole);
                [hip, knee, ankle, end]
            }
            _ => {
                let end = hip + unit(foot - hip, Vec3::NEG_Y) * reach;
                [hip, end, end, end]
            }
        }
    }

    /// Returns joint `joint` of strand `strand` laid along its chain, from `base` with its `axis`.
    fn link(&self, (strand, joint): (usize, usize), base: Place, axis: Vec3) -> Placed {
        let points = self.strands.get(strand).map_or(&[][..], |s| &s.points[..]);
        match (points.get(joint), points.get(joint + 1)) {
            (Some(&at), Some(&next)) => (at, aimed(base.1, axis, next - at), Vec3::ONE),
            _ => (base.0, base.1, Vec3::ONE),
        }
    }
}
