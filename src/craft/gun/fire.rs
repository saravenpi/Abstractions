//! Firearm input handling: aiming down sights, reloading from pack bullets, and firing shots.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::ads::{FLASH_INTENSITY, ads_step};
use super::{Gun, GunKind, GunState, MuzzleFlash, ShotFired};
use crate::condition::luck;
use crate::craft::Pack;
use crate::craft::matter::{Mineral, Stuff};
use crate::effects::Fling;
use crate::render::GameCamera;
use crate::screens::Screen;
use crate::view::CameraPose;

/// Inputs and camera read by [`fire`].
#[derive(SystemParam)]
pub struct FireIo<'w, 's> {
    /// Commands used to spawn muzzle flash point lights.
    commands: Commands<'w, 's>,
    /// Frame delta and elapsed time for timers and deterministic spread.
    time: Res<'w, Time>,
    /// Mouse buttons for firing and ADS.
    buttons: Res<'w, ButtonInput<MouseButton>>,
    /// Keyboard state for reload input.
    keys: Res<'w, ButtonInput<KeyCode>>,
    /// Latest world-space camera pose from the camera rig.
    pose: Option<Res<'w, CameraPose>>,
    /// Fallback camera global transform when [`CameraPose`] is unavailable.
    cameras: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
}

/// Message writers used when a firearm discharges.
#[derive(SystemParam)]
pub struct FireOut<'w> {
    /// Shot tracer spawn event writer.
    shots: MessageWriter<'w, ShotFired>,
    /// Spatial gunshot audio event writer.
    gunshots: MessageWriter<'w, crate::audio::Gunshot>,
    /// Muzzle smoke fleck burst writer.
    flings: MessageWriter<'w, Fling>,
}

/// Finds the first available bullet mineral in `pack`, preferring `preferred`.
pub fn find_ammo(pack: &Pack, preferred: Option<Mineral>) -> Option<Mineral> {
    if let Some(m) = preferred.filter(|m| pack.count_of(&Stuff::Bullet(*m)) > 0) {
        return Some(m);
    }
    Mineral::ALL
        .into_iter()
        .find(|m| pack.count_of(&Stuff::Bullet(*m)) > 0)
}

/// Consumes bullets from `pack` to top up firearm `gun_id`, returning true when any round loaded.
pub fn finish_reload(pack: &mut Pack, gun_id: u32) -> bool {
    let Some((capacity, loaded, ammo)) = pack
        .gun(gun_id)
        .map(|g| (g.kind.stats().capacity, g.loaded, g.ammo))
    else {
        return false;
    };
    let Some(m) = (loaded < capacity).then(|| find_ammo(pack, ammo)).flatten() else {
        return false;
    };
    let avail = pack.count_of(&Stuff::Bullet(m)).min(u32::from(u8::MAX)) as u8;
    let need = (capacity - loaded).min(avail);
    if need == 0 || !pack.take_count(&Stuff::Bullet(m), u16::from(need)) {
        return false;
    }
    if let Some(gun) = pack.gun_mut(gun_id) {
        gun.loaded += need;
        gun.ammo = Some(m);
    }
    true
}

/// Attempts to chamber and fire one round from `gun`, updating ammo, wear, cooldown, and recoil.
pub fn consume_round(gun: &mut Gun, state: &mut GunState) -> Option<(GunKind, Mineral, Mineral)> {
    if gun.loaded == 0 || state.cooldown > 0.0 || state.reloading > 0.0 {
        return None;
    }
    let ammo = gun.ammo.unwrap_or(Mineral::Basalt);
    gun.loaded -= 1;
    if gun.loaded == 0 {
        gun.ammo = None;
    }
    gun.wear = (gun.wear + 0.015).min(1.0);
    let stats = gun.kind.stats();
    state.cooldown = stats.cadence;
    let kick = if gun.kind == GunKind::MineralRifle {
        1.0
    } else {
        0.75
    };
    state.recoil_vel += 2.6 * kick;
    state.recoil = (state.recoil + 0.35 * kick).min(1.2);
    state.flash_age = 0.0;
    Some((gun.kind, gun.barrel, ammo))
}

/// Applies deterministic hipfire and wear spread to `fwd`.
fn spread_dir(fwd: Vec3, ads: f32, wear: f32, seed: u32) -> Vec3 {
    let cone = (1.0 - ads.clamp(0.0, 1.0)) * 0.022 + wear.clamp(0.0, 1.0) * 0.015;
    if cone <= 1.0e-5 {
        return fwd.normalize_or(Vec3::NEG_Z);
    }
    let (u, v) = fwd.any_orthonormal_pair();
    let dx = luck(seed ^ 0x51a7) * 2.0 - 1.0;
    let dy = luck(seed ^ 0xa3c9) * 2.0 - 1.0;
    (fwd + (u * dx + v * dy) * cone).normalize_or(Vec3::NEG_Z)
}

