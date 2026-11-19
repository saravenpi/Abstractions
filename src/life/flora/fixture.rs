//! Hand-built life for the plant tests: producer groups, a planet of them and the world they stand on.

use crate::life::body::{Body, Locomotion};
use crate::life::{
    Activity, Chemistry, Energy, Guild, Habitat, History, Life, Plan, Relation, Role, Rung,
    Solvent, Species, SpeciesId, Structure, TissueKind, Tolerance, World,
};

/// Pigment of a planet under a yellow star.
pub const GREEN: [f32; 3] = [0.06, 0.3, 0.04];

/// A temperate wet climate of water life.
pub const MILD: Tolerance = Tolerance {
    cold: 280.0,
    heat: 300.0,
    dry: 0.4,
    wet: 0.9,
};

/// Returns a world of `gravity` metres per second squared around a star of `star` kelvin, otherwise like home.
pub fn world(gravity: f32, star: f32) -> World {
    World {
        gravity,
        pressure: 1.0,
        temperature: 288.0,
        flux: 1.0,
        radius: 5000.0,
        ocean: 0.6,
        star,
    }
}

/// Returns producer group `index` of planet `planet`, of growth architecture `arch`, living in `tolerance`.
pub fn producer(planet: u64, index: u16, arch: u8, tolerance: Tolerance) -> Species {
    let habitat = if arch == 6 {
        Habitat::Water
    } else {
        Habitat::Ground
    };
    Species {
        id: SpeciesId { planet, index },
        name: format!("Plant {index}"),
        epithet: "the plant".into(),
        plan: Plan::Colonial,
        archetype: arch,
        role: Role {
            guild: Guild::Producer,
            habitat,
            activity: Activity::Day,
            niche: 0.0,
            range: 0.0,
            centre: 0.0,
            level: 1.0,
        },
        tolerance,
        history: History {
            mass: 10.0,
            lifespan: 1.0e6,
            clutch: 1,
            group_min: 1,
            group_max: 1,
            care: 0.0,
            speed: 0.0,
            range: 0.0,
        },
        tissues: Vec::new(),
        signature: TissueKind::Fibre,
        glow: None,
        body: Body {
            parts: Vec::new(),
            legs: Vec::new(),
            spine: None,
            neck: None,
            tail: None,
            tentacles: Vec::new(),
            wings: None,
            locomotion: Locomotion::Swarm,
            length: 0.0,
            height: 0.0,
            features: Vec::new(),
        },
        prey: Vec::new(),
    }
}

/// Returns the life of a planet of `rung` with the producers `species`, water life of `pigment`, and
/// `relations` between them and animals that are not listed.
pub fn life(
    rung: Rung,
    pigment: [f32; 3],
    species: Vec<Species>,
    relations: Vec<Relation>,
) -> Life {
    Life {
        rung,
        chemistry: Chemistry {
            solvent: Solvent::Water,
            energy: Energy::Light,
            structure: Structure::Carbon,
            pigment,
            glow: [0.1, 0.8, 0.6],
        },
        species,
        relations,
        keystone: None,
    }
}

/// Returns a living planet `planet` of rung `Skeletal` with one producer of each architecture in `archs`.
pub fn garden(planet: u64, archs: &[u8], relations: Vec<Relation>) -> Life {
    let species = archs
        .iter()
        .enumerate()
        .map(|(i, &arch)| producer(planet, i as u16, arch, MILD))
        .collect();
    life(Rung::Skeletal, GREEN, species, relations)
}
