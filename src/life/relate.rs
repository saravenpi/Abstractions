//! Relationships other than eating: pollinators and seed carriers tie animals to plants, small
//! species ride large ones, harmless species copy the warning colours of toxic ones, builders leave
//! structures, and one keystone holds the web together.

use super::{Build, Guild, Habitat, Plan, Reaction, Relation, Species};

#[cfg(test)]
mod tests;

/// Growth architectures that flower: trunk, rosette, vine and bulb.
const FLOWERING: [u8; 4] = [0, 1, 3, 4];

/// Returns true when `species` carries a toxic tissue.
fn toxic(species: &Species) -> bool {
    species
        .tissues
        .iter()
        .any(|t| t.reaction != Reaction::Inert)
}

/// Returns the pollinators and seed carriers among `species`: nectar feeders pollinate and fruit eaters
/// sow the flowering plants they eat, or the first flowering plant when they eat none.
fn carriers(species: &[Species]) -> Vec<Relation> {
    let flowers: Vec<u16> = species
        .iter()
        .filter(|s| s.role.guild == Guild::Producer && FLOWERING.contains(&s.archetype))
        .map(|s| s.id.index)
        .collect();
    let mut out = Vec::new();
    for animal in species
        .iter()
        .filter(|s| matches!(s.role.guild, Guild::Nectar | Guild::Frugivore))
    {
        let eaten: Vec<u16> = animal
            .prey
            .iter()
            .copied()
            .filter(|p| flowers.contains(p))
            .collect();
        let plants = if eaten.is_empty() {
            flowers.first().copied().into_iter().collect()
        } else {
            eaten
        };
        let a = animal.id.index;
        out.extend(plants.into_iter().map(|plant| match animal.role.guild {
            Guild::Nectar => Relation::Pollinates { animal: a, plant },
            _ => Relation::Sows { animal: a, plant },
        }));
    }
    out
}

/// Returns the riders among `species`: every parasite, and every animal under a hundredth of the
/// mass of the largest plant eater, ride that plant eater.
fn riders(species: &[Species]) -> Vec<Relation> {
    let host = species
        .iter()
        .filter(|s| matches!(s.role.guild, Guild::Grazer | Guild::Browser))
        .max_by(|a, b| a.history.mass.total_cmp(&b.history.mass));
    let Some(host) = host else {
        return Vec::new();
    };
    species
        .iter()
        .filter(|s| s.role.guild != Guild::Producer && s.id != host.id)
        .filter(|s| {
            s.role.guild == Guild::Parasite
                || (s.role.level >= 2.3 && s.history.mass < host.history.mass / 100.0)
        })
        .map(|s| Relation::Rides {
            rider: s.id.index,
            host: host.id.index,
        })
        .collect()
}

/// Returns the first harmless animal of `species`, not yet `used`, that shares the habitat of the
/// toxic animal `model`, or nothing when `model` shows no warning.
fn mimic(species: &[Species], model: usize, used: &[bool]) -> Option<usize> {
    let warns = species[model].role.guild != Guild::Producer && toxic(&species[model]);
    let habitat = species[model].role.habitat;
    let fits = |i: usize, s: &Species| {
        !used[i]
            && i != model
            && s.role.guild != Guild::Producer
            && s.role.habitat == habitat
            && !toxic(s)
    };
    warns
        .then(|| species.iter().enumerate().position(|(i, s)| fits(i, s)))
        .flatten()
}

/// Makes one harmless animal in the habitat of each toxic animal of `species` copy its warning
/// colours and pattern, and returns the pairs.
fn mimics(species: &mut [Species]) -> Vec<Relation> {
    let mut out = Vec::new();
    let mut used = vec![false; species.len()];
    for model in 0..species.len() {
        let Some(copy) = mimic(species, model, &used) else {
            continue;
        };
        used[copy] = true;
        let warning = species[model]
            .tissues
            .iter()
            .find(|t| t.reaction != Reaction::Inert)
            .copied();
        if let (Some(look), Some(main)) = (warning, species[copy].tissues.first_mut()) {
            (main.colour, main.pattern) = (look.colour, look.pattern);
        }
        out.push(Relation::Mimics {
            mimic: copy as u16,
            model: model as u16,
        });
    }
    out
}

/// Returns what `species` builds, by plan and guild: reefs and hives for swarms, webs for segmented
/// ambushers, mounds and burrows for segmented ground dwellers, nests for skeletal flyers and
/// caring parents, and burrows for small skeletal ground animals.
fn build(species: &Species) -> Option<Build> {
    let (role, mass) = (&species.role, species.history.mass);
    match (species.plan, role.habitat, role.guild) {
        (_, _, Guild::Producer) => None,
        (Plan::Colonial, Habitat::Water, _) => Some(Build::Reef),
        (Plan::Colonial, _, _) => Some(Build::Hive),
        (Plan::Segmented, _, Guild::Ambusher) => Some(Build::Web),
        (Plan::Segmented, Habitat::Ground, _) if mass < 1.0 => Some(Build::Mound),
        (Plan::Segmented, Habitat::Ground, _) => Some(Build::Burrow),
        (Plan::Skeletal, Habitat::Air, _) => Some(Build::Nest),
        (Plan::Skeletal, Habitat::Ground, _) if mass < 5.0 => Some(Build::Burrow),
        (Plan::Skeletal, _, _) if species.history.care > 0.5 => Some(Build::Nest),
        _ => None,
    }
}

/// Returns the keystone of `species`: the hunter or ambusher highest in the web, else the first
/// builder in `relations`.
fn keystone(species: &[Species], relations: &[Relation]) -> Option<u16> {
    let top = species
        .iter()
        .filter(|s| matches!(s.role.guild, Guild::Hunter | Guild::Ambusher))
        .max_by(|a, b| a.role.level.total_cmp(&b.role.level))
        .map(|s| s.id.index);
    top.or_else(|| {
        relations.iter().find_map(|r| match r {
            Relation::Builds { species, .. } => Some(*species),
            _ => None,
        })
    })
}

/// Returns the relationships other than eating among `species`, and its keystone. Mimics take the
/// colours of their models.
pub fn of(species: &mut [Species]) -> (Vec<Relation>, Option<u16>) {
    let mut relations = carriers(species);
    relations.extend(riders(species));
    relations.extend(mimics(species));
    relations.extend(species.iter().filter_map(|s| {
        build(s).map(|build| Relation::Builds {
            species: s.id.index,
            build,
        })
    }));
    let keystone = keystone(species, &relations);
    (relations, keystone)
}
