//! Species names: each region of a galaxy has its own sounds and a small vocabulary of roots for
//! the traits a name speaks of, so neighbouring systems sound alike. Every name comes with a plain
//! epithet from its signature and its guild, such as "the glowing hunter".

use bevy::math::IVec3;

use super::{Guild, Habitat, Reaction, Species, TissueKind};
use crate::universe::StarSpec;
use crate::universe::hash::{Rng, cell, child};

#[cfg(test)]
mod tests;

/// Sounds a syllable may start with.
const ONSETS: [&str; 24] = [
    "b", "c", "d", "f", "g", "h", "k", "l", "m", "n", "p", "r", "s", "t", "v", "z", "th", "sh",
    "br", "dr", "kr", "st", "vr", "qu",
];

/// Vowel sounds.
const VOWELS: [&str; 12] = [
    "a", "e", "i", "o", "u", "ae", "ia", "ei", "ou", "ar", "or", "y",
];

/// Sounds a name may end with.
const CODAS: [&str; 9] = ["n", "r", "s", "l", "x", "th", "m", "k", "sh"];

/// Sectors per side of the cube of sectors that shares one set of sounds.
const REGION: i32 = 4;

/// Nouns of the growth architectures, in the order of the contract.
const PLANTS: [&str; 8] = [
    "tree", "rosette", "mat", "vine", "bulb", "cap", "kelp", "lattice",
];

/// The sounds of a region: a few onsets, vowels and codas out of the full lists.
struct Voice {
    /// Onsets of the region.
    onsets: [&'static str; 6],
    /// Vowels of the region.
    vowels: [&'static str; 4],
    /// Codas of the region.
    codas: [&'static str; 3],
}

impl Voice {
    /// Returns the voice of the region of seed `region`.
    fn of(region: u64) -> Self {
        let mut rng = Rng::new(region);
        let mut pick = |list: &[&'static str]| list[rng.below(list.len() as u32) as usize];
        Self {
            onsets: [(); 6].map(|_| pick(&ONSETS)),
            vowels: [(); 4].map(|_| pick(&VOWELS)),
            codas: [(); 3].map(|_| pick(&CODAS)),
        }
    }

    /// Returns one syllable of the region, an onset and a vowel, drawn from `rng`.
    fn syllable(&self, rng: &mut Rng) -> String {
        let onset = self.onsets[rng.below(6) as usize];
        format!("{onset}{}", self.vowels[rng.below(4) as usize])
    }
}

/// Returns the seed of the naming region of `star`: its galaxy and its cube of 4 by 4 by 4 sectors.
pub fn region(star: &StarSpec) -> u64 {
    let galaxy = cell(0x4E41_4D45, star.id.galaxy.0);
    cell(galaxy, star.id.sector.div_euclid(IVec3::splat(REGION)))
}

/// Returns a proper name of a species in the voice of `region`, drawn from `seed`: the region's root
/// for its main trait (its glow, else its signature tissue), one or two more syllables and a coda.
pub fn name(region: u64, seed: u64, species: &Species) -> String {
    let voice = Voice::of(region);
    let root = species.glow.map_or(species.signature as u64, |_| 8);
    let mut roots = Rng::new(child(region, root));
    let mut rng = Rng::new(seed);
    let mut name = voice.syllable(&mut roots);
    for _ in 0..1 + rng.below(2) {
        name.push_str(&voice.syllable(&mut rng));
    }
    name.push_str(voice.codas[rng.below(3) as usize]);
    name[..1].make_ascii_uppercase();
    name
}

/// Returns the adjective of an epithet: the glow, the size at the extremes, else the signature tissue.
fn adjective(species: &Species) -> &'static str {
    let gland = species.tissues.iter().find(|t| t.kind == TissueKind::Gland);
    match (species.glow, species.history.mass, species.signature) {
        (Some(_), _, _) => "glowing",
        (_, m, _) if m > 1000.0 => "great",
        (_, m, _) if m < 0.01 && species.role.guild != Guild::Producer => "tiny",
        (_, _, TissueKind::Bone) => "horned",
        (_, _, TissueKind::Hide) => "thick-skinned",
        (_, _, TissueKind::Shell) => "shelled",
        (_, _, TissueKind::Gel) => "soft",
        (_, _, TissueKind::Crystal) => "glassy",
        (_, _, TissueKind::Fibre) => "stringy",
        (_, _, TissueKind::Resin) => "sticky",
        (_, _, TissueKind::Gland) => match gland.map(|t| t.reaction) {
            Some(Reaction::Venom) => "venomous",
            Some(Reaction::Acid) => "acid",
            Some(Reaction::Charge) => "shocking",
            _ => "musky",
        },
    }
}

/// Returns the noun of an epithet: the growth architecture of a producer, else what the guild does.
fn noun(species: &Species) -> &'static str {
    match species.role.guild {
        Guild::Producer => PLANTS[usize::from(species.archetype) % PLANTS.len()],
        Guild::Hunter if species.role.habitat == Habitat::Air => "hawk",
        Guild::Grazer => "grazer",
        Guild::Browser => "browser",
        Guild::Frugivore => "fruit eater",
        Guild::Nectar => "nectar drinker",
        Guild::Granivore => "seed cracker",
        Guild::Filter => "sifter",
        Guild::Hunter => "hunter",
        Guild::Ambusher => "lurker",
        Guild::Scavenger => "scavenger",
        Guild::Decomposer => "rotter",
        Guild::Parasite => "leech",
    }
}

/// Returns the plain epithet of a species, such as "the glowing hunter".
pub fn epithet(species: &Species) -> String {
    format!("the {} {}", adjective(species), noun(species))
}

/// Names every species of a planet in the voice of `region`, every name different from the others.
pub fn christen(species: &mut [Species], region: u64) {
    let mut taken: Vec<String> = Vec::with_capacity(species.len());
    for s in species.iter_mut() {
        let seed = child(s.id.planet, 0x4E_0000 + u64::from(s.id.index));
        let mut name = name(region, seed, s);
        while taken.contains(&name) {
            name.push_str(Voice::of(region).vowels[taken.len() % 4]);
        }
        if s.epithet.is_empty() {
            s.epithet = epithet(s);
        }
        taken.push(name.clone());
        s.name = name;
    }
}
