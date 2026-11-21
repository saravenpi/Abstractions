//! Blades of grass near the camera: on ground the ground-cover species of a living planet carpet, sparse
//! procedural blades grown by the vertex shader, dense near the camera and thinning out blade by blade with
//! the distance, shorter far off, shrinking away before they go, bent by the wind, cleared under props and on
//! the patches grazers cropped.
//!
//! The ground shader draws the same carpet farther out from the same turf, so a blade always stands in
//! the colour of the ground under it. A planet without ground cover has neither.

mod clear;
mod colour;
mod fade;
mod flight;
mod ground;
mod material;
mod tile;
mod tiles;
pub mod tune;

use std::time::Duration;

use bevy::asset::{AssetEventSystems, embedded_asset};
use bevy::math::DVec3;
use bevy::pbr::MaterialPlugin;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

pub use material::{BladeMaterial, TurfUniform};

use super::props::PropAssets;
use super::scatter::{Flora, turf_of};
use super::{GroundMaterial, Terrain};
use crate::render::GameCamera;
use crate::screens::playing;
use crate::space::Anchor;
use crate::wind::Wind;
use material::{blade_material, wind_map, wind_texel};
use tiles::{Meadow, tend};

/// How often the look of the turf under the camera is read again.
const LOOK: Duration = Duration::from_secs(2);

/// The patches grazers cropped near the camera, as unit directions from the planet centre and radii in
/// metres. The signs of life write them; the blades leave them bare.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Cropped(pub Vec<(DVec3, f32)>);

/// The blade material and its wind texture.
#[derive(Resource, Clone)]
pub struct BladeAssets {
    /// The material every tile of blades shares.
    pub material: Handle<BladeMaterial>,
    /// The one-texel wind texture it reads.
    pub wind: Handle<Image>,
}

/// Returns how thick the turf grows at chart position `w` of `terrain`, 0 where no ground cover grows.
pub fn turf_at(terrain: &Terrain, w: Vec2) -> f32 {
    let chunk = (w / super::CHUNK_SIZE).floor().as_ivec2();
    super::scatter::Carpet::of(terrain, chunk).map_or(0.0, |c| c.at(w))
}

/// Makes the blade material once the ground material exists, sharing its light texture.
fn setup(
    mut commands: Commands,
    props: Res<PropAssets>,
    mut materials: ResMut<Assets<BladeMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let wind = images.add(wind_map());
    let material = materials.add(blade_material(props.place.clone(), wind.clone()));
    commands.insert_resource(BladeAssets { material, wind });
}

/// Reads the turf of the planet at the camera and gives it to the ground and the blade materials when it
/// changed: the ground cover of the season of that latitude, or bare ground.
fn look(
    (terrain, anchor): (Res<Terrain>, Res<Anchor>),
    (camera, transforms): (Query<Entity, With<GameCamera>>, TransformHelper),
    (props, blades): (Res<PropAssets>, Res<BladeAssets>),
    mut grounds: ResMut<Assets<GroundMaterial>>,
    mut materials: ResMut<Assets<BladeMaterial>>,
) {
    let seen = |c| tiles::seen(&terrain, &anchor, &transforms, c);
    let at = camera.single().ok().and_then(seen).unwrap_or(Vec3::ZERO);
    let flora = Flora::of(&terrain, at.xz());
    let turf = TurfUniform::of(flora.as_ref().and_then(turf_of).as_ref());
    if grounds
        .get(&props.material)
        .is_some_and(|m| m.extension.turf != turf)
        && let Some(mut ground) = grounds.get_mut(&props.material)
    {
        ground.extension.turf = turf;
    }
    if materials
        .get(&blades.material)
        .is_some_and(|m| m.extension.turf != turf)
        && let Some(mut blade) = materials.get_mut(&blades.material)
    {
        blade.extension.turf = turf;
    }
}

/// Writes the wind of this frame into the wind texture of the blades, in place.
fn blow(wind: Res<Wind>, blades: Option<Res<BladeAssets>>, mut images: ResMut<Assets<Image>>) {
    let Some(blades) = blades else {
        return;
    };
    let bytes = wind_texel(wind.velocity(), wind.strength);
    let same = |image: &Image| image.data.as_deref() == Some(&bytes[..]);
    if images.get(&blades.wind).is_none_or(same) {
        return;
    }
    if let Some(mut image) = images.get_mut(&blades.wind)
        && let Some(data) = image.data.as_mut()
    {
        data.copy_from_slice(&bytes);
    }
}

/// Embeds the blade shader, registers the material, and in play reads the turf, blows the wind and tends
/// the tiles around the camera.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "blades.wesl");
    app.add_plugins(MaterialPlugin::<BladeMaterial>::default())
        .init_resource::<Cropped>()
        .init_resource::<Meadow>()
        .add_systems(
            Update,
            setup.run_if(
                resource_exists::<PropAssets>.and_then(not(resource_exists::<BladeAssets>)),
            ),
        )
        .add_systems(
            Update,
            (look.run_if(on_timer(LOOK)), tend)
                .run_if(playing)
                .run_if(resource_exists::<BladeAssets>.and_then(resource_exists::<Anchor>)),
        )
        .add_systems(PostUpdate, blow.before(AssetEventSystems));
    crate::render::warm_material::<BladeMaterial>(app);
}

#[cfg(test)]
mod tests_cost;
#[cfg(test)]
mod tests_fade;
#[cfg(test)]
mod tests_pad;
#[cfg(test)]
mod tests_square;
#[cfg(test)]
mod tests_stream;
#[cfg(test)]
mod tests_tile;
