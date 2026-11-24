//! The looks of the planet's animals: one mesh per part of each species, made the first time an
//! animal of that species is drawn and shared by every individual, one material for all, and one
//! glowing material per species that glows.

use bevy::prelude::*;

use super::glow::{lamp, lit};
use super::mesh::part_mesh_toned;
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
    /// Glowing materials by species index, made with its meshes; `None` for a species that does
    /// not glow.
    lamps: Vec<Option<Handle<StandardMaterial>>>,
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

    /// Returns the material of each part of species `index` of `life`: its glowing material for the
    /// parts that glow, the shared one for the others. Call [`Looks::parts`] first.
    pub fn paints(
        &mut self,
        (life, index): (&Life, u16),
        materials: &mut Assets<StandardMaterial>,
    ) -> Vec<Handle<StandardMaterial>> {
        let skin = self.skin(materials);
        let Some(species) = life.species.get(usize::from(index)) else {
            return Vec::new();
        };
        let glow = self.lamps.get(usize::from(index)).cloned().flatten();
        lit(species)
            .into_iter()
            .map(|on| glow.clone().filter(|_| on).unwrap_or_else(|| skin.clone()))
            .collect()
    }

    /// Returns the species index and the glowing material of every glowing species made so far.
    pub fn lamps(&self) -> impl Iterator<Item = (usize, &Handle<StandardMaterial>)> {
        self.lamps
            .iter()
            .enumerate()
            .filter_map(|(i, l)| l.as_ref().map(|l| (i, l)))
    }

    /// Returns the part meshes of species `index` of `life` on the planet of seed `planet`, made the
    /// first time with its glowing material; meshes of another planet are dropped first.
    pub fn parts(
        &mut self,
        (planet, life): (u64, &Life),
        index: u16,
        (meshes, materials): (&mut Assets<Mesh>, &mut Assets<StandardMaterial>),
    ) -> &[Option<Handle<Mesh>>] {
        if self.planet != planet || self.meshes.len() != life.species.len() {
            self.planet = planet;
            self.meshes = vec![None; life.species.len()];
            self.lamps = vec![None; life.species.len()];
        }
        let (Some(slot), Some(s)) = (
            self.meshes.get_mut(usize::from(index)),
            life.species.get(usize::from(index)),
        ) else {
            return &[];
        };
        if slot.is_none() {
            self.lamps[usize::from(index)] = lamp(s, materials);
        }
        slot.get_or_insert_with(|| {
            let accent = |tissue: u8| {
                s.tissues
                    .iter()
                    .enumerate()
                    .find(|(idx, _)| *idx != usize::from(tissue))
                    .map(|(_, t)| t)
            };
            s.body
                .parts
                .iter()
                .map(|p| {
                    s.tissues
                        .get(usize::from(p.tissue))
                        .map(|t| meshes.add(part_mesh_toned(p, t, accent(p.tissue))))
                })
                .collect()
        })
    }
}
