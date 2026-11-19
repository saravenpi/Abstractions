//! The rest frame of a body: where each part sits when nothing moves, and the axis it lies along.

use bevy::math::{Quat, Vec3};

use super::frame::{Role, chains, roles};
use super::ik::unit;
use crate::life::body::{Body, Chain};

/// A position and a rotation.
pub type Place = (Vec3, Quat);

/// What a body is made of, worked out once.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    /// Rest place of every part in the body frame.
    pub rest: Vec<Place>,
    /// Role of every part.
    pub roles: Vec<Role>,
    /// Unit axis every part lies along in its own frame, from its socket toward the next joint.
    pub axes: Vec<Vec3>,
}

/// Returns `local` carried by `parent`: a child placed at its socket and rest rotation.
pub fn carry(parent: Place, local: Place) -> Place {
    (parent.0 + parent.1 * local.0, parent.1 * local.1)
}

/// Returns the rest place of every part of `body` in the body frame.
fn rest(body: &Body) -> Vec<Place> {
    let mut placed: Vec<Place> = Vec::with_capacity(body.parts.len());
    for part in &body.parts {
        let parent = part
            .parent
            .and_then(|p| placed.get(usize::from(p)).copied());
        placed.push(carry(
            parent.unwrap_or((Vec3::ZERO, Quat::IDENTITY)),
            (part.socket, part.rest),
        ));
    }
    placed
}

/// Returns the axis of every part of `body`: toward the socket of the next part of its leg or
/// chain when that part hangs from it, and down its own -Y otherwise.
fn axes(body: &Body) -> Vec<Vec3> {
    let mut axes = vec![Vec3::NEG_Y; body.parts.len()];
    let runs = body
        .legs
        .iter()
        .map(|l| &l.bones)
        .chain(chains(body).into_iter().map(|(_, c)| &c.parts));
    for run in runs {
        for pair in run.windows(2) {
            let (from, to) = (usize::from(pair[0]), usize::from(pair[1]));
            let Some(next) = body.parts.get(to).filter(|p| p.parent == Some(pair[0])) else {
                continue;
            };
            if let Some(axis) = axes.get_mut(from) {
                *axis = unit(next.socket, Vec3::NEG_Y);
            }
        }
    }
    axes
}

impl Frame {
    /// Returns the frame of `body`.
    pub fn of(body: &Body) -> Self {
        Self {
            rest: rest(body),
            roles: roles(body),
            axes: axes(body),
        }
    }

    /// Returns the joints of `chain` at rest under `root`: each part's socket, then its tip.
    pub fn joints(&self, chain: &Chain, root: Place) -> Vec<Vec3> {
        let mut points: Vec<Vec3> = Vec::with_capacity(chain.lengths.len() + 1);
        let mut last = (root.0, root.1 * Vec3::NEG_Y, 0.0);
        for (i, &length) in chain.lengths.iter().enumerate() {
            let part = chain.parts.get(i).map(|&p| usize::from(p));
            let at = match part.and_then(|p| self.rest.get(p).map(|&place| (p, place))) {
                Some((p, place)) => {
                    let (at, turn) = carry(root, place);
                    last.1 = turn * self.axes[p];
                    at
                }
                None => last.0 + last.1 * last.2,
            };
            last = (at, last.1, length);
            points.push(at);
        }
        points.push(last.0 + last.1 * last.2);
        points
    }
}
