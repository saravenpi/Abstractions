//! Hand-made lives, worlds and sites for the tests of the population modules.

use bevy::math::DVec3;

use crate::life::body::{Body, Locomotion};
use crate::life::census::Site;
use crate::life::{
    Activity, Chemistry, Energy, Guild, Habitat, Life, Plan, Role, Rung, Solvent, Species,
    SpeciesId, Structure, TissueKind, Tolerance, World, history,
};
use crate::world::Biome;

/// Terrain seed of the test planet.
pub const PLANET: u64 = 0x5EED_C0DE;

/// Returns a world like Kodya: heavy, temperate, half sea, 6 km in radius.
pub fn kodya() -> World {
    World {
        gravity: 22.0,
        pressure: 1.0,
        temperature: 288.0,
        flux: 1.0,
        radius: 6000.0,
        ocean: 0.5,
        star: 5800.0,
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
        history: history::of(u64::from(index), &role, mass, Rung::Skeletal, &kodya()),
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
/// feeder in the sea and a shore grazer. Species 1 is the most common ground grazer.
pub fn meadow_life() -> Life {
    use Activity::{Day, Night, Twilight};
    use Guild::{Browser, Filter, Grazer, Hunter, Producer};
    use Habitat::{Ground, Shore, Water};
    let table = [
        (role(Producer, Ground, Day, 1.0), 1.0),
        (role(Grazer, Ground, Day, 2.0), 40.0),
        (role(Grazer, Ground, Day, 2.0), 400.0),
        (role(Browser, Ground, Twilight, 2.0), 300.0),
        (role(Hunter, Ground, Night, 3.0), 80.0),
        (role(Hunter, Ground, Day, 4.0), 300.0),
        (role(Filter, Water, Day, 2.0), 5.0),
        (role(Grazer, Shore, Twilight, 2.0), 2.0),
    ];
    Life {
        rung: Rung::Skeletal,
        chemistry: Chemistry {
            solvent: Solvent::Water,
            energy: Energy::Light,
            structure: Structure::Carbon,
            pigment: [0.1, 0.5, 0.1],
            glow: [0.2, 0.8, 0.6],
        },
        species: (0u16..)
            .zip(table)
            .map(|(i, (role, mass))| species(i, role, mass))
            .collect(),
        relations: Vec::new(),
        keystone: None,
    }
}

/// Returns a site on land or sea at `height` metres, in a temperate meadow under `daylight`.
pub fn site(height: f32, daylight: f32) -> Site {
    Site {
        height,
        temperature: 288.0,
        moisture: 0.5,
        biome: Biome::Meadow,
        liquid: height < 0.0,
        daylight,
    }
}

/// Returns the ground height in metres of a test planet of hills, bays and seas at `dir`.
pub fn height(dir: DVec3) -> f32 {
    let d = dir.as_vec3();
    300.0 * ((3.0 * d.x).sin() * (2.0 * d.y).cos() + 0.3 * (5.0 * d.z).sin()) - 40.0
}

/// Returns the site of the test planet at `dir` under `daylight`.
pub fn sample(dir: DVec3, daylight: f32) -> Site {
    site(height(dir), daylight)
}
