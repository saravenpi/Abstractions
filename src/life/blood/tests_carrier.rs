//! Tests of the carrier and the colour of blood.

use super::*;
use crate::life::{Chemistry, Energy, Plan, Rung, Solvent, Structure};

/// Returns the chemistry of a planet with `solvent`, `energy` and `structure`.
fn chemistry(solvent: Solvent, energy: Energy, structure: Structure) -> Chemistry {
    Chemistry {
        solvent,
        energy,
        structure,
        pigment: [0.2, 0.5, 0.15],
        glow: [1.0, 0.35, 0.1],
    }
}

/// A skeleton on a home-like planet bleeds red, closed and with a brighter artery than vein.
#[test]
fn home_blood_is_red() {
    let home = chemistry(Solvent::Water, Energy::Light, Structure::Carbon);
    let blood = of(&home, Plan::Skeletal, Rung::Skeletal).unwrap();
    assert_eq!(blood.carrier, Carrier::Haem);
    assert!(blood.closed);
    let [r, g, b] = blood.oxygenated;
    assert!(r > 10.0 * g && r > 10.0 * b);
    assert!(blood.oxygenated[0] > blood.spent[0]);
}

/// Bodies with no circulating fluid have no blood, on any planet and at the soft rungs.
#[test]
fn bloodless_bodies_have_none() {
    let home = chemistry(Solvent::Water, Energy::Light, Structure::Carbon);
    for plan in [
        Plan::Amorphous,
        Plan::Radial,
        Plan::Colonial,
        Plan::Crystalline,
    ] {
        assert_eq!(of(&home, plan, Rung::Old), None);
    }
    assert_eq!(of(&home, Plan::Skeletal, Rung::Soft), None);
    assert_eq!(of(&home, Plan::Skeletal, Rung::Mats), None);
}

/// Two skeletal species of one planet share the carrier and so the colour exactly.
#[test]
fn one_planet_shares_the_hue() {
    let world = chemistry(Solvent::Water, Energy::Light, Structure::Carbon);
    let a = of(&world, Plan::Skeletal, Rung::Skeletal);
    let b = of(&world, Plan::Skeletal, Rung::Old);
    assert_eq!(a.map(|x| x.oxygenated), b.map(|x| x.oxygenated));
}

/// A segmented body on the same planet carries the other metal and an open circulation.
#[test]
fn insects_carry_the_other_metal() {
    let home = chemistry(Solvent::Water, Energy::Light, Structure::Carbon);
    let bug = of(&home, Plan::Segmented, Rung::Shelled).unwrap();
    assert_eq!(bug.carrier, Carrier::Haemocyanin);
    assert!(!bug.closed);
    assert!(bug.oxygenated[2] > bug.oxygenated[0]);
}

/// The chemistry picks the carrier: metal crust copper, vents haemerythrin, cold solvents others.
#[test]
fn chemistry_picks_the_carrier() {
    let pick = |s, e, m| of(&chemistry(s, e, m), Plan::Skeletal, Rung::Skeletal).unwrap();
    assert_eq!(
        pick(Solvent::Water, Energy::Light, Structure::Metal).carrier,
        Carrier::Haemocyanin
    );
    assert_eq!(
        pick(Solvent::Water, Energy::Chemistry, Structure::Carbon).carrier,
        Carrier::Haemerythrin
    );
    assert_eq!(
        pick(Solvent::Ammonia, Energy::Light, Structure::Carbon).carrier,
        Carrier::Vanadium
    );
    assert_eq!(
        pick(Solvent::Silicate, Energy::Heat, Structure::Silica).carrier,
        Carrier::Melt
    );
}

/// A stain darkens as it dries and never goes black.
#[test]
fn stains_dry_dark() {
    let home = chemistry(Solvent::Water, Energy::Light, Structure::Carbon);
    let blood = of(&home, Plan::Skeletal, Rung::Skeletal).unwrap();
    assert_eq!(blood.stain(1.0), blood.oxygenated);
    assert!(blood.stain(0.0)[0] < blood.stain(1.0)[0]);
    assert!(blood.stain(0.0)[0] > 0.0);
}
