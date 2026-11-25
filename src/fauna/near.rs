//! The herds the compass, the minimap and the map mark: live marks of the herds drawn near the robot.
//!
//! A herd has a mark only while some of its living animals are drawn, so a mark always leads to animals
//! the player can see. The mark stands at the middle of those animals, flyers and floaters where they
//! fly, and moves with them every frame; it goes the frame no animal of the herd is drawn any more. No
//! mark is kept from memory. At most [`MARKS`] are shown, the nearest first, each over [`MARK_APART`]
//! from a nearer one so that one mark stands for herds that keep together.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Dead;
use super::beast::Beast;
use super::gate::footing;
use super::tune::{MARK_APART, MARK_SIZE, MARKS};
use super::want::{Lead, drawn};
use crate::life::body::Locomotion;
use crate::life::{Life, Species, SpeciesId};
use crate::player::Player;
use crate::space::Anchor;
use crate::universe::BodyId;
use crate::world::Terrain;

/// One herd marked: where its drawn animals are.
#[derive(Debug, Clone, PartialEq)]
pub struct Sighting {
    /// Identity of the herd.
    pub id: u64,
    /// The middle of its drawn living animals, in metres from the planet centre in its turning frame.
    pub at: DVec3,
    /// Its species.
    pub species: SpeciesId,
    /// The name of its species.
    pub name: String,
    /// Metres along the ground from the robot to the mark.
    pub metres: f64,
}

/// The herds the compass and the minimap mark, nearest first, and the planet they are on.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Nearby {
    /// The planet the herds are on, none off a living planet.
    pub body: Option<BodyId>,
    /// The herds drawn now, at most [`MARKS`].
    pub herds: Vec<Sighting>,
}

/// One drawn living animal of a herd: the herd, the species index and where the animal is.
pub type Animal = (u64, u16, DVec3);

/// Returns true when animals of `species` are worth a mark: drawn, not a swarm, and at least
/// [`MARK_SIZE`] long, so a mark leads to animals the player can see.
fn markable(species: &Species) -> bool {
    drawn(species)
        && species.body.locomotion != Locomotion::Swarm
        && species.body.length >= MARK_SIZE
}

/// Returns the marks of the herds of `animals` of `life`, on a planet of `radius` metres, seen from the
/// robot at `robot`: one per herd of a markable species, at the middle of its animals, the nearest
/// [`MARKS`] at least [`MARK_APART`] apart.
pub fn marks(life: &Life, animals: &[Animal], (robot, radius): (DVec3, f64)) -> Vec<Sighting> {
    let mut herds: Vec<Sighting> = Vec::new();
    for &(id, index, _) in animals {
        let Some(species) = life.species.get(usize::from(index)).filter(|s| markable(s)) else {
            continue;
        };
        if herds.iter().any(|s| s.id == id) {
            continue;
        }
        let mine = animals.iter().filter(|a| a.0 == id).map(|a| a.2);
        let (sum, count) = mine.fold((DVec3::ZERO, 0u32), |(sum, n), at| (sum + at, n + 1));
        let at = sum / f64::from(count.max(1));
        let metres = at.normalize().angle_between(robot.normalize()) * radius;
        herds.push(Sighting {
            id,
            at,
            species: species.id,
            name: species.name.clone(),
            metres,
        });
    }
    herds.sort_by(|a, b| {
        let boss = |s: &Sighting| {
            life.species
                .get(usize::from(s.species.index))
                .is_some_and(|sp| sp.boss)
        };
        boss(b).cmp(&boss(a)).then(a.metres.total_cmp(&b.metres))
    });
    let mut kept: Vec<Sighting> = Vec::with_capacity(MARKS);
    for s in herds {
        if kept.len() < MARKS && kept.iter().all(|k| k.at.distance(s.at) > MARK_APART) {
            kept.push(s);
        }
    }
    kept
}

/// Marks, every frame, the herds whose living animals are drawn near the robot, at the middle of them,
/// in the place the walk just moved them to; on no living planet under the robot, marks nothing.
pub fn mark(
    (terrain, anchor): (Res<Terrain>, Res<Anchor>),
    robot: Query<&Transform, With<Player>>,
    beasts: Query<&Beast, Without<Dead>>,
    mut nearby: ResMut<Nearby>,
    mut animals: Local<Vec<Animal>>,
) {
    animals.clear();
    let body = footing(&anchor);
    let (Some(life), Ok(robot)) = (terrain.globe().life(), robot.single()) else {
        nearby.set_if_neq(Nearby {
            body,
            herds: Vec::new(),
        });
        return;
    };
    for beast in beasts.iter().filter(|b| !b.fade.leaving()) {
        if let Lead::Herd { id, .. } = beast.lead {
            animals.push((id, beast.index, beast.spot.planet(beast.motion.root.place)));
        }
    }
    let robot = anchor.to_frame(robot.translation);
    let herds = marks(life, &animals, (robot, terrain.globe().radius()));
    nearby.set_if_neq(Nearby { body, herds });
}

/// Registers the live marks: every frame in play on a living planet, after the walk moved the animals.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        mark.after(super::walk::walk)
            .run_if(crate::screens::playing)
            .run_if(super::gate::living),
    );
}