/// Spawns the warm muzzle light and dark gunsmoke puff at `origin` along `dir`.
fn muzzle_fx(
    commands: &mut Commands,
    flings: &mut MessageWriter<Fling>,
    origin: Vec3,
    dir: Vec3,
    seed: u32,
) {
    let light = PointLight {
        intensity: FLASH_INTENSITY,
        range: 10.0,
        radius: 0.12,
        color: Color::srgb(1.0, 0.72, 0.35),
        shadow_maps_enabled: false,
        ..default()
    };
    let at = Transform::from_translation(origin);
    let flash = MuzzleFlash { age: 0.0 };
    commands.spawn((flash, light, at, DespawnOnExit(Screen::Gameplay)));
    let (u, v) = dir.any_orthonormal_pair();
    let (at, colour, wet, tag) = (origin, [0.22, 0.22, 0.24], false, None);
    for k in 0..4u32 {
        let rx = luck(seed ^ k.wrapping_mul(0x9e37) ^ 0x51) * 2.0 - 1.0;
        let ry = luck(seed ^ k.wrapping_mul(0x9e37) ^ 0xa3) * 2.0 - 1.0;
        let puff = (dir + (u * rx + v * ry) * 0.35).normalize_or(dir);
        let velocity = puff * (2.6 + 1.4 * luck(seed ^ k ^ 0x3c));
        let radius = 0.028 + 0.008 * k as f32;
        flings.write(Fling {
            at,
            velocity,
            radius,
            colour,
            wet,
            tag,
        });
    }
}

/// Emits audio, [`ShotFired`], muzzle flash, and smoke for a discharged round.
fn emit_shot(
    io: &mut FireIo,
    out: &mut FireOut,
    state: &GunState,
    (kind, barrel, ammo): (GunKind, Mineral, Mineral),
    wear: f32,
) {
    let pose = io
        .pose
        .as_deref()
        .filter(|p| p.fov > 0.0 && p.rotation.is_finite());
    let cam = pose
        .map(|p| (p.position, p.rotation * Vec3::NEG_Z))
        .or_else(|| {
            io.cameras
                .single()
                .ok()
                .map(|g| (g.translation(), g.forward().as_vec3()))
        });
    let (cam_pos, cam_fwd) = cam.unwrap_or((Vec3::ZERO, Vec3::NEG_Z));
    let origin = state
        .muzzle_world
        .unwrap_or_else(|| cam_pos + cam_fwd * 0.65);
    let seed = (io.time.elapsed_secs() * 1000.0) as u32;
    let dir = spread_dir(cam_fwd, state.ads, wear, seed);
    let rifle = kind == GunKind::MineralRifle;
    out.gunshots
        .write(crate::audio::Gunshot { at: origin, rifle });
    out.shots.write(ShotFired {
        origin,
        dir,
        kind,
        barrel,
        ammo,
        local: true,
    });
    muzzle_fx(&mut io.commands, &mut out.flings, origin, dir, seed);
}

/// Starts or advances the reload timer for `gun_id`.
fn tick_reload(pack: &mut Pack, state: &mut GunState, gun_id: u32, reload_key: bool, dt: f32) {
    if state.reloading > 0.0 {
        state.reloading = (state.reloading - dt).max(0.0);
        if state.reloading <= 0.0 {
            finish_reload(pack, gun_id);
        }
    } else if reload_key && let Some(gun) = pack.gun(gun_id) {
        let stats = gun.kind.stats();
        if gun.loaded < stats.capacity && find_ammo(pack, gun.ammo).is_some() {
            state.reloading = stats.reload;
        }
    }
}

/// Handles local firearm ADS, reload, and firing while the player has control.
pub fn fire(
    mut io: FireIo,
    mut out: FireOut,
    mut pack: ResMut<Pack>,
    mut state: ResMut<GunState>,
    mut last_id: Local<Option<u32>>,
) {
    let Some((gun_id, wear)) = pack.held_gun().map(|g| (g.id, g.wear)) else {
        (state.reloading, state.ads, *last_id) = (0.0, 0.0, None);
        return;
    };
    if *last_id != Some(gun_id) {
        (state.reloading, *last_id) = (0.0, Some(gun_id));
    }
    let dt = io.time.delta_secs();
    state.ads = ads_step(state.ads, io.buttons.pressed(MouseButton::Right), dt);
    tick_reload(
        &mut pack,
        &mut state,
        gun_id,
        io.keys.just_pressed(KeyCode::KeyR),
        dt,
    );
    let trigger =
        io.buttons.just_pressed(MouseButton::Left) || io.buttons.pressed(MouseButton::Left);
    if trigger
        && let Some(gun) = pack.gun_mut(gun_id)
        && let Some(shot) = consume_round(gun, &mut state)
    {
        emit_shot(&mut io, &mut out, &state, shot, wear);
    }
}
