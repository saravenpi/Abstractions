//! The looks of the planet's animals: one mesh per part of each species, made the first time an
//! animal of that species is drawn and shared by every individual, and one material for all.

use bevy::prelude::*;

use super::mesh::part_mesh;
use crate::life::Life;

/// Meshes of the species of one planet, by species then part, and their shared material.
#[derive(Resource, Default)]
pub struct Looks {
    /// Seed of the planet the meshes were made for.
    planet: u64,
    /// The material every part is drawn with: its colours are in the mesh.
    skin: Option<Handle<StandardMaterial>>,
    /// Meshes by species index, then part index, made once; `None` for a part with no tissue.
    meshes: Vec<Option<Vec<Option<Handle<Mesh>>>>>,
}

impl Looks {
    /// Returns the shared material, made the first time.
    pub fn skin(&mut self, materials: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
        self.skin
            .get_or_insert_with(|| {
                materials.add(StandardMaterial {
                    perceptual_roughness: 0.8,
                    ..default()
                })
            })
            .clone()
    }

    /// Returns the part meshes of species `index` of `life` on the planet of seed `planet`, made the
    /// first time; meshes of another planet are dropped first.
    pub fn parts(
        &mut self,
        (planet, life): (u64, &Life),
        index: u16,
        meshes: &mut Assets<Mesh>,
    ) -> &[Option<Handle<Mesh>>] {
        if self.planet != planet || self.meshes.len() != life.species.len() {
            self.planet = planet;
            self.meshes = vec![None; life.species.len()];
        }
        let (Some(slot), Some(s)) = (
            self.meshes.get_mut(usize::from(index)),
            life.species.get(usize::from(index)),
        ) else {
            return &[];
        };
        slot.get_or_insert_with(|| {
            s.body
                .parts
                .iter()
                .map(|p| {
                    s.tissues
                        .get(usize::from(p.tissue))
                        .map(|t| meshes.add(part_mesh(p, t)))
                })
                .collect()
        })
    }
}
