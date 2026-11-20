//! Sparks: a burst where a blow lands, and a trickle from a part that is nearly gone.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::super::animate::Anim;
use super::super::blow::Blow;
use super::super::own::Own;
use super::plates;
use super::scar::Scars;
use super::tune::{
    DRIZZLE_ODDS, SPARK_DRAG, SPARK_JOULES, SPARK_LIFE, SPARK_SIZE, SPARK_SPEED, SPARKING,
};
use crate::condition::{Condition, Part, luck};

/// One spark, flying free of the body in render space.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Spark {
    /// Where it is going, in metres per second.
    pub velocity: Vec3,
    /// Seconds it still lives.
    pub left: f32,
}

/// Spawns `count` sparks at `at`, thrown out in the directions the roll `key` picks.
fn throw(commands: &mut Commands, scars: &Scars, (at, key): (Vec3, u32), count: usize) {
    for spark in 0..count as u32 {
        let roll = |n: u32| luck(key.wrapping_mul(31).wrapping_add(spark * 7 + n)) * 2.0 - 1.0;
        let way = Vec3::new(roll(1), roll(2), roll(3)).normalize_or(Vec3::Y);
        commands.spawn((
            Spark {
                velocity: way * SPARK_SPEED * (0.5 + luck(key ^ spark)),
                left: SPARK_LIFE,
            },
            Mesh3d(scars.cube.clone()),
            MeshMaterial3d(scars.spark.clone()),
            Transform::from_translation(at).with_scale(Vec3::splat(SPARK_SIZE)),
            Visibility::default(),
            NotShadowCaster,
        ));
    }
}

/// Throws a burst of sparks from every blow that lands, more for a harder one.
pub fn burst(
    mut commands: Commands,
    mut blows: MessageReader<Blow>,
    scars: Res<Scars>,
    time: Res<Time>,
) {
    for blow in blows.read() {
        let count = (3.0 + blow.joules / SPARK_JOULES).min(14.0) as usize;
        let key = (time.elapsed_secs() * 1000.0) as u32;
        throw(&mut commands, &scars, (blow.at, key), count);
    }
}

/// Throws a spark now and then from the plates of every part hurt below [`SPARKING`].
pub fn drizzle(
    mut commands: Commands,
    (condition, scars, time): (Res<Condition>, Res<Scars>, Res<Time>),
    (own, bodies): (Own, Query<(&GlobalTransform, &Anim)>),
) {
    let Some((placed, anim)) = own.body().and_then(|(_, body)| bodies.get(body).ok()) else {
        return;
    };
    let tick = (time.elapsed_secs() * 4.0) as u32;
    let joints = super::super::rig::globals(&anim.pose);
    for part in Part::ALL {
        let weak = 1.0 - condition.of(part) / SPARKING;
        let key = tick.wrapping_mul(13).wrapping_add(part as u32);
        if weak <= 0.0 || luck(key) > weak * DRIZZLE_ODDS {
            continue;
        }
        if let Some(plate) = plates::of(part).next() {
            let local = joints[plate.seg as usize].transform_point(plate.centre);
            throw(
                &mut commands,
                &scars,
                (placed.transform_point(local), key),
                1,
            );
        }
    }
}

/// Flies every spark on, slowing and shrinking it, and removes it once spent.
pub fn fly(
    mut commands: Commands,
    time: Res<Time>,
    mut sparks: Query<(Entity, &mut Spark, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut spark, mut at) in &mut sparks {
        spark.left -= dt;
        if spark.left <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let drag = (-SPARK_DRAG * dt).exp();
        spark.velocity *= drag;
        at.translation += spark.velocity * dt;
        at.scale = Vec3::splat(SPARK_SIZE * spark.left / SPARK_LIFE);
    }
}
