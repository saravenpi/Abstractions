//! The systems of the dust: raise puffs where the jets of every ship meet loose ground, then fly
//! the puffs in the air, in the frame of the planet.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::super::Exhaust;
use super::Cloud;
use super::jet::{Hit, impinge, jets};
use super::look::recolour;
use super::puff::{Mote, fling, pose, reread, step};
use super::soil::{loose, settle};
use super::tune::{OWED, RATE, REREAD, THIN};
use crate::avatar::Soil;
use crate::home::Base;
use crate::render::GameCamera;
use crate::space::{Anchor, Frame, Placed};
use crate::wind::Wind;
use crate::world::Terrain;

/// Longest frame the dust simulates, in seconds, to survive hitches.
const MAX_DT: f32 = 0.1;

/// The world the dust flies in.
#[derive(SystemParam)]
pub struct Field<'w> {
    /// Frame time.
    time: Res<'w, Time>,
    /// The render anchor and the frame of the planet.
    anchor: Res<'w, Anchor>,
    /// The ground of the planet.
    terrain: Res<'w, Terrain>,
    /// The wind over it.
    wind: Res<'w, Wind>,
    /// The home base, whose landing pad gives no dust.
    base: Option<Res<'w, Base>>,
}

impl Field<'_> {
    /// Returns true when the anchor sits on the round surface of a planet, whose frame has its
    /// centre at the origin.
    fn round(&self) -> bool {
        !self.anchor.flat && matches!(self.anchor.frame, Frame::Surface(_))
    }

    /// Returns the distance from the centre of the planet of the landing pad over `at`, the
    /// planet position of a point of the ground, when there is one.
    fn pad(&self, at: DVec3) -> Option<f64> {
        let Frame::Surface(body) = self.anchor.frame else {
            return None;
        };
        let base = self.base.as_deref()?;
        (base.body == body && base.on_pad(at)).then(|| base.pad.length())
    }

    /// Returns the distance from the centre of the planet of what a jet meets in unit direction
    /// `dir`: the ground, the sea over it, or the landing pad where it is higher.
    fn floor(&self, dir: DVec3) -> f64 {
        let globe = self.terrain.globe();
        let level = globe.height(dir).max(globe.sea_level());
        let ground = globe.radius() + f64::from(level);
        self.pad(dir * ground).map_or(ground, |pad| ground.max(pad))
    }

    /// Returns the soil the ground at the planet position `at` gives as dust and the speed at
    /// which it settles: none on the landing pad.
    fn dust(&self, at: DVec3) -> Option<(Soil, f32)> {
        if self.pad(at).is_some() {
            return None;
        }
        let (globe, dir) = (self.terrain.globe(), at.normalize_or_zero());
        let soil = loose(globe.record().class, globe.surface(dir), globe.height(dir))?;
        Some((soil, settle(soil, globe.gravity() as f32)))
    }

    /// Returns the wind in frame axes.
    fn breeze(&self) -> Vec3 {
        (self.anchor.rotation.inverse() * self.wind.velocity().as_dvec3()).as_vec3()
    }
}

/// The pooled puffs, with the material each wears.
type Pool<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Mote,
        &'static mut MeshMaterial3d<StandardMaterial>,
    ),
>;

/// Throws the whole puffs a jet owes from `hit` into parked puffs, and returns what it still owes:
/// nothing when the ground there gives no dust or the pool is full.
fn throw(
    field: &Field,
    (hit, owed): (&Hit, f32),
    (pool, cloud, serial): (&mut Pool, &mut Cloud, &mut u32),
) -> f32 {
    let Some((soil, speed)) = field.dust(hit.at) else {
        return 0.0;
    };
    let mut owed = owed;
    while owed >= 1.0 {
        let Some((mut mote, mut paint)) = pool.iter_mut().find(|(mote, _)| mote.life <= 0.0) else {
            return 0.0;
        };
        *serial = serial.wrapping_add(1);
        *mote = fling(hit.at, (*serial, hit.strength), speed);
        paint.set_if_neq(MeshMaterial3d(cloud.soils[soil as usize].clone()));
        cloud.live += 1;
        owed -= 1.0;
    }
    owed
}

/// Raises dust where the jets of every ship meet loose ground within their reach, at a rate that
/// falls off with the run of each jet, in air thick enough to hold it up.
pub fn raise(
    field: Field,
    mut ships: Query<(&Placed, &mut Exhaust)>,
    mut pool: Pool,
    (mut cloud, mut materials): (ResMut<Cloud>, ResMut<Assets<StandardMaterial>>),
    mut serial: Local<u32>,
) {
    if !field.round() {
        return;
    }
    let dt = field.time.delta_secs().min(MAX_DT);
    recolour(
        &mut cloud,
        field.terrain.globe().record().class,
        &mut materials,
    );
    for (placed, mut exhaust) in &mut ships {
        let thick = exhaust.air >= THIN;
        for (k, jet) in jets(placed, &exhaust).iter().enumerate() {
            let hit = impinge(jet, |dir| field.floor(dir)).filter(|_| thick);
            exhaust.dust[k] = match hit {
                Some(hit) => {
                    let owed = (exhaust.dust[k] + RATE * hit.strength * dt).min(OWED);
                    throw(&field, (&hit, owed), (&mut pool, &mut cloud, &mut serial))
                }
                None => 0.0,
            };
        }
    }
}

/// Flies the puffs in the air, drawn as round discs turned to the camera, and parks the spent
/// ones and every one once the anchor leaves the surface of the planet. The turn of the camera
/// is the one of the last frame: a round disc shows no lag.
pub fn drift(
    field: Field,
    camera: Single<&GlobalTransform, With<GameCamera>>,
    mut cloud: ResMut<Cloud>,
    mut pool: Query<(&mut Mote, &mut Transform)>,
) {
    let (dt, wind, round) = (
        field.time.delta_secs().min(MAX_DT),
        field.breeze(),
        field.round(),
    );
    let turn = camera.rotation();
    let mut live = 0;
    for (mut mote, mut place) in &mut pool {
        if mote.life <= 0.0 {
            continue;
        }
        if !round || mote.age >= mote.life {
            (mote.life, place.scale) = (0.0, Vec3::ZERO);
            continue;
        }
        reread(&mut mote, REREAD, |dir| field.floor(dir));
        step(&mut mote, wind, dt);
        *place = pose(&field.anchor, turn, &mote);
        live += 1;
    }
    cloud.live = live;
}
