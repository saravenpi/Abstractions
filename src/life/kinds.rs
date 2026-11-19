//! The small vocabularies every life generator shares: rungs, chemistry, guilds, habitats and plans.

/// How far evolution got on a planet. Each rung unlocks the body plans of the ones below and its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rung {
    /// Mats and films only: coloured ground and glowing shores, nothing that moves.
    Mats,
    /// Soft life: slimes, jellies, drifting colonies and simple growths.
    Soft,
    /// Shelled and segmented life, first plants with structure, first hunters.
    Shelled,
    /// Skeletal animals, layered forests, herds, and flyers where the air allows.
    Skeletal,
    /// An old ecosystem: armour, venom, speed, mimicry and long food chains.
    Old,
}

/// The liquid life is built around.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Solvent {
    /// Liquid water.
    Water,
    /// Liquid ammonia, on cold worlds.
    Ammonia,
    /// Liquid hydrocarbons, on very cold worlds.
    Hydrocarbon,
    /// Molten silicate, on lava shores.
    Silicate,
}

/// Where producers get their energy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Energy {
    /// Starlight: producers grow on open ground.
    Light,
    /// Chemistry: producers grow at vents and seeps.
    Chemistry,
    /// Heat: producers grow at lava shores and hot ground.
    Heat,
}

/// What hard tissues are made of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Structure {
    /// Carbon fibre and bone, as at home.
    Carbon,
    /// Silica and crystal: hard, brittle and heat proof.
    Silica,
    /// Metal rich shell, under metal rich stars.
    Metal,
}

/// The biochemistry of one living planet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chemistry {
    /// The liquid of life.
    pub solvent: Solvent,
    /// The energy producers live on.
    pub energy: Energy,
    /// What hard tissues are made of.
    pub structure: Structure,
    /// Linear RGB colour of the light gathering pigment of producers, from the colour of the star.
    pub pigment: [f32; 3],
    /// Linear RGB colour of bioluminescence on the planet.
    pub glow: [f32; 3],
}

/// How a consumer eats. Producers have [`Guild::Producer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Guild {
    /// Makes its own food: plants, mats and films.
    Producer,
    /// Crops low plants with a wide flat mouth.
    Grazer,
    /// Reaches high plants with a long neck.
    Browser,
    /// Eats fruit and carries seeds.
    Frugivore,
    /// Drinks nectar and carries pollen.
    Nectar,
    /// Cracks seeds.
    Granivore,
    /// Sieves small food from water or air.
    Filter,
    /// Chases prey.
    Hunter,
    /// Waits hidden for prey.
    Ambusher,
    /// Eats what is already dead.
    Scavenger,
    /// Breaks down dead matter, where things die.
    Decomposer,
    /// Lives on a host.
    Parasite,
}

/// Where a species spends its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Habitat {
    /// On the ground.
    Ground,
    /// In the air.
    Air,
    /// In the sea or ponds.
    Water,
    /// Along shores, in and out of the liquid.
    Shore,
}

/// When a species is out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Activity {
    /// By day.
    Day,
    /// By night.
    Night,
    /// At dawn and dusk.
    Twilight,
}

/// The body plan of a species.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Plan {
    /// A jointed skeleton with legs, necks and tails.
    Skeletal,
    /// Slimes: one soft body that flows.
    Amorphous,
    /// Jellies: a pulsing bell and trailing tentacles.
    Radial,
    /// A chain of segments, with or without many legs.
    Segmented,
    /// A swarm of small identical members.
    Colonial,
    /// Rigid crystal bodies that hop and tumble.
    Crystalline,
}

/// What a glow is for, which sets where it sits and how it behaves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlowUse {
    /// A hunter's bait, on a stalk before the mouth.
    Lure,
    /// Toxic tissue showing itself.
    Warning,
    /// Pulsing patterns at dusk.
    Courtship,
    /// Belly lights that hide a swimmer from below.
    Belly,
}

/// A bioluminescent patch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glow {
    /// What it is for.
    pub purpose: GlowUse,
    /// Linear RGB colour.
    pub colour: [f32; 3],
    /// Emissive strength, 0 to 1.
    pub strength: f32,
}

/// Something a species leaves in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Build {
    /// An earth mound.
    Mound,
    /// A nest of fibre.
    Nest,
    /// A hive.
    Hive,
    /// A hole in the ground.
    Burrow,
    /// A web between plants.
    Web,
    /// A reef in shallow water.
    Reef,
}

/// A relationship between two species other than eating, by index into [`crate::life::Life::species`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Relation {
    /// An animal carries the pollen of a plant.
    Pollinates { animal: u16, plant: u16 },
    /// An animal carries the seeds of a plant.
    Sows { animal: u16, plant: u16 },
    /// A small species lives on a large one.
    Rides { rider: u16, host: u16 },
    /// A harmless species copies the warning pattern of a toxic one.
    Mimics { mimic: u16, model: u16 },
    /// A species leaves structures.
    Builds { species: u16, build: Build },
}
