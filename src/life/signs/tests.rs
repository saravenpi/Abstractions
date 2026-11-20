//! Tests of the signs on the ground: tracks follow recent herd paths and fade, plant eaters leave
//! droppings and grazed patches, dens lie at herd homes, bones follow emptied carcasses, and the same
//! seed and time give the same signs.

use bevy::math::DVec3;

use super::rig::{RADIUS, carcass, find, on, search, signs, travels};
use super::{Mark, Sign};
use crate::life::herd::fixtures::{height, meadow_life};
use crate::life::herd::{Herd, Watch, arc};
use crate::life::{Guild, Habitat, Life};

/// Returns a travelling herd of `life`, the time it was found at, where it was 20 s before, and its
/// prints within 60 m of there.
fn trail(life: &Life) -> (Herd, f64, DVec3, Vec<Sign>) {
    let (herd, time) = find(life, travels).expect("a travelling herd");
    let back = on(|g| {
        let mut watch = Watch::new(life, g, &search(herd.centre, 400.0, time));
        watch.herd(life, g, herd.id, time - 20.0)
    });
    let at = back.expect("the herd 20 s before").centre;
    let mine = |s: &Sign| s.mark == Mark::Print && s.species == herd.species;
    let prints = signs(life, at, time).into_iter().filter(mine).collect();
    (herd, time, at, prints)
}

/// Prints of a walking herd lie on its path of the last minutes.
#[test]
fn tracks_follow_recent_herd_paths() {
    let (herd, _, at, prints) = trail(&meadow_life());
    println!(
        "{} prints of herd {} near where it was 20 s ago",
        prints.len(),
        herd.id
    );
    assert!(!prints.is_empty(), "no print on the path of the herd");
    let spread = f64::from(herd.spacing) * f64::from(herd.base).sqrt() * 2.0;
    let path = spread + 20.0 * f64::from(herd.speed) + 2.0;
    assert!(prints.iter().any(|p| arc(p.at, at) * RADIUS < path));
}

/// A print stays where it is and ages as time goes on, and is gone once its trail time is past.
#[test]
fn tracks_fade() {
    let life = meadow_life();
    let (_, time, at, prints) = trail(&life);
    let print = prints[0];
    let later = signs(&life, at, time + 60.0);
    let same = later
        .iter()
        .find(|s| s.mark == Mark::Print && s.at == print.at);
    let same = same.expect("the print a minute later");
    let aged = print.fresh - same.fresh;
    assert!(
        (aged - 0.25).abs() < 1e-3,
        "{} then {}",
        print.fresh,
        same.fresh
    );
    let gone = signs(&life, at, time + super::tune::TRAIL + 5.0);
    assert!(
        !gone
            .iter()
            .any(|s| s.mark == Mark::Print && s.at == print.at)
    );
}

/// A grazing herd leaves droppings and a grazed patch where it stands.
#[test]
fn grazers_leave_droppings_and_grazed_patches() {
    let life = meadow_life();
    let grazer = |h: &Herd| {
        h.state == crate::life::herd::HerdState::Graze
            && life.species[usize::from(h.species)].role.guild == Guild::Grazer
            && h.count > 4
    };
    let (herd, time) = find(&life, grazer).expect("a grazing herd");
    let here = signs(&life, herd.centre, time + 200.0);
    let mine = |mark| {
        here.iter()
            .filter(|s| s.mark == mark && s.species == herd.species)
            .count()
    };
    println!(
        "{} droppings, {} patches",
        mine(Mark::Dropping),
        mine(Mark::Grazed)
    );
    assert!(mine(Mark::Grazed) > 0, "no grazed patch");
    assert!(mine(Mark::Dropping) > 0, "no dropping");
}

/// Species that tend their young keep dens at the home of their herds, on dry ground, burrows for the
/// small and nests for the large, and none in the water.
#[test]
fn dens_lie_at_herd_homes() {
    let life = meadow_life();
    let keeps = |h: &Herd| {
        let s = &life.species[usize::from(h.species)];
        let home = arc(h.centre, h.home) * RADIUS < 250.0 && height(h.home) > 2.0;
        s.history.care >= super::tune::NEST_CARE && s.role.habitat != Habitat::Water && home
    };
    let (herd, time) = find(&life, keeps).expect("a herd near its home that keeps dens");
    let here = signs(&life, herd.home, time);
    let dens: Vec<_> = here
        .iter()
        .filter(|s| matches!(s.mark, Mark::Nest | Mark::Burrow))
        .collect();
    println!("{} dens at the home of herd {}", dens.len(), herd.id);
    assert!(!dens.is_empty(), "no den at a herd home");
    for den in dens {
        let species = &life.species[usize::from(den.species)];
        assert_ne!(species.role.habitat, Habitat::Water);
        let small = species.role.habitat == Habitat::Ground
            && species.history.mass <= super::tune::BURROW_KG;
        assert_eq!(den.mark == Mark::Burrow, small, "{}", species.name);
    }
}

/// Bones lie where a kill fell only once its meat is gone, and are gone once the bones are.
#[test]
fn bones_appear_after_a_carcass_empties() {
    let life = meadow_life();
    let c = carcass(&life).expect("a kill");
    let bones = |time| {
        signs(&life, c.at, time)
            .iter()
            .any(|s| s.mark == Mark::Bones && arc(s.at, c.at) < 1e-9)
    };
    assert!(!bones(c.meat - 5.0), "bones before the meat is gone");
    assert!(bones(c.meat + 5.0), "no bones once the meat is gone");
    assert!(bones(0.5 * (c.meat + c.bones)), "bones gone too soon");
    assert!(!bones(c.bones + 5.0), "bones left after their time");
}

/// The same life, place and time give the same signs, read from two watches.
#[test]
fn same_seed_same_signs() {
    let life = meadow_life();
    let (herd, time) = find(&life, travels).expect("a travelling herd");
    let (a, b) = (
        signs(&life, herd.centre, time),
        signs(&meadow_life(), herd.centre, time),
    );
    assert!(!a.is_empty());
    assert_eq!(a, b);
}
