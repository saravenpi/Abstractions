//! The pictures of the guide: the silhouette of a species in the list, by its growth for a plant
//! and its body plan for an animal, the icon that tells a plant from an animal on the card, and
//! the icon of each fact and each group by its label.

use super::seen::Seen;
use crate::life::{Guild, Life, Plan, Species};
use crate::ui::Icon;

/// Icons of the growth architectures of plants, in the order of [`super::words::GROWTHS`].
pub const GROWTHS: [Icon; 8] = [
    Icon::Tree,
    Icon::Flower,
    Icon::Grass,
    Icon::Vine,
    Icon::Bulb,
    Icon::Cap,
    Icon::Kelp,
    Icon::Crystal,
];

/// Returns the icon of a body plan.
pub fn plan(plan: Plan) -> Icon {
    match plan {
        Plan::Skeletal => Icon::Skeleton,
        Plan::Amorphous => Icon::Blob,
        Plan::Radial => Icon::Radial,
        Plan::Segmented => Icon::Segments,
        Plan::Colonial => Icon::Swarm,
        Plan::Crystalline => Icon::Crystal,
    }
}

/// Returns the silhouette of `species` in the list: its growth for a plant, its body plan for an
/// animal.
pub fn silhouette(species: &Species) -> Icon {
    match species.role.guild {
        Guild::Producer => GROWTHS[usize::from(species.archetype) % GROWTHS.len()],
        _ => plan(species.plan),
    }
}

/// Returns the icon that says what `species` is: a plant or an animal.
pub fn class(species: &Species) -> Icon {
    match species.role.guild {
        Guild::Producer => Icon::Plant,
        _ => Icon::Animal,
    }
}

/// The icon of each label of a card of the guide and of the pack, headings and facts alike.
pub const LABELS: [(&str, Icon); 42] = [
    ("Body", Icon::Skeleton),
    ("Life", Icon::Life),
    ("Habitat", Icon::Terrain),
    ("Relations", Icon::Link),
    ("Size", Icon::Radius),
    ("Weight", Icon::Mass),
    ("Plan", Icon::Class),
    ("Limbs", Icon::Footprint),
    ("Moves", Icon::Moves),
    ("Growth", Icon::Plant),
    ("Food", Icon::Light),
    ("Diet", Icon::Diet),
    ("Herd", Icon::Herd),
    ("Active", Icon::Clock),
    ("Young", Icon::Egg),
    ("Temper", Icon::Shield),
    ("Yields", Icon::Yield),
    ("Lives", Icon::Home),
    ("Climate", Icon::Temperature),
    ("Moisture", Icon::Moisture),
    ("Eats", Icon::Eats),
    ("Eaten by", Icon::Skull),
    ("Pollinates", Icon::Flower),
    ("Pollinated by", Icon::Flower),
    ("Sows", Icon::Grass),
    ("Sown by", Icon::Grass),
    ("Rides", Icon::Users),
    ("Carries", Icon::Users),
    ("Looks like", Icon::Copy),
    ("Copied by", Icon::Copy),
    ("Builds", Icon::Workshop),
    ("Load", Icon::Pack),
    ("Carried", Icon::Mass),
    ("Pace", Icon::Speed),
    ("Mass", Icon::Mass),
    ("Wear", Icon::Repair),
    ("Swing", Icon::Speed),
    ("Depth", Icon::Height),
    ("Lasts", Icon::Age),
    ("This world", Icon::Planet),
    ("Treatments", Icon::Workshop),
    ("Made of", Icon::Yield),
];

/// Returns the icon of the fact or the group labelled `label`, or nothing for a label the guide
/// does not know, which keeps the room of an icon so the labels stay in line.
pub fn of_label(label: &str) -> Option<Icon> {
    let known = LABELS.iter().find(|(name, _)| *name == label);
    known.map(|(_, icon)| *icon)
}

/// Returns the icon beside the name on the card of species `index` of `life`: a plant or an
/// animal for a seen species, a question mark for one not seen yet, the guide itself otherwise.
pub fn sign(life: Option<&Life>, index: Option<u16>, seen: &Seen) -> Icon {
    let species = life
        .zip(index)
        .and_then(|(l, i)| l.species.get(usize::from(i)));
    match species {
        Some(species) if seen.has(species.id) => class(species),
        Some(_) => Icon::Unknown,
        None => Icon::Guide,
    }
}
