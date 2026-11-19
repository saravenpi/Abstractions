//! The records of life: tissues, roles, tolerances, life histories, species and the life of a planet.

use super::body::Body;
use super::kinds::{Activity, Chemistry, Glow, Guild, Habitat, Plan, Relation, Rung};

/// Stable identity of a species: the seed of its planet and its index there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpeciesId {
    /// Terrain seed of the planet.
    pub planet: u64,
    /// Index into [`Life::species`].
    pub index: u16,
}

/// The kinds of tissue every body, plants included, is built from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TissueKind {
    /// Bone.
    Bone,
    /// Hide.
    Hide,
    /// Shell.
    Shell,
    /// Gel.
    Gel,
    /// Crystal.
    Crystal,
    /// Fibre.
    Fibre,
    /// Resin.
    Resin,
    /// Gland.
    Gland,
}

/// What a tissue does to what touches it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reaction {
    /// Nothing.
    Inert,
    /// Burns with acid.
    Acid,
    /// Poisons.
    Venom,
    /// Shocks.
    Charge,
}

/// How a tissue is patterned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pattern {
    /// One colour.
    Plain,
    /// Stripes across the body.
    Stripes,
    /// Spots.
    Spots,
    /// Bands along the body.
    Bands,
    /// Dark back, pale belly.
    Countershade,
}

/// One tissue record: its physics and its look. Crafting in later versions reads these numbers only.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tissue {
    /// Which kind.
    pub kind: TissueKind,
    /// Mohs-like hardness, 0 to 10.
    pub hardness: f32,
    /// Toughness, 0 to 1.
    pub toughness: f32,
    /// Density in kilograms per cubic metre.
    pub density: f32,
    /// Elasticity, 0 to 1.
    pub elasticity: f32,
    /// Highest temperature it stands, in kelvin.
    pub heat: f32,
    /// Heat and charge conductivity, 0 to 1.
    pub conductivity: f32,
    /// What it does to what touches it.
    pub reaction: Reaction,
    /// Linear RGB base colour.
    pub colour: [f32; 3],
    /// Pattern.
    pub pattern: Pattern,
    /// Gloss, 0 to 1.
    pub gloss: f32,
    /// Translucency, 0 to 1.
    pub translucency: f32,
    /// Self glow, 0 to 1.
    pub glow: f32,
}

/// The place of a species in the food web.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Role {
    /// How it eats.
    pub guild: Guild,
    /// Where it lives.
    pub habitat: Habitat,
    /// When it is out.
    pub activity: Activity,
    /// Niche value of the niche model, 0 to 1.
    pub niche: f32,
    /// Width of its feeding range on the niche axis.
    pub range: f32,
    /// Centre of its feeding range on the niche axis.
    pub centre: f32,
    /// Trophic level: 1 for producers, 2 for what eats them, and so on.
    pub level: f32,
}

/// The climate a species lives in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tolerance {
    /// Coldest temperature in kelvin.
    pub cold: f32,
    /// Hottest temperature in kelvin.
    pub heat: f32,
    /// Driest moisture, 0 to 1.
    pub dry: f32,
    /// Wettest moisture, 0 to 1.
    pub wet: f32,
}

/// The life history of a species, from its mass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct History {
    /// Adult mass in kilograms.
    pub mass: f32,
    /// Life span in seconds of game time.
    pub lifespan: f32,
    /// Young per brood.
    pub clutch: u8,
    /// Smallest group it lives in, 1 for a loner.
    pub group_min: u8,
    /// Largest group it lives in.
    pub group_max: u8,
    /// Parental care, 0 (none) to 1 (guards its young).
    pub care: f32,
    /// Top speed in metres per second on its planet.
    pub speed: f32,
    /// Radius of its home range in metres.
    pub range: f32,
}

/// One species of a planet.
#[derive(Debug, Clone, PartialEq)]
pub struct Species {
    /// Stable identity.
    pub id: SpeciesId,
    /// Proper name.
    pub name: String,
    /// Plain epithet, such as "the glowing hunter".
    pub epithet: String,
    /// Body plan.
    pub plan: Plan,
    /// Archetype silhouette within the plan.
    pub archetype: u8,
    /// Place in the web.
    pub role: Role,
    /// Climate it lives in.
    pub tolerance: Tolerance,
    /// Life history.
    pub history: History,
    /// Tissues: the main one first, then at most two accents.
    pub tissues: Vec<Tissue>,
    /// The tissue whose value is farthest from the planet's average.
    pub signature: TissueKind,
    /// Its glow, if any.
    pub glow: Option<Glow>,
    /// Its body, parts and limbs.
    pub body: Body,
    /// Indices of the species it eats.
    pub prey: Vec<u16>,
}

/// The life of one planet: built once when the planet is entered, a pure function of the planet and its star.
#[derive(Debug, Clone, PartialEq)]
pub struct Life {
    /// How far evolution got.
    pub rung: Rung,
    /// Biochemistry.
    pub chemistry: Chemistry,
    /// Every species, producers first. `species[i].id.index == i`.
    pub species: Vec<Species>,
    /// Relationships other than eating.
    pub relations: Vec<Relation>,
    /// The species whose loss would move the most others.
    pub keystone: Option<u16>,
}
