//! Tests of the spears that stay the robot's: where a tool that is out is noted, and how it comes
//! back into the world when its spear is gone.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::flight::{Rest, Spear};
use super::stage::{earth, spear};
use super::stray::{above, fallen, note};
use super::tune::{STRAY_LIFT, STRAY_NEAR};
use crate::craft::matter::{Class, Mineral};
use crate::craft::pack::Item;
use crate::craft::{Out, Pack};
use crate::space::{Anchor, Frame};
use crate::universe::BodyId;

/// Metres from the centre of the test planet to its ground.
const RADIUS: f64 = 3000.0;

/// Returns an anchor on the surface of `body` whose render origin is the ground under the pole.
fn anchor(body: BodyId) -> Anchor {
    Anchor {
        frame: Frame::Surface(body),
        origin: DVec3::Y * RADIUS,
        rotation: DQuat::IDENTITY,
        flat: false,
    }
}

/// Returns the thrown spear of the test stage as a tool of the pack with `id`, out in the world,
/// resting on `body` at the render position `at` of `anchor`.
fn out(id: u32, rests: Option<(BodyId, &Anchor, Vec3)>) -> Out {
    let held = spear();
    let materials = vec![
        Class::Wood.anchor(),
        Mineral::Flint.material(),
        Class::Hide.anchor(),
    ];
    Out {
        item: Item {
            id,
            recipe: held.recipe,
            materials,
        },
        slot: Some(2),
        body: rests.map(|(body, ..)| body),
        at: rests.map_or([0.0; 3], |(_, anchor, at)| anchor.to_frame(at).to_array()),
    }
}

/// A tool that is out comes back above the place it rested at when the robot is near it on the same
/// planet; far away or on another planet it waits; one that never came to rest comes back at the
/// robot.
#[test]
fn a_lost_spear_comes_back_where_it_rested() {
    let body = BodyId::default();
    let anchor = anchor(body);
    let rested = Vec3::new(20.0, 0.0, 5.0);
    let record = out(4, Some((body, &anchor, rested)));
    let near = above(&record, &anchor, body, Vec3::ZERO).expect("near enough");
    assert!(
        near.distance(rested + Vec3::Y * STRAY_LIFT) < 0.05,
        "{near}"
    );
    let far = Vec3::X * (STRAY_NEAR + 30.0);
    assert_eq!(above(&record, &anchor, body, -far), None);
    let other = BodyId {
        index: body.index + 1,
        ..body
    };
    assert_eq!(above(&record, &anchor, other, Vec3::ZERO), None);
    let robot = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(above(&out(4, None), &anchor, body, robot), Some(robot));
}

/// The spear put back in the world is the same tool: it falls from rest, carries the item of the
/// pack and its hotbar slot, and does not hit the body it belongs to.
#[test]
fn the_spear_put_back_is_the_same_tool() {
    let record = out(9, None);
    let (player, body) = (
        Entity::from_raw_u32(7).unwrap(),
        Entity::from_raw_u32(8).unwrap(),
    );
    let back = fallen(&record, earth(), (player, body), Vec3::ONE).expect("it builds");
    assert_eq!((back.rest, back.velocity), (Rest::Flying, Vec3::ZERO));
    assert_eq!((back.thrower, back.body, back.item.id), (player, body, 9));
    assert_eq!(back.lent, Some((record.item.clone(), Some(2))));
}

/// A spear at rest is noted in the pack where it stands, in the frame of the planet, and a spear
/// that has not moved is not noted again: the pack stays unchanged, so it is not saved each frame.
#[test]
fn a_resting_spear_is_noted_in_the_pack() {
    let body = BodyId::default();
    let record = out(3, None);
    let mut pack = Pack::default();
    pack.out.push(record.clone());
    let mut app = App::new();
    app.insert_resource(anchor(body))
        .insert_resource(pack)
        .add_systems(Update, note);
    let at = Vec3::new(4.0, 1.0, -6.0);
    let (player, robot) = (
        Entity::from_raw_u32(7).unwrap(),
        Entity::from_raw_u32(8).unwrap(),
    );
    let stuck = Spear {
        rest: Rest::Stuck,
        ..fallen(&record, earth(), (player, robot), at).expect("it builds")
    };
    app.world_mut()
        .spawn((stuck, GlobalTransform::from_translation(at)));
    app.update();
    let noted = app.world().resource::<Pack>().out[0].clone();
    assert_eq!(noted.body, Some(body));
    assert!(DVec3::from(noted.at).distance(DVec3::new(4.0, RADIUS + 1.0, -6.0)) < 1e-3);
    app.update();
    app.update();
    assert!(!app.world().resource_ref::<Pack>().is_changed());
}
