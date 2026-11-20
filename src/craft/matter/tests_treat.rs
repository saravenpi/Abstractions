//! Tests of treatments: the trade-offs they make and the worlds that allow them.

use super::tests::{home, near};
use super::*;
use crate::life::TissueKind;
use crate::universe::{PlanetClass, PlanetRecord};

/// Sun-drying wet antler multiplies stiffness by 2.5 and toughness by 0.8, landing near measured
/// dry antler (17.1 GPa, 188 MPa).
#[test]
fn sun_drying_trades_toughness_for_stiffness() {
    let wet = material(Class::Antler, &home(TissueKind::Bone), Grade::Clean).unwrap();
    let dry = Treatment::SunDry.apply(wet);
    assert!(near(dry.stiffness, wet.stiffness * 2.5, 1e-4));
    assert!(near(dry.toughness, wet.toughness * 0.8, 1e-4));
    assert!(near(dry.stiffness, 17.1, 0.1) && near(dry.strength, 188.0, 0.05));
}

/// Fire-hardening makes wood slightly harder, 30 % weaker, and cuts its work of fracture by 36 %.
#[test]
fn fire_hardening_trades_strength_for_hardness() {
    let wood = Class::Wood.anchor();
    let fired = Treatment::FireHarden.apply(wood);
    assert!(fired.hardness > wood.hardness);
    assert!(near(fired.strength, wood.strength * 0.7, 1e-4));
    assert!(near(fired.work(), wood.work() * 0.64, 1e-3));
}

/// A treatment counts once, and an empty set changes nothing.
#[test]
fn treatments_count_once() {
    let bone = Class::Bone.anchor();
    let once = Treatments::default().with(Treatment::SunDry);
    assert_eq!(once.with(Treatment::SunDry), once);
    assert_eq!(Treatments::default().apply(bone), bone);
    assert_eq!(once.apply(bone), Treatment::SunDry.apply(bone));
}

/// A temperate world dries hides but neither fires nor freezes them; a lava world fires them; a cold
/// world freezes them; an airless one dries nothing.
#[test]
fn worlds_decide_the_treatments() {
    let temperate = PlanetRecord::TEMPERATE;
    let allowed = |r: &PlanetRecord| Treatment::ALL.map(|t| t.possible(r));
    assert_eq!(allowed(&temperate), [true, false, false]);
    let lava = PlanetRecord {
        class: PlanetClass::Lava,
        temperature: 900.0,
        ..temperate
    };
    assert!(allowed(&lava)[1]);
    let cold = PlanetRecord {
        temperature: 180.0,
        ..temperate
    };
    assert!(allowed(&cold)[2]);
    let airless = PlanetRecord {
        pressure: 0.0,
        ..temperate
    };
    assert!(!allowed(&airless)[0]);
}
