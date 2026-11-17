//! The render anchor of a flying ship: where its origin sits and how its axes carry over a
//! change of frame, so that nothing on screen turns when the ship crosses the atmosphere top.

use bevy::math::{DQuat, DVec3};

use crate::space::{Anchor, Frame, rebased};
use crate::universe::PlanetSpec;
use crate::world::Link;

/// Returns the rotation from the axes of `frame` to the render axes `anchor` already has, or
/// nothing when the two frames are not joined: the same frame, the surface of `land` and the
/// frame of its star at `time` seconds, or a star and a galaxy, whose axes are the same.
fn axes(anchor: &Anchor, frame: Frame, land: Option<&PlanetSpec>, time: f64) -> Option<DQuat> {
    let turn = |body| {
        land.filter(|planet| planet.id == body)
            .map(|planet| Link::orbiting(planet, time).turn)
    };
    match (anchor.frame, frame) {
        _ if anchor.flat => None,
        (old, new) if old == new => Some(anchor.rotation),
        (Frame::Surface(body), Frame::System(star)) if body.star == star => {
            turn(body).map(|turn| anchor.rotation * turn.inverse())
        }
        (Frame::System(star), Frame::Surface(body)) if body.star == star => {
            turn(body).map(|turn| anchor.rotation * turn)
        }
        (Frame::Surface(_), _) | (_, Frame::Surface(_)) => None,
        _ => Some(anchor.rotation),
    }
}

/// Returns the anchor for a ship drawn at `pos` in `frame`, given the anchor of the frame before.
///
/// Away from a surface the origin is the ship. In the surface frame of `land` the origin is a
/// sea level point kept until the ship is too far from it, as on foot. The render axes are
/// carried over from `anchor` whenever its frame joins the new one, and on a surface they are
/// then turned upright only once the local vertical has left render +Y by the rebase limit.
/// Without an anchor to carry, a surface gets the tangent frame under the ship and space gets
/// the axes of its frame.
pub fn anchor_for(
    anchor: &Anchor,
    (frame, pos): (Frame, DVec3),
    land: Option<&PlanetSpec>,
    time: f64,
) -> Anchor {
    let kept = axes(anchor, frame, land, time);
    let radius = match frame {
        Frame::Surface(body) => land.filter(|planet| planet.id == body).map(|p| p.radius),
        _ => None,
    };
    let Some(radius) = radius else {
        return Anchor {
            frame,
            origin: pos,
            rotation: kept.unwrap_or(DQuat::IDENTITY),
            flat: false,
        };
    };
    let up = pos.normalize_or(DVec3::Y);
    let under = up * radius;
    let held = Anchor {
        frame,
        origin: if anchor.frame == frame {
            anchor.origin
        } else {
            under
        },
        rotation: kept.unwrap_or_else(|| DQuat::from_rotation_arc(up, DVec3::Y)),
        flat: false,
    };
    rebased(&held, radius, held.to_render(pos)).unwrap_or(held)
}
