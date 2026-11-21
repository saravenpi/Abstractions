//! What a species does when a robot comes near, and what its body gives when it is harvested.
//! Both come from the rules the game plays by: the fight or flight of the senses, and the classes
//! of material the tissues of its parts turn into.

use super::super::senses::stance::{Foe, Me, Stance, choose};
use super::super::senses::tune::ROBOT_KG;
use crate::craft::matter::Class;
use crate::life::body::PartKind;
use crate::life::{Guild, Species};

/// What the card says of an animal that runs from a robot.
pub const SHY: &str = "Shy: runs from you";

/// What the card says of an animal that faces a robot.
pub const BOLD: &str = "Stands its ground";

/// What the card says of an animal that goes for a robot.
pub const HUNTS: &str = "Hunts you when hungry";

/// What the card says of a species whose body gives nothing to build with.
pub const NOTHING: &str = "Nothing to build with";

/// Returns what a healthy, hungry adult of `species` does once it is fully aware of a robot in
/// the open, in plain words, or nothing for a plant.
pub fn temper(species: &Species) -> Option<&'static str> {
    if species.role.guild == Guild::Producer {
        return None;
    }
    let me = Me {
        mass: species.history.mass,
        health: 1.0,
        guild: species.role.guild,
        hungry: true,
        cornered: false,
        guarding: false,
        grace: false,
    };
    let foe = Foe {
        mass: ROBOT_KG,
        robot: true,
    };
    Some(match choose(1.0, &me, &foe) {
        Stance::Attack => HUNTS,
        Stance::Threaten => BOLD,
        _ => SHY,
    })
}

/// Returns the classes of material `species` gives: one for each tissue of each part of its
/// body, or for each of its tissues when its body has no parts, as a plant.
fn classes(species: &Species) -> Vec<Class> {
    let parts = species.body.parts.iter().filter_map(|part| {
        let tissue = species.tissues.get(usize::from(part.tissue))?;
        Class::of(tissue.kind, part.kind)
    });
    let whole = species.tissues.iter();
    let whole = whole.filter_map(|tissue| Class::of(tissue.kind, PartKind::Torso));
    match species.body.parts.is_empty() {
        true => whole.collect(),
        false => parts.collect(),
    }
}

/// Returns the materials `species` gives when harvested, in the order of the classes, or
/// [`NOTHING`] for a body too soft to build with.
pub fn yields(species: &Species) -> String {
    let given = classes(species);
    let all = Class::ALL.into_iter().filter(|class| given.contains(class));
    let names: Vec<&str> = all.map(Class::name).collect();
    match names.is_empty() {
        true => NOTHING.to_string(),
        false => names.join(", "),
    }
}
