//! Hand-made lives, worlds and sites for the tests of the population modules.

mod ground;

pub use ground::{height, light, sample, site};

use crate::life::body::{Body, Locomotion};
use crate::life::{
    Activity, Chemistry, Energy, Guild, Habitat, Life, Plan, Role, Rung, Solvent, Species,
    SpeciesId, Structure, TissueKind, Tolerance, World, history,
};

/// Terrain seed of the test planet.
pub const PLANET: u64 = 0x5EED_C0DE;

/// Returns a world like the home planet: heavy, temperate, half sea, 6 km in radius.
pub fn homeworld() -> World {
    World {
        gravity: 22.0,
        pressure: 1.0,
        temperature: 288.0,
        flux: 1.0,
        radius: 6000.0,
        ocean: 0.5,
        star: 5800.0,
        tilt: 0.0,
        liquids: [0.5, 0.0375, 0.0375, 0.5],
    }
}

/// Returns a role of `guild` living in `habitat` and out at `activity`, at trophic `level`.
pub fn role(guild: Guild, habitat: Habitat, activity: Activity, level: f32) -> Role {
    Role {
        guild,
        habitat,
        activity,
        niche: 0.5,
        range: 0.1,
        centre: 0.4,
        level,
    }
}

/// Returns species number `index` of the test planet with `role` and adult `mass` in kilograms.
pub fn species(index: u16, role: Role, mass: f32) -> Species {
    let plan = if role.guild == Guild::Producer {
        Plan::Colonial
    } else {
        Plan::Skeletal
    };
    Species {
        id: SpeciesId {
            planet: PLANET,
            index,
        },
        name: format!("Species {index}"),
        epithet: String::new(),
        plan,
        archetype: 0,
        role,
        tolerance: Tolerance {
            cold: 255.0,
            heat: 315.0,
            dry: 0.1,
            wet: 0.95,
        },
        history: history::of(u64::from(index), &role, mass, Rung::Skeletal, &homeworld()),
        tissues: Vec::new(),
        signature: TissueKind::Hide,
        glow: None,
        body: empty_body(),
        prey: Vec::new(),
    }
}

/// Returns a body with no parts.
fn empty_body() -> Body {
    Body {
        parts: Vec::new(),
        legs: Vec::new(),
        spine: None,
        neck: None,
        tail: None,
        tentacles: Vec::new(),
        wings: None,
        locomotion: Locomotion::Walk,
        length: 1.0,
        height: 0.5,
        features: Vec::new(),
    }
}

/// Returns a skeletal life: a plant, two grazers, a browser, a hunter, an apex hunter, a filter
/// feeder in the sea, a shore grazer, a scavenger and an ambusher. Species 1 is the most common
/// ground grazer.
pub fn meadow_life() -> Life {
    use Activity::{Day, Night, Twilight};
    use Guild::{Ambusher, Browser, Filter, Grazer, Hunter, Producer, Scavenger};
    use Habitat::{Ground, Shore, Water};
    let table: [(Role, f32, &[u16]); 10] = [
        (role(Producer, Ground, Day, 1.0), 1.0, &[]),
        (role(Grazer, Ground, Day, 2.0), 40.0, &[0]),
        (role(Grazer, Ground, Day, 2.0), 400.0, &[0]),
        (role(Browser, Ground, Twilight, 2.0), 300.0, &[0]),
        (role(Hunter, Ground, Night, 3.0), 80.0, &[1, 7]),
        (role(Hunter, Ground, Day, 4.0), 300.0, &[1, 2, 3, 4]),
        (role(Filter, Water, Day, 2.0), 5.0, &[0]),
        (role(Grazer, Shore, Twilight, 2.0), 2.0, &[0]),
        (role(Scavenger, Ground, Day, 3.0), 8.0, &[]),
        (role(Ambusher, Ground, Night, 3.0), 30.0, &[1, 7]),
    ];
    let species = (0u16..).zip(table).map(|(i, (role, mass, prey))| Species {
        prey: prey.to_vec(),
        ..species(i, role, mass)
    });
    Life {
        rung: Rung::Skeletal,
        chemistry: chemistry(),
        species: species.collect(),
        relations: Vec::new(),
        keystone: None,
    }
}

/// Returns a water and carbon biochemistry under a yellow star.
fn chemistry() -> Chemistry {
    Chemistry {
        solvent: Solvent::Water,
        energy: Energy::Light,
        structure: Structure::Carbon,
        pigment: [0.1, 0.5, 0.1],
        glow: [0.2, 0.8, 0.6],
    }
}
