//! Tests of the material numbers of tissues and stones against the measured table.

use super::*;
use crate::life::body::PartKind;
use crate::life::tissue::HOME;
use crate::life::{Pattern, Reaction, Tissue, TissueKind};
use crate::universe::PlanetRecord;

/// Returns the tissue of `kind` at its home values.
pub(super) fn home(kind: TissueKind) -> Tissue {
    let [hardness, toughness, density, elasticity, heat, conductivity] = HOME[kind as usize];
    Tissue {
        kind,
        hardness,
        toughness,
        density,
        elasticity,
        heat,
        conductivity,
        reaction: Reaction::Inert,
        colour: [0.5; 3],
        pattern: Pattern::Plain,
        gloss: 0.0,
        translucency: 0.0,
        glow: 0.0,
    }
}

/// Returns true when `a` lies within `share` of `b`.
pub(super) fn near(a: f32, b: f32, share: f32) -> bool {
    (a - b).abs() <= share * b.abs()
}

/// A home tissue from a clean kill has the measured numbers of its anchor: femur, wet antler and
/// wet horn keratin, as in the plan's table.
#[test]
fn home_tissues_match_the_table() {
    let cases = [
        (TissueKind::Bone, PartKind::Thigh, 17.7, 99.0, 6.35),
        (TissueKind::Bone, PartKind::Horn, 7.5, 108.0, 10.3),
        (TissueKind::Hide, PartKind::Horn, 0.85, 40.0, 2.56),
        (TissueKind::Shell, PartKind::Segment, 3.05, 70.0, 4.12),
    ];
    for (kind, part, e, strength, k) in cases {
        let class = Class::of(kind, part).expect("a building class");
        let m = material(class, &home(kind), Grade::Clean).expect("the kind fits");
        assert!(near(m.stiffness, e, 0.01), "{class:?} E {}", m.stiffness);
        assert!(near(m.strength, strength, 0.01), "{class:?} {}", m.strength);
        assert!(near(m.toughness, k, 0.01), "{class:?} K {}", m.toughness);
    }
}

/// Obsidian and glass tissue sit at the K_IC of glass and count as brittle; flint and bone do not.
#[test]
fn glass_is_brittle() {
    let glass = material(Class::Glass, &home(TissueKind::Crystal), Grade::Clean).unwrap();
    assert!(glass.brittle() && Mineral::Obsidian.material().brittle());
    assert!((0.7..=0.8).contains(&Mineral::Obsidian.material().toughness));
    assert!(!Mineral::Flint.material().brittle());
    assert!(!Class::Bone.anchor().brittle());
}

/// A poor kill weakens the tissue, a silica species' bone is harder and less tough than home bone,
/// and a tissue that does not fit its class gives nothing.
#[test]
fn species_and_grade_scale_the_numbers() {
    let bone = home(TissueKind::Bone);
    let clean = material(Class::Bone, &bone, Grade::Clean).unwrap();
    let poor = material(Class::Bone, &bone, Grade::Poor).unwrap();
    assert!(poor.strength < clean.strength && poor.toughness < clean.toughness);
    let silica = Tissue {
        hardness: 7.0,
        toughness: 0.36,
        ..bone
    };
    let hard = material(Class::Bone, &silica, Grade::Clean).unwrap();
    assert!(hard.hardness > clean.hardness && hard.toughness < clean.toughness);
    assert!(material(Class::Wood, &bone, Grade::Clean).is_none());
    assert_eq!(Class::of(TissueKind::Gland, PartKind::Head), None);
}

/// Kill grades follow the plan: one hit clean, two or three good, more or blunt poor.
#[test]
fn grades_follow_the_hits() {
    assert_eq!(Grade::of(1, false), Grade::Clean);
    assert_eq!(Grade::of(3, false), Grade::Good);
    assert_eq!(Grade::of(5, false), Grade::Poor);
    assert_eq!(Grade::of(1, true), Grade::Poor);
}

/// A temperate world of seas has basalt, obsidian and flint but no ice; a frozen one has ice and no
/// flint; a gas giant has no ground at all.
#[test]
fn minerals_follow_the_planet() {
    let temperate = PlanetRecord::TEMPERATE;
    let found = |r: &PlanetRecord| Mineral::ALL.map(|m| m.lies_on(r));
    assert_eq!(found(&temperate), [true, true, true, false]);
    let frozen = PlanetRecord {
        class: crate::universe::PlanetClass::Ice,
        water: crate::universe::Water::Ice,
        temperature: 200.0,
        ..temperate
    };
    assert_eq!(found(&frozen), [true, true, false, true]);
    let giant = PlanetRecord {
        class: crate::universe::PlanetClass::GasGiant,
        ..temperate
    };
    assert_eq!(found(&giant), [false; 4]);
}
