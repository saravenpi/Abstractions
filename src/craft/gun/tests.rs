//! Unit tests for firearm ballistics energy, reload preference, firing wear, and ADS blend math.

use super::ads::{ads_fov_scale, ads_step};
use super::ballistics::bullet_energy;
use super::fire::{consume_round, finish_reload};
use super::{Gun, GunKind, GunState};
use crate::craft::Pack;
use crate::craft::matter::{Mineral, Stuff};

/// Bullet impact energy scales with chassis damage and bullet mineral density.
#[test]
fn bullet_energy_scales_with_damage_and_density() {
    let pistol_flint = bullet_energy(GunKind::FlintlockPistol.stats().damage, Mineral::Flint);
    assert!((pistol_flint - 83.6).abs() < 1.0, "{pistol_flint}");
    let rifle_basalt = bullet_energy(GunKind::MineralRifle.stats().damage, Mineral::Basalt);
    assert!(rifle_basalt >= 170.0, "{rifle_basalt}");
    let rifle_copper = bullet_energy(GunKind::MineralRifle.stats().damage, Mineral::Copper);
    assert!(rifle_copper > rifle_basalt);
    let pistol_ice = bullet_energy(GunKind::FlintlockPistol.stats().damage, Mineral::Ice);
    assert!(pistol_ice < pistol_flint);
}

/// Reloading prefers the chambered mineral and falls back to other bullets in pack order.
#[test]
fn reload_prefers_chambered_mineral_and_consumes_pack() {
    let mut pack = Pack::default();
    let id = pack
        .keep_gun(GunKind::MineralRifle, Mineral::Ironstone)
        .expect("gun slot");
    let gun = pack.gun_mut(id).expect("gun");
    gun.loaded = 1;
    gun.ammo = Some(Mineral::Flint);
    pack.add_stack(Stuff::Bullet(Mineral::Flint), Mineral::Flint.material(), 2);
    pack.add_stack(
        Stuff::Bullet(Mineral::Copper),
        Mineral::Copper.material(),
        5,
    );
    assert!(finish_reload(&mut pack, id));
    let gun = pack.gun(id).expect("gun");
    assert_eq!(gun.loaded, 3);
    assert_eq!(gun.ammo, Some(Mineral::Flint));
    assert_eq!(pack.count_of(&Stuff::Bullet(Mineral::Flint)), 0);
    assert_eq!(pack.count_of(&Stuff::Bullet(Mineral::Copper)), 5);

    assert!(finish_reload(&mut pack, id));
    let gun = pack.gun(id).expect("gun");
    assert_eq!(gun.loaded, 4);
    assert_eq!(gun.ammo, Some(Mineral::Copper));
    assert_eq!(pack.count_of(&Stuff::Bullet(Mineral::Copper)), 4);
}

/// Firing consumes one round, clears ammo when empty, bumps wear, and sets cooldown and recoil.
#[test]
fn firing_decrements_ammo_clears_when_empty_and_adds_wear() {
    let mut gun = Gun::new(0, GunKind::FlintlockPistol, Mineral::Basalt);
    gun.loaded = 1;
    gun.ammo = Some(Mineral::Quartz);
    let mut state = GunState::default();
    let fired = consume_round(&mut gun, &mut state);
    assert_eq!(
        fired,
        Some((GunKind::FlintlockPistol, Mineral::Basalt, Mineral::Quartz))
    );
    assert_eq!(gun.loaded, 0);
    assert_eq!(gun.ammo, None);
    assert!((gun.wear - 0.015).abs() < 1.0e-6);
    assert!(state.cooldown > 0.0 && state.recoil > 0.0);
    assert!(consume_round(&mut gun, &mut state).is_none());
}

/// ADS blend smooths toward target and scales rifle FOV tighter than pistol FOV.
#[test]
fn ads_blend_and_fov_scale_math() {
    assert!((ads_fov_scale(0.0, GunKind::MineralRifle) - 1.0).abs() < 1.0e-6);
    assert!((ads_fov_scale(1.0, GunKind::MineralRifle) - 0.62).abs() < 1.0e-6);
    assert!((ads_fov_scale(1.0, GunKind::FlintlockPistol) - 0.78).abs() < 1.0e-6);
    let up = ads_step(0.0, true, 0.1);
    assert!(up > 0.0 && up < 1.0, "{up}");
    let down = ads_step(1.0, false, 0.1);
    assert!(down > 0.0 && down < 1.0, "{down}");
}
