//! Writes the exhaust of every ship onto what is drawn: the uniform of its three materials from
//! the shape the thrust and the air give, and the length and width of every plume.

use bevy::prelude::*;

use super::jets::{Jet, Kind, Plume};
use super::plume::{ExhaustMaterial, JetUniform};
use super::shape::{Shape, shape, warp};
use super::state::Exhaust;
use super::tune::{OFF, RCS_LENGTH, SCROLL};
use crate::ship::mesh::{EXIT, LIFT};

/// Returns the look of the plumes of `kind` of a ship: for a main engine the warp plume or the
/// thrust plume, whichever is longer, for a lift nozzle its thrust plume, and for the attitude
/// jets, which differ from port to port, only whether any of them fires.
pub(super) fn look(kind: Kind, exhaust: &Exhaust) -> Shape {
    match kind {
        Kind::Main => {
            let thrust = shape(EXIT, exhaust.main, exhaust.air);
            let streak = warp(EXIT, exhaust.warp);
            if streak.length > thrust.length {
                streak
            } else {
                thrust
            }
        }
        Kind::Lift => shape(LIFT.1, exhaust.lift, exhaust.air),
        Kind::Rcs => Shape {
            bright: f32::from(u8::from(exhaust.rcs.abs().max_element() > OFF)),
            scroll: SCROLL,
            ..Shape::default()
        },
    }
}

/// Returns the shape of the plume of `jet`: the look of its kind, except for an attitude jet,
/// which is as long as the fire of its own port.
fn form(jet: &Jet, exhaust: &Exhaust) -> Shape {
    let Some(port) = jet.port else {
        return look(jet.kind, exhaust);
    };
    let fire = (exhaust.rcs[port.axis] * port.sign).clamp(0.0, 1.0);
    Shape {
        length: RCS_LENGTH * fire,
        width: jet.exit * (1.0 + 0.6 * fire),
        ..look(Kind::Rcs, exhaust)
    }
}

/// Returns the uniform of the material of `kind`: the shape of its plumes and its colours. The
/// core of a main plume is the linear colour of the chamber face at full throttle, and the core
/// of a lift plume is close to it: both are far over the bloom threshold of 2.5 at full thrust
/// in air, so the fire blooms against a day sky.
fn uniform(kind: Kind, shape: Shape, exhaust: &Exhaust) -> JetUniform {
    let (hot, cool) = match kind {
        Kind::Main => (Vec3::new(2.2, 5.6, 9.5), Vec3::new(0.35, 0.8, 2.4)),
        Kind::Lift => (Vec3::new(3.2, 4.8, 7.0), Vec3::new(0.5, 0.9, 2.0)),
        Kind::Rcs => (Vec3::new(1.2, 1.6, 2.4), Vec3::new(0.4, 0.6, 1.2)),
    };
    let warp = if kind == Kind::Main {
        exhaust.warp
    } else {
        0.0
    };
    JetUniform {
        drive: Vec4::new(shape.bright, shape.spread, warp, 0.0),
        hot: hot.extend(shape.bright),
        cool: cool.extend(shape.scroll),
        form: Vec4::new(shape.length, shape.shock, 0.0, 0.0),
    }
}

/// Sets the three materials of every ship from its exhaust, when they change.
pub fn paint(ships: Query<&Exhaust>, mut assets: ResMut<Assets<ExhaustMaterial>>) {
    for exhaust in &ships {
        let kinds = [Kind::Main, Kind::Lift, Kind::Rcs];
        for (kind, handle) in kinds.into_iter().zip(&exhaust.jets) {
            let next = uniform(kind, look(kind, exhaust), exhaust);
            let Some(made) = assets.get(handle) else {
                continue;
            };
            if made.jet == next {
                continue;
            }
            if let Some(mut made) = assets.get_mut(handle) {
                made.jet = next;
            }
        }
    }
}

/// Stretches the plumes of every ship to the length and the width its exhaust gives them, and
/// hides the ones that are dead.
pub fn stretch(
    ships: Query<&Exhaust>,
    mut plumes: Query<(&ChildOf, &Plume, &mut Transform, &mut Visibility)>,
) {
    for (parent, Plume(jet), mut at, mut visibility) in &mut plumes {
        let Ok(exhaust) = ships.get(parent.parent()) else {
            continue;
        };
        let look = form(jet, exhaust);
        let scale = Vec3::new(look.width, look.width, look.length);
        if at.scale != scale {
            at.scale = scale;
        }
        visibility.set_if_neq(if look.length > 0.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}
