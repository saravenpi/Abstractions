//! The dust the lift nozzles blow off the dry ground under a ship that hovers or lands: a fixed
//! pool of soft quads, woken where the exhaust meets the real ground and thrown out along it.
//! The rate comes from the thrust of the nozzle and the height above the ground. Over water the
//! churn of the sea throws the spray: see `crate::effects::Churn`.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::state::Exhaust;
use super::tune::DUST;
use crate::render::GameCamera;
use crate::screens::Screen;
use crate::ship::mesh::LIFT_AT;
use crate::ship::sky::Skies;
use crate::space::{Anchor, Frame, Rebased, up};

/// Number of puffs in the pool, made once.
const COUNT: usize = 40;

/// Drag of the air on a puff, per second.
const DRAG: f32 = 1.4;

/// Speed in metres per second of the puffs along the ground at full thrust, and their lift off it.
const SPEED: (f32, f32) = (7.0, 1.5);

/// Largest size of a puff in metres, and seconds it lasts.
const PUFF: (f32, f32) = (2.2, 1.6);

/// One pooled puff, free in render space.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Mote {
    /// Velocity in render axes, in metres per second.
    velocity: Vec3,
    /// Seconds since it was thrown.
    age: f32,
    /// Seconds it lasts. Zero while parked.
    life: f32,
    /// Its largest size in metres.
    size: f32,
}

/// How many puffs are in the air, so that nothing runs while there are none.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Cloud {
    /// Puffs with a life left.
    live: usize,
}

/// Run condition: the pool is made and some ship blows its lift nozzles.
pub fn blowing(ships: Query<&Exhaust>, cloud: Option<Res<Cloud>>) -> bool {
    cloud.is_some() && ships.iter().any(|exhaust| exhaust.lift > 0.1)
}

/// Run condition: some puffs are in the air.
pub fn drifting(cloud: Option<Res<Cloud>>) -> bool {
    cloud.is_some_and(|cloud| cloud.live > 0)
}

/// Builds the pool when gameplay starts, unless it is there: parked puffs at no size.
pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    old: Query<(), With<Mote>>,
) {
    if !old.is_empty() {
        return;
    }
    let quad = meshes.add(Rectangle::new(1.0, 1.0));
    let dust = materials.add(StandardMaterial {
        base_color: Color::srgba(0.72, 0.64, 0.52, 0.45),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        double_sided: true,
        cull_mode: None,
        ..default()
    });
    for _ in 0..COUNT {
        let puff = (Mesh3d(quad.clone()), MeshMaterial3d(dust.clone()));
        let marks = (Rebased, NotShadowCaster, DespawnOnEnter(Screen::Title));
        commands.spawn((
            Mote::default(),
            puff,
            Transform::from_scale(Vec3::ZERO),
            marks,
        ));
    }
    commands.insert_resource(Cloud::default());
}

/// Returns a number in `[0, 1)` for the `serial`th puff.
fn noise(serial: u32) -> f32 {
    let mut x = serial.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 13;
    (x >> 8) as f32 / 16_777_216.0
}

/// Returns where the ground is under the render position `at` and how many metres above it
/// `at` is, when the ground is dry land and within reach of the nozzles.
fn dry(anchor: &Anchor, skies: &Skies, at: Vec3) -> Option<(Vec3, f32)> {
    let (Frame::Surface(_), Some(planet)) = (anchor.frame, skies.planet()) else {
        return None;
    };
    let frame = anchor.to_frame(at);
    let height = skies.height(frame);
    let (radius, clear) = (
        planet.radius + height,
        frame.length() - planet.radius - height,
    );
    let near = (-0.5..f64::from(DUST.0)).contains(&clear);
    (near && height > skies.sea())
        .then(|| (anchor.to_render(frame.normalize() * radius), clear as f32))
}

/// Returns the `serial`th puff thrown at `strength` from the ground at render position `at`,
/// out along the ground in a random direction and a little up.
fn throw(serial: u32, strength: f32, (anchor, at): (&Anchor, Vec3)) -> Mote {
    let vertical = up(anchor, at);
    let (a, b) = vertical.any_orthonormal_pair();
    let turn = noise(serial) * std::f32::consts::TAU;
    let along = a * turn.cos() + b * turn.sin();
    let fast = SPEED.0 * strength * (0.6 + 0.8 * noise(serial ^ 0x55));
    Mote {
        velocity: along * fast + vertical * (SPEED.1 * noise(serial ^ 0xAA)),
        age: 0.0,
        life: PUFF.1 * (0.7 + 0.6 * noise(serial ^ 0x33)),
        size: PUFF.0 * (0.6 + 0.8 * noise(serial ^ 0x77)),
    }
}

/// What the dust is raised and thrown with.
type Hulls = (&'static GlobalTransform, &'static mut Exhaust);

/// Wakes puffs where the lift nozzles of every ship that hovers blow on dry ground.
pub fn raise(
    time: Res<Time>,
    (anchor, skies): (Res<Anchor>, Res<Skies>),
    mut ships: Query<Hulls>,
    mut motes: Query<(&mut Mote, &mut Transform)>,
    (mut cloud, mut serial): (ResMut<Cloud>, Local<u32>),
) {
    let dt = time.delta_secs().min(0.1);
    for (hull, mut exhaust) in &mut ships {
        let strength = exhaust.lift.min(1.0);
        let spots: Vec<(Vec3, f32)> = LIFT_AT
            .iter()
            .filter_map(|at| dry(&anchor, &skies, hull.transform_point(*at)))
            .collect();
        let reach = spots
            .iter()
            .map(|spot| 1.0 - spot.1 / DUST.0)
            .fold(0.0, f32::max);
        exhaust.dust += strength * reach * DUST.1 * dt * spots.len() as f32;
        while exhaust.dust >= 1.0 && !spots.is_empty() {
            exhaust.dust -= 1.0;
            *serial = serial.wrapping_add(1);
            let (at, _) = spots[*serial as usize % spots.len()];
            let free = motes.iter_mut().find(|(mote, _)| mote.life <= 0.0);
            let Some((mut mote, mut place)) = free else {
                exhaust.dust = 0.0;
                break;
            };
            *mote = throw(*serial, strength, (&*anchor, at));
            place.translation = at;
            cloud.live += 1;
        }
    }
}

/// Moves the puffs that are in the air, turned to the camera, and parks the spent ones.
pub fn drift(
    time: Res<Time>,
    camera: Single<&GlobalTransform, With<GameCamera>>,
    mut cloud: ResMut<Cloud>,
    mut motes: Query<(&mut Mote, &mut Transform)>,
) {
    let dt = time.delta_secs().min(0.1);
    let mut live = 0;
    for (mut mote, mut at) in &mut motes {
        if mote.life <= 0.0 {
            continue;
        }
        mote.age += dt;
        let spent = (mote.age / mote.life).min(1.0);
        mote.velocity *= (-DRAG * dt).exp();
        at.translation += mote.velocity * dt;
        let swell = (spent * 5.0).min(1.0) * (1.0 - spent * spent);
        at.scale = Vec3::splat(mote.size * swell * (0.4 + 0.6 * spent));
        at.rotation = camera.rotation();
        if spent >= 1.0 {
            (mote.life, at.scale) = (0.0, Vec3::ZERO);
        } else {
            live += 1;
        }
    }
    cloud.live = live;
}
